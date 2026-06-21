# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-06-22 — end of Session 27.*

---

## Current focus

**Digital-first pivot.** The deliverable is now a complete digital implementation of (GAME NAME) in `game/` (Rust core + multi-platform frontend), with Stack M rules as the default. Paper pipeline is archived. Architecture ADR is the next anchor.

## Active stack

**Stack M — Game Length Cut.** Six bundled simultaneous changes vs paper baseline. P6 has not yet run; may be absorbed into the digital prototype rather than a paper run.

Full rule substance: `sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-m';"`

## What changed in Session 27

1. **Repo restructured.** All design knowledge migrated into `design/design.db` (12 tables, 365 rows). Source `.md` files deleted. `old-game-versions/` → `archive/old-game-versions/`. `docs/test-scenarios/` → `archive/paper-pipeline/test-scenarios/`. Raw assets → `design/raw/`.
2. **`CLAUDE.md` rewritten** as an orientation pointer-document; facts now live in the DB.
3. **Inboxes added** for the designer's brainstorm dumps and AI-chat transcripts: `design/inbox/brainstorm/`, `design/inbox/ai-chats/`. READMEs explain the fast-write → DB-distill flow.
4. **`game/README.md` placeholder** written; architecture pending an ADR.

## Immediate next action

**Architecture ADR for `game/`.** Decide Rust core + frontend split (Desktop / Web / Mobile), multiplayer transport, AI-opponent approach, save format. Insert as `adr-005` once decided.

## Live critical / high-priority open questions

Query: `sqlite3 design/design.db "SELECT id, title FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"`

## Open methodological loose ends

- Slash-command skills (`.claude/skills/*`) reference deleted MD paths and will fail until rewritten.
- One-shot migrators (`.claude/migrate_*.py`) kept for audit; consider moving to `archive/migrators/` once DB is trusted.

## DB sanity

- 12 tables, 365 rows.
- `PRAGMA foreign_key_check` returns no rows; `PRAGMA integrity_check = ok`.
