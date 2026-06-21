# game/

The digital implementation of (GAME NAME). This is the **target**: a complete digital version of the board game with Stack M rules as the default — not a "push-things-around simulator."

## Status

**Empty.** Architecture not yet chosen. No code written.

## Decisions still pending

The architecture ADR (Rust core + multi-platform frontend) has not been written. Open questions:

- **Frontend split.** Desktop (egui? Tauri? GTK?) · Web (Yew? Leptos? WASM-bindgen?) · Mobile (Bevy? native bindings?).
- **Multiplayer transport.** Self-hosted server vs P2P (relay-fallback) vs local-only first.
- **AI opponent.** Search-based (minimax/MCTS — game is perfect-info, no luck) vs learned (NN) vs hybrid.
- **Save format.** Per-move log + state snapshot, exportable as JSON / PDF (P5 lost its log to a browser refresh — this must not happen again, see `next_steps` priority 6 in DB).

These get resolved in the next session via an architecture ADR before any code.

## Rule source of truth

**The rules are in the database**, not in this folder. Specifically:

```bash
sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"
```

That body is the full rule substance for Stack M (board, setup, components, all 7 systems, all 15 skills with costs/effects, Quick Reference, what's bundled, hypothesis, watch list, routing). When the Rust core ships, it implements that ruleset.

For canonical numbers that haven't changed under Stack M, read `essay-game-economy-map` (the 12-economy analysis) and the baseline mechanics rows:

```bash
sqlite3 design/design.db "SELECT id, name, body FROM mechanics WHERE verdict='baseline';"
```

## Why "digital-first" now

Three drivers:

1. **Iteration speed.** Paper playtests need 2-3 hour blocks with another human; digital lets variants run faster (designer-vs-AI smoke tests at any time of day, multiplayer with Jonathan async).
2. **P5 surfaced felt-PI problems (OQ-64).** A digital UI can show counters, ranges, and threat zones at a glance — surfacing the formal-PI information that paper hides behind cognitive load.
3. **Stack M is bundled and load-bearing.** Six simultaneous changes are hard to track on paper; digital makes the bundle observable and the rollback granular.

## What this folder is **not**

- Not a rumschieb-simulator. A digital board with drag-and-drop pieces is the *first 10%*, not the deliverable.
- Not a place for rules. Rules live in the DB (and re-derive into a printable PDF if needed via `archive/paper-pipeline/test-scenarios/baseline/`).
- Not a place for design discussion. Use `design/inbox/` for that.
