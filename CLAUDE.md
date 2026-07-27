# Board Game Project - Orientation

A 2-player abstract-tactical board game. 2 players, 8x8 grid, no luck, perfect information. Win by capturing the enemy King. The core fantasy: discovering and executing clever skill combos.

## Where things live

```
board-game-project/
├── design/
│   ├── RULES.md              <- canonical ruleset, always authoritative
│   └── knowledge/            <- design history as JSON
│       ├── principles.json   <- hard constraints + north-star principles
│       ├── open_questions.json
│       ├── backpocket.json   <- candidate fixes/ideas (fixes + trigger_cond fields)
│       ├── mechanics.json    <- all mechanics ever considered, with verdicts
│       ├── playtests.json    <- playtest summaries
│       └── playtests/        <- verbatim transcripts + game logs per playtest
├── game/
│   ├── crates/core_engine/   <- Rust: all game logic
│   ├── crates/tauri_wrapper/ <- desktop app (Tauri 2)
│   ├── crates/nn_trainer/    <- neural net self-play training
│   ├── crates/search_bench/  <- benchmarking harness
│   ├── frontend/             <- SvelteKit + TypeScript UI
│   ├── relay/                <- WebSocket multiplayer relay (Fly.io)
│   ├── bench/                <- benchmark baselines + corpus + run scripts
│   ├── plans/                <- active engineering plan docs
│   └── tools/                <- helper scripts
└── .github/workflows/release.yml  <- builds on v* tag push
```

## Rules

`design/RULES.md` is the single source of truth. Read it before touching any game logic or design question. `core_engine` implements exactly this ruleset.

Current ruleset in brief:
- 8x8 grid. 1 King + 5 Champions + 6 Guards per player.
- Champions and King have 2 equip slots. Guards have none.
- Turn: Move Phase (2 actions, move or Move-Attack) then Skill Phase (actions grow with Progression).
- All pieces have 2 HP and up to 2 Armor. Armor absorbs damage first.
- Bodyguard rule: a Guard can intercept Move-Attacks targeting Champions/King.
- Multi-Champion Combo Bonus: chaining hits from different Champions on the same target deals bonus damage.
- Money income and Skill Phase actions both scale over rounds (see Progression table in RULES.md).

## Design knowledge

`design/knowledge/` is a snapshot of the full design history. Most useful files:
- `principles.json` - what the game must and must not be
- `open_questions.json` - unresolved design questions with priority
- `backpocket.json` - candidate fixes, each with `fixes` (the problem) and `trigger_cond` (when to look at it)
- `mechanics.json` - every mechanic tried, with verdict and reasoning

## Building and running

```bash
# Check the Rust workspace compiles
cd game && cargo check

# Run the desktop app (dev mode)
cd game/crates/tauri_wrapper && cargo tauri dev

# Frontend dev server only (browser, no Rust engine)
cd game/frontend && npm install && npm run dev

# Run the search benchmark
cd game && cargo build --release -p search_bench
./target/release/search_bench --help

# Analyse a playtest game log
python3 game/tools/analyze_playtest.py <path-to-bundle.json>
```

## Versioning

`v<major>.<minor>.<patch>` - the version tracks the active rule state. `v0.1.0` = first digital playtest ruleset. Confirmed change -> bump version. Rejected change -> revert code and rules, bump version anyway.

## Git

All work on `main`. No branches. Tag releases to trigger GitHub Actions:

```bash
git push origin main
git tag v0.1.1
git push origin v0.1.1
```

**Release tagging rules (MUST follow):**
- The canonical version lives in `game/Cargo.toml` (`version = "x.y.z"`). ALWAYS read that file to get the version — never invent or increment a version number yourself.
- Use exactly `v{version}` from that file as the tag. No guessing, no bumping.
- If a GitHub release or tag already exists for that version, ask before deleting it.
