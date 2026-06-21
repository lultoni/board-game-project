# Digital Prototype — Intake

*One-file dump for everything related to building a full digital implementation of the game. Started Session 27 (2026-06-21). Stack M rules are the intended default ruleset.*

*This is a working intake — raw input lives here. When sections crystallise into decisions, promote them to proper docs (ADR, spec, etc.) and link back.*

---

## Context (why this file exists)

Decision made during the holiday between Session 26 and Session 27: a "rumschieb simulator" (drag-and-drop only) is not enough. The target is a **complete digital implementation** of the game — rules-enforcing, with Stack M as the default rule set.

This is a fundamental project pivot:
- The existing paper-pipeline (Typst stacks, printed feedback forms, skill cards) becomes secondary. Stack M is the last paper-print target unless explicitly revived.
- The existing `prototype/` folder (single-file Pole-B web prototype) is dead and will be deleted as part of the restructure.
- The digital prototype becomes the primary testing surface AND the long-term product direction (multiplayer, AI opponent, multi-platform frontend).

Pre-existing related entries in `docs/backpocket.md`:
- *Digital Prototype Persistence — Tooling Requirement* (Session 25) — auto-save, export, per-turn log. Constraint, not design.
- *Digital Playtest Prototype* (Session 15, `[TO DISCUSS]`) — old framing as a minimal drag-and-drop tool. **This intake supersedes that framing.**
- *Spec the game for a programmer* — write-only exercise that doubles as the rules-engine foundation. Still relevant; would feed this build.

---

## Session 27 (2026-06-21) — Designer's raw direction-setting

*Verbatim notes from the user, captured at the start of Session 27. Source material for the planning pass.*

### 1. Pivot the repo to digital-first

> "das boardgame repo anpassen, sodass es auf das neue testen und spielen und so passt (weil ich nur noch digital spielen will dann)"

Repo restructure required. The current shape (Typst-heavy `docs/test-scenarios/`, paper feedback forms, photo-based `playtest-results/`) assumes paper as the primary surface. That assumption is dropped. Digital becomes the testing surface; paper artifacts move to archive.

### 2. Rust implementation, cleanly structured in the repo

> "also dass die rust implementation dann auch da strukturell clean drin ist"

**Stack decision**: Rust for the backend / rules-engine / core. Frontend separate; see point 6 below for multi-platform target.

Code does not yet exist. The plan + system structure get delivered through this INTAKE doc first, then revised, then built.

### 3. Structured/queryable storage for docs and decisions

> "die aktuell verhandenen docs und design entscheidungen und alles sollen dann auch mal overhauled werden, sodass sie besser searchable und standardisiert sind mit iwwie json oder sqlite oder sowas gespeichert sind, sodass alles cleaner ist und wir leichter das skalieren können"

The current Markdown-as-database approach (OPEN_QUESTIONS.md, mechanics-evaluated.md, SESSION_LOG.md, backpocket.md) is hitting scale limits. Lots of cross-references, lots of duplicate state, hygiene-rework needed periodically (cf. Session 17 rework). Moving to structured storage (JSON / SQLite / similar) so:
- entries are queryable by status, system, OQ-ID, stack-ID
- cross-references are real links, not hand-typed strings
- updating one fact updates everywhere it shows
- scaling to more decisions, more playtests, more skills is sustainable

### 4. Cross-linked information access

> "das repo umbauen mit db's oder kanbans oder so, sodass es easy wird infos mit ihren related sachen systematisch zu callen an infos"

When viewing an OQ, see its connected stacks, mechanics-log entries, playtests, and related OQs without grep. When viewing a stack, see which OQs it addresses and which playtests evaluated it. When viewing a playtest, see which stacks/OQs were under test.

This is graph-shaped data, currently encoded as Markdown links. Wants a real model.

### 5. Existing prototype is dead

> "generell soll alles abgestimmt werden auf den neuen digitalen prototypen (und der alte soll gelöscht werden)"

`prototype/index.html` (Pole-B web prototype from P5) gets deleted as part of the restructure. Insights from P5 stay archived in `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/`.

### 6. Frontend = multi-platform (Desktop / Web / Mobile)

> "das frontend der anwendung für mehrere anwednungne (desktop, web, mobile) gebaut werden soll, weswegen das bauen vom digitalen prototypen auch wichtig sein wird"

The frontend isn't a single-platform tool. Targets:
- **Desktop** (Mac primary, likely Win/Linux follow)
- **Web** (browser-playable, hot-seat at minimum)
- **Mobile** (iOS / Android — touch-first UI)

This is a hard architecture constraint: rules engine must be portable, frontend must be cross-platform-capable. Rules out anything Rust-frontend-monolithic; pulls toward "Rust core as library/server + frontend stack that compiles to all three" (Tauri, Flutter, React Native + Rust core via WASM/FFI, etc.). To be planned — flagged as the *biggest architecture decision* and probably the first ADR.

### 7. Long-term feature scope

> "der digitale prototyp wird dann auch multiplayer und einen ai gegner und alles bekommen wird"

Not MVP, but they shape architecture:
- **Multiplayer** — networked play, requires authoritative server-side rules + state sync. Cannot be retrofitted into a client-only design.
- **AI opponent** — needs an interface the engine can drive (not just human input). Pulls rules engine + game state into a form suitable for search/eval.

### 8. Cross-cutting concerns

> "natürlich steht dann telemetrie und feedback und player onboarding und co weiterhin im fokus"

These don't go away when paper does — they become digital-native:
- **Telemetry** — per-turn state, picks, timings; analysis-grade data automatically (replaces handwritten game-tracking sheets).
- **Feedback** — in-app feedback forms tied to game sessions (replaces paper feedback PDFs).
- **Player onboarding** — interactive tutorial, pre-made loadouts (OQ-65), guided first game. The thing paper rule sheets currently do, but better.

---

## Open questions surfaced from the direction-setting

*Each will likely become its own ADR or planning artefact. Listed for visibility, not yet answered.*

1. **What's the architecture for "Rust core + multi-platform frontend"?** Tauri vs. Flutter+FFI vs. React Native+WASM vs. Bevy vs. separate-frontend-per-platform. Biggest single decision.
2. **What's the structured-storage shape?** SQLite (relational, queryable via SQL) vs. JSON-on-disk + a tool layer vs. a document DB. What lives in DB vs. what stays as Markdown narrative.
3. **What's the schema for the design knowledge?** OQ, Stack, Mechanic, Playtest, Decision, Principle, Backpocket-entry — each as a record with which fields and which links?
4. **What's the migration path?** Hand-port everything once, or build a Markdown-to-DB importer, or run hybrid for a transition window?
5. **What's the source of truth for game rules?** Currently `baseline-sections.typ` + Stack overrides. Going forward: Rust code? A rules-data file (JSON/RON/TOML) that both the engine reads and the Typst rendering reads? Spec doc?
6. **How does this coexist with Stack M's pending paper playtest?** Print + run P6 before deleting paper pipeline, or skip P6 and absorb the changes into the digital prototype directly?
7. **What's the minimum viable digital prototype?** First playable version definition. Likely: hot-seat, Stack M rules, on one device, with auto-save and per-turn log. Multiplayer/AI/onboarding come later.
8. **Where does Phase B (theme, naming, visual identity) sit in this?** Digital frontend will need visual identity earlier than paper did. Pulled forward?
9. **How do telemetry/feedback flow back into the design loop?** What's the digital equivalent of `/playtest <N>` skill — auto-aggregate session data → insights doc?
10. **What's the dev workflow for the designer (you) once Rust exists?** You're not a Rust dev today — how much of the build is you, how much is me, how much gets contracted/scoped?

---

## Proposed next-session shape

Session 28 (or this session continued) should produce:
- An **ADR** on architecture (Rust core + frontend stack choice) — the biggest blocker.
- A **second ADR** or planning doc on the storage layer (DB shape + what migrates from MD).
- A **target repo layout** sketch — what folders exist, what's in each, what gets deleted.
- A **migration plan** — order of operations from current state to target state.

No code yet. No deletions yet. Plan first, review, then move.

---

## AI chat dumps

*Paste full conversations from other AIs here. Mark each with date + which AI + what you were exploring. Don't edit them — raw is fine; we'll extract decisions later.*

### [Template — copy for each dump]

**Date**:
**AI / source**:
**Topic / what I was after**:

```
(paste transcript here)
```

**My takeaway / what to do with this**:

---

## Free-form notes

*Anything that doesn't fit above. Brain-dump zone. Date entries so we can see the trail later.*

###
