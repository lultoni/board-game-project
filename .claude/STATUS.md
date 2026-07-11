# STATUS

*One-screen re-entry doc. Read first after a gap. Regenerated from the DB at session end.*

*Last updated: 2026-07-11 — Session 45 end (Stack N staging + repo/DB cleanup + design/RULES.md).*

---

## Current focus

**Stack N is staged; the next gate is P7 (playtest it).** Engine implementation (ns-42) blocks the playtest. This session was a design pass (diagnose + stage the standoff fix) plus a structural repo cleanup — no game code changed yet.

## Current ruleset

**`design/RULES.md`** is the canonical ruleset (authoritative on conflict; Help page is the derived player summary). It is Stack M + three **⧗ Stack N — staged, awaiting P7** rules: Focus cost 2, max 1 move-attack/turn, strike-moves-caster. Those three are NOT yet in the engine.

## Parked levers under watch

`sqlite3 design/design.db "SELECT id, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked';"` — 3 live: forward-Guard skill-immunity (Stack N reserve), dual-counter combo (oq-58), piece-count reduction (oq-27/66). *(The "one stack per test" methodology is retired — 16 stack rows frozen for provenance; new candidates park in backpocket via `/scenario`.)*

## What changed this session (45)

- **Stack N staged** (`stacks.body` for `stack-n`): Focus 1→2, 1 move-attack/turn, strike-moves-caster. Root cause: ranged kills are non-reciprocal (no positional cost). Reserve: `bp-forward-guard-partial-skill-immunity`.
- **`design/RULES.md` created** as canonical rules; Typst paper-pipeline archived → `design/_archive/`; 3 inboxes collapsed → `design/inbox/`; stacks methodology retired (`/scenario` now writes backpocket).
- **6 residual plan files deleted**; skills rewritten to query DB/RULES.md. Committed `be2affe` (+456/−1803).
- **New todos:** ns-42 (engine implement Stack N — P1, P7-blocking), ns-43 (phase-aware evaluator — P2).

## Immediate next action

1. **ns-42** — implement Stack N's 3 rules in `game/crates/core_engine` (Focus cost 2, 1 move-attack/turn cap, strike-moves-caster) with regression tests. This unblocks P7.
2. Then **P7** — playtest Stack N (does the standoff dissolve without re-inflating game length?).
3. **ns-43** (phase-aware evaluator) — independent of P7; can run in parallel.
4. **`main` is unpushed** (7 commits incl. this session) — pushing needs explicit designer OK.

## Live critical / high-priority open questions

`sqlite3 design/design.db "SELECT id, title, priority FROM open_questions WHERE status IN ('critical','high') ORDER BY priority, id;"` — 8 critical, 10 high. oq-58 (standoff) updated this session with the non-reciprocity diagnosis + Stack N pointer; others untouched.

## Open loose ends

- **`main` ahead of `origin/main`** — unpushed (needs designer approval).
- v0.1.0 cross-platform release smoke test still outstanding (ns-28/ns-29).
- ns-39 (AI think-time default value — awaiting designer), ns-40 (bodyguard visual indicator), ns-41 (rendering decouple + research) — deferred.

## DB sanity

Session 45 row inserted; `created_in` stamped on all new rows. `integrity_check` → ok, `foreign_key_check` → clean.
