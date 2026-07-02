# Multiplayer Protocol Trace — SOLL vs IST

Written 2026-07-02. Purpose: map the entire message flow end-to-end, compare
the intended design against the actual code, identify every gap that could
cause the disconnect problem, and define the target architecture going forward.

---

## Layer map (bottom → top)

```
Layer 0 — Relay server         game/relay/server.ts
Layer 1 — WS transport         websocket-transport.ts
Layer 2 — Multiplayer wrapper  multiplayer.svelte.ts
Layer 3 — Engine wrapper       multiplayer-engine.ts  (createMpEngine)
Layer 4 — Route wiring         /multiplayer/+page.svelte  (lobby)
                                /match/+page.svelte        (match)
```

---

## Part 1: IST — How the code actually works today

### What `$effect` is and why it's wrong here

`$effect` is a Svelte 5 reactive primitive. It re-runs a block of code whenever any
reactive state it reads changes. It was designed for keeping UI in sync with data
(e.g. "when this value changes, update the DOM"). The current code uses it to trigger
network protocol steps:

```ts
// In /match/+page.svelte:
$effect(() => {
  if (mpState.status === "connected") mpEngine?.notifyConnectionOpen();
  else if (mpState.status === "disconnected") mpEngine?.notifyConnectionLost();
});
```

This is the wrong tool for a network state machine because:

1. Effects fire asynchronously — on the next microtask tick after state changes, not
   immediately. Protocol sequencing that depends on ordering guarantees breaks silently.
2. Effects fire across component lifecycles in undefined order. If two components read
   the same state, you can't control which one's effect runs first.
3. An effect has no concept of "this is the first time status became connected" vs
   "status became connected again after a disconnect" — it just re-runs on any change.
4. When a route navigates away, its effects are torn down, but the new route's effects
   aren't set up yet. There's a window where `mpState.status` can change and nothing
   is listening.

**Protocol state transitions should be driven by explicit event callbacks, not reactive
declarations.** The relay/transport already delivers explicit events (`onOpen`, `onClose`,
`onData`) — those are the right hooks.

### Current session setup flow (host first-connect)

```
HOST                          RELAY                         JOINER
  │                             │                             │
  │── WS connect ──────────────►│                             │
  │── {type:"create"} ─────────►│                             │
  │◄─ {type:"created",code} ───│                             │
  │                             │                             │
  │  [mpHost() resolves]        │                             │
  │  [lobby shows code]         │                             │
  │                             │                             │
  │                             │◄─── WS connect ────────────│
  │                             │◄─── {type:"join",code} ────│
  │◄─ {type:"peer-connected"} ─│                             │
  │                             │──── {type:"joined"} ───────►│
  │                             │                             │
  │  [mpState.status="connected"]│[mpJoin() resolves]         │
  │  [$effect in lobby fires]   │[lobby view = "joined"]      │
  │  [goto("../setup/")]        │[waits for session-hello]    │
  │                             │                             │
  │  [/setup/ mounts]           │                             │
  │  [host picks draft mode]    │                             │
  │  [goto("../match/")]        │                             │
  │                             │                             │
  │  [/match/ mounts]           │                             │
  │  [mpEngine created]         │                             │
  │  [$effect sees "connected"] │                             │
  │  [notifyConnectionOpen()]   │                             │
  │── session-hello ──────────────────────────────────────►│
  │  {kind:"session-hello",matchId,phase:"play",seq:0}       │
  │                             │                             │
  │                             │  [lobby handleSessionHelloPeek fires]
  │                             │  [goto("../match/")]        │
```

Problems with this flow:
- Host navigates through /setup/ while joiner sits on lobby watching a spinner.
  Joiner has no indication of what's happening. Host can take arbitrarily long.
- The navigation trigger is `$effect` watching `mpState.status` — fragile.
- `session-hello` drives the joiner's navigation, meaning the joiner is always one
  full route-transition behind the host.
- If the host is on /setup/ and the WS drops, there is no mpEngine anywhere to handle
  the event. The `$effect` in /setup/ does not exist. State gets out of sync silently.
- "session-hello" encoding both "you should navigate now" AND "here is the match
  state" conflates two concerns that should be separate.

### Current reconnect flow

```
HOST (in /match/)             RELAY                         JOINER (in /match/)
  │                             │                             │
  │                             │◄── WS close ───────────────│  (joiner drops)
  │◄─ {type:"peer-disconnected"}│                             │
  │  [mpState.status="disconnected"]                          │
  │  [$effect → notifyConnectionLost()]                       │
  │  [host engine: paused=true]  │                             │
  │                             │  [auto-redial after ~400ms] │
  │                             │◄── WS connect ─────────────│
  │                             │◄── {type:"join",code} ─────│
  │◄─ {type:"peer-connected"} ─│──── {type:"joined"} ───────►│
  │  [mpState.status="connected"]│[mpState.status="connected"]│
  │  [$effect → notifyConnectionOpen()]                       │
  │── session-hello ──────────────────────────────────────►│
  │◄── request-snapshot ─────────────────────────────────────│
  │── snapshot ───────────────────────────────────────────►│
  │  [both: playing again]                                   │
```

This path is structurally OK but also driven by `$effect` — same fragility concerns.

### The pong-age-out bug (primary disconnect symptom)

```
t=0   Both peers connected. Heartbeat starts (5s ping, 500ms tick).
t=5   Ping sent. Pong arrives. lastPongAt = t+ε.
t=10  Ping sent. Pong arrives. lastPongAt = t+ε.
t=15  Ping sent.
      --- JS timers throttled (tab backgrounded / Tauri webview suspended) ---
t=30  Device/tab resumes. The 500ms tick fires.
      now - lastPongAt = 15s ≥ PILL_DISCONNECTED_MS(15s)
      → pong-age-out bridge fires
      → mpState.status = "disconnected"   ← WRONG: WS is still open
      → $effect fires → notifyConnectionLost() → host engine paused
t=31  Queued pong from t=15 ping finally delivered (or ping from t=30 gets response).
      lastPongAt = t+ε
      BUT: nothing restores mpState.status to "connected"
      WS is open. No onOpen() fires. Status stuck at "disconnected" forever.
      Host engine stays paused. Pill stays red. Game is bricked.
```

**Root cause**: The pong-age-out writes `mpState.status = "disconnected"` but there is
no inverse path. `mpState.status = "connected"` is only written by `transport.onOpen()`
via `cbs.onOpen()`. That only fires on a real WS connection event, not on pong arrival.

When the WS is alive but JS was suspended long enough to trigger the age-out, the game
enters a permanently paused state with no recovery. The user has to leave and rejoin.

This would have been rare with PeerJS because a real network drop closes the ICE
connection promptly, producing a clean `conn.close` event that triggers auto-redial and
a genuine new connection. With a WebSocket relay, the WS itself survives through JS
suspension — only the application-layer heartbeat goes stale.

---

## Part 2: SOLL — The target architecture

### Design principles

1. **No `$effect` for protocol sequencing.** Transport events (`onOpen`, `onClose`,
   `onData`) trigger protocol callbacks directly. `$effect` is only for UI rendering.

2. **Both peers navigate together, driven by relay signals.** The relay already sends
   symmetric events: `peer-connected` to host and `joined` to joiner at the same moment.
   Navigation should happen on those events, not via application-layer messages.

3. **Host waits for the joiner before leaving the lobby.** The host stays on the lobby
   showing the code until the relay confirms the joiner is present. Only then do both
   sides navigate to /setup/ simultaneously.

4. **Setup is visible to both players; only the host controls it.** The joiner sees
   "waiting for host to configure the game" while the host picks draft mode. A single
   explicit wire message (`game-config` or re-use `session-hello`) carries the config
   to the joiner and drives navigation to /draft/ or /match/.

5. **Pong-age-out has a recovery path.** A pong arriving after an age-out should
   restore `mpState.status = "connected"` if the WS is still open (i.e. it was a false
   positive from JS suspension, not a real drop).

6. **Connection/disconnection callbacks are explicit, not reactive.** The mpEngine is
   notified of connect/disconnect via direct calls from transport callbacks, not from
   Svelte effects.

### SOLL session setup flow

```
HOST (lobby)                  RELAY                         JOINER (lobby)
  │                             │                             │
  │── {type:"create"} ─────────►│                             │
  │◄─ {type:"created",code} ───│                             │
  │  [lobby shows code, waits]  │                             │
  │                             │                             │
  │                             │◄─── {type:"join",code} ────│
  │◄─ {type:"peer-connected"} ─│──── {type:"joined"} ───────►│
  │                             │                             │
  │  [transport.onOpen fires]   │  [transport.onOpen fires]   │
  │  → onOpen callback fires    │  → onOpen callback fires    │
  │  → lobby.onPeerConnected()  │  → lobby.onPeerConnected()  │
  │  [goto("../setup/")]        │  [goto("../setup/")]        │
  │                             │                             │
  │  [/setup/ mounts]           │  [/setup/ mounts]           │
  │  [host sees config UI]      │  [joiner sees "waiting..."] │
  │  [host picks: preMade/custom]│                            │
  │  [host presses "Start game"]│                             │
  │── game-config ────────────────────────────────────────►│
  │  {kind:"game-config",mode:"preMade"|"custom",matchId}    │
  │                             │                             │
  │  [both: goto("../draft/")   │  [joiner: goto("../draft/") │
  │         or "../match/"]     │          or "../match/"]    │
  │                             │                             │
  │  [/match/ mounts]           │  [/match/ mounts]           │
  │  [mpEngine created, role=host│  [mpEngine created, role=joiner]
  │  [onOpen callback → notifyConnectionOpen()]               │
  │── session-hello ──────────────────────────────────────►│
  │  {kind:"session-hello",matchId,phase:"play",seq:0}       │
```

Key changes vs IST:
- Host stays on lobby until `peer-connected` fires. Both sides navigate together.
- No `$effect` triggers navigation. `onOpen` callback in the lobby calls `goto` directly.
- /setup/ is a shared screen. Host configures, joiner waits for `game-config` wire message.
- `session-hello` now only carries match state (matchId, phase, seq), not navigation intent.

### SOLL reconnect flow (mid-match drop)

```
HOST (/match/)                RELAY                         JOINER (/match/)
  │                             │                             │
  │◄─ {type:"peer-disconnected"}│                             │  (joiner WS dropped)
  │  [transport.onClose callback fires directly]              │
  │  [mpEngine.notifyConnectionLost()]                        │
  │  [host engine: paused=true, sends "paused" best-effort]   │
  │                             │                             │
  │                             │  [auto-redial fires ~400ms] │
  │                             │◄── {type:"join",code} ─────│
  │◄─ {type:"peer-connected"} ─│──── {type:"joined"} ───────►│
  │  [transport.onOpen callback fires directly]               │
  │  [mpEngine.notifyConnectionOpen()]                        │
  │── session-hello ──────────────────────────────────────►│
  │◄── request-snapshot ─────────────────────────────────────│
  │── snapshot ───────────────────────────────────────────►│
  │  [joiner engine resynced]   │  [joiner: onSnapshotApplied]│
  │  [both: playing again]                                   │
```

"Transport.onOpen callback fires directly" means the callback registered with the transport
calls `notifyConnectionOpen()` explicitly, without going through a Svelte reactive effect.

### SOLL pong-age-out recovery

```
t=15  Ping sent.
      --- JS throttled for 18s ---
t=33  Device resumes. Tick fires: age = 18s > 15s → age-out would fire.
      BUT: before flipping status, send an urgent ping.
      Set a "confirming-dead" flag. Do NOT flip status yet.
t=33  Urgent ping arrives at relay immediately (WS is alive).
t=33  Pong arrives back. lastPongAt = t=33.
      "confirming-dead" flag cleared.
      Status never flipped. No disruption.

--- OR, if WS is actually dead: ---

t=33  Urgent ping times out (no pong in 3s). "confirming-dead" expires.
      NOW flip status = "disconnected".
      This is a real drop — transport's onClose fires separately or
      the relay's TCP keepalive kills the connection promptly.
```

Alternatively (simpler): just treat a pong arriving when status is "disconnected"
but `isActive()` is true as a recovery signal:

```ts
if (msg.kind === "pong") {
  mpState.lastPongAt = Date.now();
  if (mpState.status === "disconnected" && isActive()) {
    // False positive age-out: WS is alive, pong just arrived late.
    mpState.status = "connected";
    mpState.disconnectedSince = null;
    heartbeat.startPings();
    mpEngine?.notifyConnectionOpen();   // ← re-announces session to peer
  }
}
```

---

## Part 3: Delta Table — IST vs SOLL

| # | Area | IST | SOLL | Gap |
|---|------|-----|------|-----|
| 1 | Relay pairing signals | `peer-connected` / `joined` are transport internals | Both exposed as explicit callbacks to application layer | Same behavior, but IST doesn't use them directly for navigation |
| 2 | Host navigation trigger | `$effect` on `mpState.status === "connected"` | Direct `goto()` in `onOpen` transport callback | ✗ IST uses fragile reactive trigger |
| 3 | Joiner navigation trigger | Waits for `session-hello` from host app layer | `onOpen` callback + `game-config` message from host | ✗ IST has joiner depend on host being in correct route |
| 4 | Host waits for joiner | Host navigates to /setup/ immediately after pairing | Host stays on lobby until pairing, then both navigate | ✗ IST separates host and joiner navigation |
| 5 | /setup/ visibility | Only host sees /setup/; joiner waits on lobby | Both on /setup/; joiner sees waiting state | ✗ IST has joiner stuck on lobby with no feedback |
| 6 | Protocol event dispatch | `$effect` triggers `notifyConnectionOpen/Lost` | Direct callback chain from `onOpen`/`onClose` | ✗ IST uses reactive effects for protocol steps |
| 7 | Pong-age-out | Flips status to "disconnected"; no recovery path | Recovery on late pong if WS is still open | ✗ IST bug: permanently pauses game on JS suspension |
| 8 | Ping after age-out | Pings stop when age-out fires | Ping restarts on pong recovery | ✗ IST secondary bug |
| 9 | `session-hello` role | Carries both navigation intent AND match state | Only carries match state (matchId, phase, seq) | ✗ IST conflates navigation + state sync |
| 10 | Mid-match reconnect | `$effect` → `notifyConnectionOpen/Lost` | Direct callbacks from transport `onOpen`/`onClose` | ✗ Same fragility as #6 |
| 11 | Relay code | Correct | Same | ✓ OK |
| 12 | Transport code | Correct | Same | ✓ OK |
| 13 | Engine wrapper (createMpEngine) | Correct | Same | ✓ OK — only its callers change |
| 14 | Wire protocol (V2 messages) | Correct | Add `game-config` message | Small extension needed |

---

## Part 4: Required changes, in order

### Step 1: Fix the pong-age-out bug (immediate, isolated)

In `multiplayer.svelte.ts`, the pong handler:

```ts
// Current:
if (msg.kind === "pong") {
  mpState.lastPongAt = Date.now();
  return;
}

// SOLL:
if (msg.kind === "pong") {
  mpState.lastPongAt = Date.now();
  if (mpState.status === "disconnected" && isActive()) {
    mpState.status = "connected";
    mpState.disconnectedSince = null;
    heartbeat.startPings();
    // Notify the engine wrapper directly — no $effect needed.
    // (Requires mpEngine to be accessible here, or a callback to be registered.)
  }
  return;
}
```

Also raise `PILL_DISCONNECTED_MS` from 15s to 30s to reduce false positives under
tab throttling / Tauri webview suspension.

### Step 2: Replace $effect protocol wiring with direct callbacks

In `/match/+page.svelte`, replace:

```ts
// Current — fragile $effect:
mpConnectedUnsub = $effect.root(() => {
  $effect(() => {
    if (mpState.status === "connected") mpEngine?.notifyConnectionOpen();
    else if (mpState.status === "disconnected") mpEngine?.notifyConnectionLost();
  });
});
```

With explicit callbacks registered on the transport:

```ts
// SOLL — register once, fire directly:
const onConnected = () => mpEngine?.notifyConnectionOpen();
const onDisconnected = () => mpEngine?.notifyConnectionLost();
registerMpCallbacks({ onConnected, onDisconnected });
// unregister in onDestroy
```

The wrapper module (`multiplayer.svelte.ts`) exposes `onConnected`/`onDisconnected`
subscription functions — same pattern as `onData`/`onRawData` already use.

### Step 3: Both peers navigate from lobby on pairing signal

In `/multiplayer/+page.svelte`:

```ts
// Current — host navigates via $effect after mpState.status changes:
$effect(() => {
  if (view === "hosting" && mpState.status === "connected") {
    goto("../setup/");
  }
});

// SOLL — register onOpen callback before calling mpHost():
// The transport's onOpen fires when peer-connected arrives.
// Both sides call goto("../setup/") in the same callback.
```

The lobby registers an `onConnected` callback before calling `mpHost()` or `mpJoin()`.
Both host and joiner have: "when pairing confirmed, navigate to /setup/".

### Step 4: Add `game-config` wire message; host controls setup navigation

Host is on /setup/, joiner is on /setup/ (waiting screen). Host picks draft mode and
presses start. Host sends `game-config`. Both peers navigate to /draft/ or /match/.

```ts
// New wire message (add to WireMessageV2 union):
| { kind: "game-config"; mode: "preMade" | "custom"; matchId: string }
```

/setup/ registers an `onRawData` listener for `game-config`. Host sends it on confirm;
joiner receives it and navigates. Host navigates immediately on send.

This replaces the current flow where `session-hello` is what drives joiner navigation
from the lobby, and the joiner has no feedback during /setup/.

---

## Summary

The core issue is architectural: the current code uses Svelte's reactive `$effect` to
drive network protocol events, and uses the application-layer `session-hello` message
to drive route navigation. Both are the wrong tool for the job.

The immediate fix (step 1 above) addresses the most common real-world symptom: a
pong-age-out caused by JS timer throttling that has no recovery path. This can be
shipped now as a one-file patch.

The structural fixes (steps 2-4) clean up the architecture to make the system
understandable and testable. They touch more files but are not blocked on each other
and can be done incrementally.
