# game/

The digital implementation. Rust workspace (4 crates) + SvelteKit frontend + multiplayer relay.

## Structure

```
game/
├── Cargo.toml                  ← Rust workspace
├── crates/
│   ├── core_engine/            ← pure Rust game engine
│   │   └── src/
│   │       ├── state/          ← bitboards, mailbox, position, zobrist
│   │       ├── game_logic/     ← action, move generator, make/unmake, turns
│   │       ├── search/         ← alpha-beta, transposition table, evaluator
│   │       ├── session.rs      ← match manager + action history
│   │       └── telemetry.rs    ← match logs + export
│   ├── tauri_wrapper/          ← Tauri 2 desktop wrapper
│   ├── nn_trainer/             ← neural net self-play training loop
│   └── search_bench/           ← search benchmarking harness
├── frontend/                   ← SvelteKit + TypeScript UI
│   └── src/
│       ├── lib/
│       │   ├── engine/         ← engine bridge (Tauri IPC + web fallback)
│       │   ├── match/          ← match state, board rendering
│       │   ├── multiplayer*    ← WS relay protocol + session management
│       │   └── training/       ← training observatory UI
│       └── routes/             ← pages: draft, match, multiplayer, replay, loadouts, training...
├── relay/                      ← WebSocket relay server (Node, deployed on Fly.io)
├── bench/                      ← search benchmark baselines, corpus, run scripts
├── plans/                      ← active engineering plan docs
├── runs/                       ← training run outputs (active/ gitignored)
└── tools/                      ← helper scripts (playtest log analysis etc.)
```

## Build

```bash
# Check everything compiles
cd game && cargo check

# Desktop app - dev mode (hot reload)
cd game/crates/tauri_wrapper && cargo tauri dev

# Frontend dev server only (browser, no Rust)
cd game/frontend && npm install && npm run dev

# Production desktop build
cd game/crates/tauri_wrapper && cargo tauri build

# Static web build
cd game/frontend && npm run build
```

### NN trainer backend options

| Cargo feature | Backend | Platforms |
|---|---|---|
| `backend-ndarray` | CPU (always available) | All |
| `backend-wgpu` | GPU via wgpu | Metal (Mac), Vulkan, DX12 |
| `backend-cuda` | NVIDIA CUDA | Linux x86_64, CUDA 12.x |

Default: `backend-ndarray + backend-wgpu`. CUDA build (Linux + NVIDIA only):

```bash
cd game/crates/tauri_wrapper
cargo tauri build --features backend-ndarray,backend-cuda \
                  --config ../tauri.cuda.conf.json
```

Search-time inference (in-game AI) always uses CPU ndarray - GPU dispatch overhead dominates at batch size 1.

## Rules

The engine implements the ruleset in [`design/RULES.md`](../design/RULES.md). That file is authoritative. When the ruleset changes, the engine changes with it.

## Training Observatory

Open the desktop app and navigate to `/training`:

- **Live Match View** - board + eval bars updated per ply
- **Tournament Standings** - W/L/D per population member
- **Lineage Tree** - every accepted rater (click to inspect)
- **Network Inspector** - forward output + per-layer weight stats
- **Gauntlet Matrix** - N×N win-rate heatmap per bracket

Training run outputs are persisted under `game/runs/active/` (gitignored). Copy a run to `game/runs/archive/<run-id>/` to keep it.

## Benchmarking

```bash
cd game && cargo build --release -p search_bench
./target/release/search_bench --help

# Compare a result against a baseline
python3 bench/compare.py bench/baseline.json bench/results/my-run.json
```

## Release

Releases are triggered by pushing a `v*` tag - GitHub Actions builds binaries for macOS and Linux and attaches them to the release. See `../.github/workflows/release.yml`.

```bash
git push origin main
git tag v0.1.1
git push origin v0.1.1
```
