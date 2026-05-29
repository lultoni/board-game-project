# STATUS

*One-screen re-entry doc. Read this first after a gap. Updated by `/wrapup`.*

*Last updated: 2026-05-29 — Session 22 (close).*

## Current focus

Playtest 4 analysed (Niko (P1, first-time) beat Elias (P2) on 2026-05-28, Stack A G2 + Niko's first game). **Stack H (Armor cap 3→2 + Armorsmith +1→+2) is Priority 1** — confirmed P4 problem, chassis volume crowds out the combo loop. **Stack A G3 (dual-counter combo + widened scope) is Priority 2, gated behind Stack H** per Path A methodology decision (one structural variable per stack).

## Active OQs (top 4)

1. **OQ-11 (confirmed)** — *Stack H — Armor Trim* (bundled cap 3→2 + Armorsmith +1→+2). P4 evidence strongest yet (Elias verbatim "armor was a part of combo calcs but it just felt like you were not able to do your combos because of it"). Needs two-experienced-player run. Smaller dose (cap-only) lives within Stack H as next iteration if bundled stalls.
2. **OQ-58 (new — exchange-pit)** — mid-game collapses into one cluster, pieces taken one-by-one. Watched under Stack H; if persists, *Stack A G3 — Dual-Counter Combo* is the targeted fix.
3. **OQ-38 (reframed)** — combo Q3 softness is design-aligned; lever is **scope, not strength**. *Stack A G3 — Dual-Counter Combo* (target + attacker counter) staged in `docs/backpocket.md`, queued behind Stack H.
4. **OQ-59 (new — opening + endgame dead-air)** — 59a no Strike skills firing in opening (only Defense), 59b post-mid-exchange endgame conversion gap. Sub-problems decomposed; design pass deferred until chassis trim lands.

## Last session

2026-05-29 (Session 22): Project-wide Nico→Niko rename. `/playtest 4` executed end-to-end with multi-agent isolation; synthesis at `docs/research/playtest-4-analysis.md`. Post-analysis design discussion produced: OQ-38 reframe, dual-counter combo design, Path A methodology decision, 3 new OQs (58/59/60), 6 new backpocket entries. **TESTING_PLAN.typ rewritten**: stacks renamed (H = Armor Trim, A G3 = Dual-Counter Combo, K = Piece Count Reduction); Stack I folded into H; Stack B withdrawn; Stack K decoupled from Stack D; Stack F sequenced after A G3; state lifecycle (Active/Queued/Dormant/Resolved) introduced; decision tree replaced with per-stack routing.

## Next action

**Re-discuss Stack H — Armor Trim before drafting** (designer flag, Session 22 close): revisit bundled dose, scope, and entry conditions in the next session. Only after that conversation: write the rule sheet at `docs/test-scenarios/stack-h-armor-trim/`, build print packet (rule sheet + skill-cards + feedback + game-tracking ×2), schedule two experienced players. Routing rules and within-stack rollback (smaller dose) live in `TESTING_PLAN.typ`.


