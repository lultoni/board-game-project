# (GAME NAME)

A 2-player abstract-tactical board game. No dice, no hidden information. Win by capturing the enemy King.

---

## Playing the game

Download a pre-built release from the [Releases page](../../releases) and run it on your machine.

| Platform | File | Notes |
|---|---|---|
| macOS (Apple Silicon) | `.dmg` | `xattr -cr "/Applications/(game-name).app"` after download |
| Linux x86_64 | `.AppImage` | `chmod +x` after download |
| Linux x86_64 + NVIDIA | `.AppImage` (CUDA build) | Needs CUDA 12.x |

The app includes a built-in rules reference (Help button in the top bar).

---

## Developing

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- Node 20+
- Tauri CLI: `cargo install tauri-cli`

### Run the app locally

```bash
# Desktop app with hot reload
cd game/crates/tauri_wrapper && cargo tauri dev

# Browser only (no Rust engine, useful for UI work)
cd game/frontend && npm install && npm run dev
```

### Build a release binary

```bash
cd game/crates/tauri_wrapper && cargo tauri build
```

Pushing a `v*` tag triggers GitHub Actions and builds + attaches binaries automatically:

```bash
git push origin main
git tag v0.1.2
git push origin v0.1.2
```

See [`game/README.md`](game/README.md) for the full build reference (NN trainer backends, benchmarking, scripts).

---

## Repo structure

```
board-game-project/
├── design/
│   ├── RULES.md              ← canonical ruleset — always authoritative
│   └── knowledge/            ← design history as JSON (principles, open questions,
│                                mechanics verdicts, playtest records + transcripts)
├── game/
│   ├── crates/
│   │   ├── core_engine/      ← Rust game engine (state, moves, search, eval)
│   │   ├── tauri_wrapper/    ← desktop app (Tauri 2)
│   │   ├── nn_trainer/       ← neural net self-play training
│   │   └── search_bench/     ← search benchmarking
│   ├── frontend/             ← SvelteKit + TypeScript UI
│   ├── relay/                ← WebSocket multiplayer relay (Fly.io)
│   ├── bench/                ← benchmark baselines and run scripts
│   ├── plans/                ← active engineering plan docs
│   └── tools/                ← helper scripts (playtest log analysis etc.)
└── .github/workflows/
    └── release.yml           ← builds + releases on v* tag push
```

---

## Versioning

`v<major>.<minor>.<patch>` tracks the rule state. `v0.1.0` = the ruleset played in the first digital playtest (July 2026). When a rule change is confirmed through playtesting, bump the version. When a change is rejected, revert and re-bump.
