# game/

The digital implementation of (GAME NAME). Architecture per **ADR-005** (`SELECT body FROM adrs WHERE id='adr-005';`).

## Structure

```
game/
├── Cargo.toml                  ← Rust workspace
├── crates/
│   ├── core_engine/            ← Layers 1–5: pure Rust game engine
│   │   └── src/
│   │       ├── state/          ← Layer 1: bitboards, mailbox, position, zobrist
│   │       ├── game_logic/     ← Layer 2: action, magic, generator, make/unmake, turns
│   │       ├── search/         ← Layer 3: alpha-beta, transposition, evaluator
│   │       ├── session.rs      ← Layer 4: match manager + action history
│   │       └── telemetry.rs    ← Layer 5: match logs + export
│   ├── wasm_wrapper/           ← wasm-bindgen surface for the web build
│   └── tauri_wrapper/          ← Tauri 2 desktop wrapper (native engine)
└── frontend/                   ← Layer 6: Svelte 5 + TypeScript (one UI, two targets)
    └── src/
        ├── App.svelte
        ├── lib/
        │   ├── Board.svelte    ← 8×8 CSS Grid, PointerEvents
        │   ├── engine.ts       ← runtime-agnostic engine bridge (WASM vs Tauri IPC)
        │   └── multiplayer.ts  ← Layer 7: PeerJS + WebRTC + commit-reveal
        └── main.ts
```

## Status

**Scaffolded (Session 28, 2026-06-22).** Module skeletons + types + bit-pack helpers + transposition-table shell are in place. Nothing is wired end-to-end yet — every `TODO` marks an implementation step.

## Build (dev)

```bash
# Rust core check (no implementation yet — just the scaffold compiles)
cd game && cargo check

# Frontend dev server
cd game/frontend && npm install && npm run dev

# Web build (static, deployable to GitHub Pages)
cd game/frontend && npm run build

# Desktop dev (Tauri 2 — requires Rust + platform deps)
cd game/crates/tauri_wrapper && cargo tauri dev    # once @tauri-apps/cli is installed
```

## Rules source of truth

The rules are in the database, not in this folder:

```bash
sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"
```

`core_engine` implements that ruleset. When Stack M changes (or 6×8 follow-on lands), the engine changes with it.

## Training Observatory

The NN-rater training loop (Step 7 of the NN-rater plan) is wired
through a dedicated UI at **`/training`** in the desktop app. Open
`cargo tauri dev` from `crates/tauri_wrapper`, then navigate to
`/training` — the route renders:

- **Live Match View** — board + three centipawn eval bars updated per ply
- **Tournament Standings** — table of every population member, W-L-D, win rate
- **Lineage Tree** — every accepted rater (click one to populate the Inspector)
- **Network Inspector** — forward output + per-layer weight stats for the selected rater
- **Gauntlet Matrix** — N×N win-rate heat-map per bracket

Status / live-position / index / matrix snapshots are persisted under
**`game/runs/active/`** (gitignored — large, churny). Promote an
interesting run by copying it to `game/runs/archive/<run-id>/`.

