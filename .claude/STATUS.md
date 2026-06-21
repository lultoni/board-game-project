# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-22 — end of Session 27 (cleanup follow-on).*

---

## Current focus

**Digital-first pivot.** The deliverable is a complete digital implementation of (GAME NAME) in `game/` (Rust core + multi-platform frontend), with Stack M rules as default. Paper pipeline is archived. **Architecture ADR is the next anchor** — no code in `game/` until it lands.

## Active stack

**Stack M — Game Length Cut.** Six bundled simultaneous changes vs paper baseline. P6 has not yet run; may be absorbed into the digital prototype rather than a paper run.

Full rule substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`

## What changed in Session 27 (incl. cleanup follow-on)

1. **Repo restructured + DB-first migration.** All design knowledge in `design/design.db` (12 tables, 366 rows). Source `.md` files deleted. `archive/` and `design/raw/` populated.
2. **`game-state/` folded** → `STATUS.md` now at `.claude/STATUS.md`. `digital-prototype/` deleted.
3. **Inboxes operational.** `design/inbox/{brainstorm,ai-chats,digital}/` with READMEs — third channel `digital/` added for `game/` architecture / UI / AI-opponent notes.
4. **All slash-command skills rewritten** to query/write the DB (`/start`, `/wrapup`, `/adr`, `/research`, `/scenario`, `/playtest`). `/build-pdfs` retired.
5. **README.md rewritten** for the new structure.
6. **Migrators archived** to `archive/migrators/` — one-shot artefacts, no longer in working memory.

## Immediate next action

**Architecture ADR for `game/`.** Decide Rust core + frontend split (Desktop / Web / Mobile), multiplayer transport, AI-opponent approach, save format. Insert as `adr-005`. Inputs: Stack M rule substance · OQ-64 felt-PI considerations · P5's lost-game-log lesson (digital persistence is mandatory). Designer can dump prep notes into `design/inbox/digital/` before the ADR conversation.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"`

## Open methodological loose ends

- None outstanding. Migrators archived, skills rewritten, STATUS/HANDOVER paths consistent.

## DB sanity

- 12 tables, 366 rows.
- `PRAGMA foreign_key_check` returns no rows; `PRAGMA integrity_check = ok`.
