# (GAME NAME) — Board Game Design Project

A 2-player abstract-tactical board game in active development. Players command armies of Guards and Champions led by a King on a grid, equipping Champions with skills and spending Money to activate them. Victory by capturing the enemy King.

**Design identity**: The intersection of chess-like spatial tactics and CCG-style build customisation. The core fantasy is discovering and executing clever skill combos. Everything else is chassis.

As of Session 27 (2026-06-22) the project is **digital-first**: a complete digital implementation in `game/` is the deliverable, with Stack M rules as the default. The paper pipeline is archived.

---

## Download

Pre-built desktop binaries are attached to each tagged release on the
[Releases page](../../releases):

| Platform                 | Artefact                  | Backends bundled       |
|--------------------------|---------------------------|------------------------|
| macOS (Apple Silicon)    | `.dmg`                    | CPU + GPU (Metal)      |
| Linux x86_64             | `.AppImage`               | CPU + GPU (Vulkan)     |
| Linux x86_64 + NVIDIA    | `.AppImage` (CUDA build)  | CPU + CUDA             |

The macOS build is ad-hoc signed — on first launch, right-click the app
→ **Open** to bypass Gatekeeper. Linux AppImages need `chmod +x` after
download.

Once running, open the Training Observatory and pick a backend from the
dropdown.

To build from source, see [`game/README.md`](game/README.md).

---

## Quick Navigation

The source of truth for all design knowledge is `design/design.db` (SQLite). Query it directly:

| I want to… | Query / file |
|---|---|
| Re-enter the project after a gap | [`.claude/STATUS.md`](.claude/STATUS.md) |
| Read the active stack's full rules (Stack M) | `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"` |
| See critical / high open questions | `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority;"` |
| See what to do next | `sqlite3 design/design.db "SELECT priority, title FROM next_steps WHERE status='todo' ORDER BY priority;"` |
| Read the most recent session narrative | `sqlite3 design/design.db "SELECT body FROM sessions ORDER BY n DESC LIMIT 1;"` |
| Read the current canonical ruleset | [`design/RULES.md`](design/RULES.md) |
| See historical test stacks (frozen for provenance) | `sqlite3 design/design.db "SELECT id, letter, name, status FROM stacks ORDER BY letter;"` |
| See parked design levers (if problem X → candidate Y) | `sqlite3 design/design.db "SELECT id, name, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked';"` |
| Read a specific essay / research artefact | `sqlite3 design/design.db "SELECT body FROM essays WHERE id='essay-<slug>';"` |
| See cross-references for any row | `sqlite3 design/design.db "SELECT to_id, relation FROM links WHERE from_id='<id>';"` |
| Read all architecture decisions | `sqlite3 design/design.db "SELECT body FROM adrs ORDER BY n;"` |
| Read the paper-era rule sheets (historical) | [`design/_archive/test-scenarios/`](design/_archive/test-scenarios/) |

More query patterns in [`CLAUDE.md`](CLAUDE.md).

---

## Project Structure

```
board-game-project/
│
├── README.md                    ← you are here
├── CLAUDE.md                    ← project conventions for Claude Code
│
├── design/
│   ├── design.db                ← SQLite source of truth (12 tables)
│   ├── schema.sql               ← table definitions, CHECK constraints, FKs
│   ├── README.md                ← DB usage notes
│   ├── RULES.md                 ← canonical current ruleset (authoritative on conflict)
│   ├── raw/                     ← binary artefacts (photos, scans, card images)
│   │   ├── playtest-photos/
│   │   ├── brainstorm-scans/
│   │   └── skill-card-images/
│   ├── inbox/                   ← fast-write staging (single folder) — promoted into the DB
│   │                              brainstorm-* · chat-* · digital-* · playtest-*-notes.md
│   └── _archive/                ← frozen Typst rule sheets + PDFs (paper-prototype era; historical)
│
├── game/                        ← digital implementation (Rust core + Tauri frontend)
│
└── .claude/
    ├── STATUS.md                ← one-screen re-entry doc (regenerated each session)
    ├── HANDOVER.md              ← session-to-session continuity prompt
    └── skills/                  ← slash-command definitions
```

---

## The Game in 2 Minutes

**Players**: 2 · **Board**: 8×8 grid (Stack M default), no terrain · **Win condition**: capture the enemy King.

### Pieces (per player)
| Piece | Count | Speed | Skills |
|---|---|---|---|
| King | 1 | 1 | 2 slots |
| Champion | 5 | 1 | 2 slots |
| Guard | 6 | 2 | — |

### How a turn works
1. **Move Phase** — move up to 2 pieces (each once).
2. **Skill Phase** — activate up to N skills (limited by actions, paid in Money).

### Key systems
- **Money** — currency for activating skills. Scales over rounds.
- **Skills** — equipped during pre-game draft. 2 slots per Champion/King. Line-of-sight paths blocked by all pieces.
- **2 HP** — Normal → Removed (Stack M: injured penalties removed; HP tracker only).
- **Bodyguard** — Guard adjacent to an attacked Champion/King can intercept move-attacks.
- **Move-Attack** — move onto enemy tile = 1 damage.

Full Stack M rules: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`

---

## Current Status

See [`.claude/STATUS.md`](.claude/STATUS.md) for the one-screen re-entry doc. The next anchor is the architecture ADR for `game/` (Rust core + multi-platform frontend).

---

## Working with Claude Code

### Starting a session
Run `/start` — it pulls remote, reads STATUS + HANDOVER, queries the DB for live state, and presents a briefing.

### Ending a session
Run `/wrapup` — it persists DB changes, runs integrity checks, regenerates STATUS.md and HANDOVER.md, commits and pushes.

### Custom skills

| Command | When to use |
|---|---|
| `/start` | Begin a design session |
| `/wrapup` | End a session — DB writes, commit, push |
| `/research <topic>` | Need external knowledge about game design |
| `/adr <topic>` | Multiple valid design approaches need comparison |
| `/scenario <desc>` | Discussion yields a testable change → park it as a backpocket lever |
| `/playtest <N>` | Analyse playtest results (paper photos or digital log) |

### Dropping notes between sessions

Everything goes into the single `design/inbox/` folder; name the file by kind:
- **Game-design ideas** (mechanics, skills, board) → `brainstorm-*.md`
- **Pasted AI chats** (ChatGPT, Perplexity, Gemini) → `chat-*.md`
- **Digital-implementation thinking** (architecture, UI, AI opponent, save format) → `digital-*.md`
- **Playtest feedback** prose → `playtest-<name>-notes.md` (keep photos/logs in `design/raw/playtest-photos/`)

I'll mine these into the DB (backpocket / essays / open_questions / adrs / next_steps) at session start.

---

## Historical archive

`design/_archive/` — Typst rule sheets and PDFs from the paper-prototype era; read-only history (the canonical rules now live in `design/RULES.md`). Earlier prototype versions (v1–v3) were removed in S38 cleanup; recoverable from git history if needed.
