#import "/docs/test-scenarios/shared/template.typ": *
#show: template.with(title: "(GAME NAME) — Testing Plan")

= Testing Plan

_Last updated: 2026-05-30 (Session 23, post-Pole-framing). Updated by Claude at end of each session._

#note-box[
  *How to read this document:*
  + Top section: *the current Active stack* — the one being run next.
  + Then: *Queued* (next in line, prioritised), *Dormant* (waiting on a trigger), *Resolved* (done or withdrawn).
  + Each stack has a *stable letter ID* (Stack H, Stack A G3, etc.) used by OQs and historical docs, plus a *descriptive name* (Armor Trim, Dual-Counter Combo).
  + Routing rules — "if result X, advance stack Y" — live with each stack, not in a separate decision tree.
]

#hr

== Active

_Exactly one stack is Active at a time — the one prepping to run, or running. Don't open a second._

=== Stack L — Pole B Per-Turn-Draft Prototype

*Pole framing (Session 23):* Pole A = pre-game-draft (current game). Pole B = per-turn-draft (this stack). See `docs/research/path-y-defense-redesign.md` for the full design discussion.

*Targets:* The defense-as-tax problem (OQ-11 from a different angle), game length (Principle 6 — length is attrition), drafting determinism (OQ-62). Tests whether moving the draft into the game flow changes the game-feel enough to justify the radical shape change.

*Mechanic summary* (full design in `docs/research/path-y-defense-redesign.md`):
- Skills are *added to pieces during play*, not all at once at the start.
- Skills still *fire once per use* but are *reusable while equipped*.
- *Equipped count cap = 12 skills per player* (6 Champions × 2 slots; King doesn't change the count).
- *Shared actions* — Move Phase and Draft Phase use the same action pool. Spending an action to draft a new skill onto a piece is a tempo cost (vs. moving). Skill Phase is free (no action cost). _(Vocabulary note Session 24: "Build Phase" renamed to "Draft Phase" to align with the existing "draft" terminology used elsewhere.)_
- *No Money-economy activation gate.* Activate as many skills as Money allows per turn; no per-turn cap. Hoarding is allowed (and risky — see backpocket "unstoppable one-turn killer" potential issue).
- *Effectively infinite skill pool* for drafting. The 12-equipped cap is the constraint, not the pool size.

*Entry conditions:* Digital prototype only for the first 2–3 games — 3-week vacation window with Jonathan is the cheap testing surface for a radical shift. No physical rule sheet for the first prototype runs; rules carried in shared digital state.

*What "good" looks like:*
- Game-feel measurably different from Pole A (whether better or worse — the comparison is the point).
- Game length drops vs P4 baseline (28-29 rounds, ~2h30).
- Drafting-determinism reads less "always better to react." Players make commitments early and the game rewards them.
- Cognitive load stays inside G4 budget (drafting *and* playing *and* opponent-drafting reads is the worry).

*Routing on result:*
- *Game-feel improves and length drops* → run 1–2 more, then design Pole B v2 with iteratively patched issues. Resolves OQ-61 partially toward "Pole B replaces."
- *Cognitive load too high or burst-turns dominate* → log learnings, restore *Stack H — Armor Trim* to Active, return to Pole A track. Resolves OQ-61 toward "Pole A continues."
- *Mixed result (some axes improved, some worse)* → design Pole B v2 to address the worst issue; do not simultaneously change another variable. OQ-61 stays open.
- *Cross-pole shared problem surfaces (e.g. defense-as-tax persists in Pole B too)* → that's the trigger to resolve OQ-63 (cross-pole fixing methodology).

*Cross-refs:* OQ-61 (two-pole framing), OQ-62 (Pole A draft information — Pole B is the alternative axis), OQ-63 (cross-pole fixing), OQ-11 (Armor — diagnosed in Pole A, may persist in Pole B), Principle 6 (game length as attrition), Principle 7 (fundamental shifts while core unsettled). Full design discussion: `docs/research/path-y-defense-redesign.md`.

*Status:* No rule sheet yet. The prototype runs digitally; if it survives 2–3 games, draft a rule sheet at `docs/test-scenarios/stack-l-per-turn-draft/`.

#hr

== Queued

_Stacks gated on a specific other stack's result, ordered by priority._

=== Q1. Stack H — Armor Trim *(Pole A track; deprioritised Session 23)*

*Targets:* OQ-11 chassis-volume hypothesis. Armor↔Armor-Breaker loop crowds out the combo loop (P4-confirmed).

*Variants (within-stack doses):*
- *Bundled (lead)*: Armor cap 3→2 *and* Plate +1→+2 (one-shot fortify, not stack-grind).
- *Smaller dose (rollback)*: Armor cap 3→2 only, Plate unchanged. Run as next iteration of Stack H if the bundled dose stalls (build cheaper than break).

*Status:* Rule sheet not yet written. Folder pending: `docs/test-scenarios/stack-h-armor-trim/`. *Deprioritised Session 23* — Pole B per-turn-draft prototype claims the active slot. Stack H bundled dose remains the lead variant for when Stack H runs.

*Entry conditions:* Two experienced players required (chassis-volume read needs both players able to plan combos). Move-Attack 1 damage (Layer 1 Stack A G1) and current combo bonus (Stack A G2) carry forward as baseline.

*"Build cheaper than break" risk* — bigger than originally framed (user verbatim Session 23): *"if it is way easier to stack armor then it is to get rid of it... the change can exponetiallise this even more."* Within-stack rollback (cap-only) is the contingency if Armor totals climb past P4 baseline (14 / 22).

*What "good" looks like:*
- Armor granted total per game drops noticeably from P4 baseline (P4: 14 / 22).
- Mid-game Armor-stack arc (P4 R15-R21 cluster) shortens or disappears.
- Q13 mental-focus self-report drops vs P4 ("Yes, a lot" → "Slightly" or lower).
- Combo loops fire mid-game, not only after a 7-round Armor consolidation.

*Routing on result:*
- *Bundled works (Armor volume drops, exchange-pit pattern softens)* → resolve OQ-11 + watch OQ-58. Next Active = *Stack A G3 — Dual-Counter Combo*.
- *Bundled stalls (Armor stalling becomes dominant — build cheaper than break)* → run smaller dose as iteration. Same OQ, smaller variable.
- *Bundled works, exchange-pit pattern persists* → confirms OQ-58 is independent of Armor volume. Next Active = *Stack A G3*.
- *Bundled works, first kill jumps past R20* → unlikely but watch. Next Active = *Stack C — Pacing*.

*Cross-refs:* OQ-11, OQ-57 (gated behind H), OQ-58 (watched under H), Q-C1, P3 + P4 evidence in `docs/research/playtest-3-analysis.md` and `playtest-4-analysis.md`.

=== Q2. Stack A G3 — Dual-Counter Combo *(gated on Stack H)*

*Targets:* OQ-38 scope-not-strength reframe + OQ-58 exchange-pit + OQ-59 (esp. 59b endgame conversion gap).

*Mechanic summary* (full design in `docs/backpocket.md`):
- *Target counter* (kept from G2): different friendly Champions hit same enemy target → bonus on 2nd+ hit.
- *Attacker counter* (new): same friendly Champion hits different enemy targets → bonus on 2nd+ hit.
- Both counters live in parallel; if a hit qualifies for both, both fire (intuitive stacking — rare in practice).
- Scope widened: any skill that hits an enemy piece counts. Move-Attacks excluded.
- Multi-target skills (Tempest) tick the counter on every hit piece. Watch flag — first rollback if dual-counter proves OP.

*Justifications:*
- (a) Cross-category crowd-out (P4 #3, Q-D3-risk).
- (b) Late-game offensive lockout (P4 #6 — Elias verbatim "I did not have any other attack champs left").
- (c) Mid-game exchange-pit pattern (P4 OQ-58 — attacker counter rewards distributing pressure across multiple fronts, not one-piece-at-a-time attrition).

*Teaching-cost flag (G4 / OQ-60):* two parallel counters is strictly more complex than current Stack A G2 — will likely need physical tokens or board-side trackers. Watched for cognitive-load violation.

*Routing on result:*
- *Exchange-pit dissolves* → resolve OQ-58 + OQ-38 dual-counter accepted. Next: monitor OQ-59 (opening + endgame dead-air).
- *Exchange-pit persists, dual-counter scope OK* → keep dual-counter, advance to *Stack F — Sente Skills* for a different mechanism.
- *Cognitive load too high (G4 violation)* → roll back to single-counter widened scope (Option A: Move-into-Strike).

=== Q3. Stack K — Piece Count Reduction *(gated on Stack H)*

*Targets:* OQ-27 piece density. *Decoupled from board geometry as of Session 22* — Stack D owns board size.

*Variant:* Current board (10×10 today) with 3 Champions + 3 Guards + 1 King per side (vs current 5+6+1).

*Entry conditions:* Two experienced players, full session.

*Routing on result:*
- *Density feels right, decisions sharper* → OQ-27 leans toward fewer pieces. Folds into Phase B baseline candidate.
- *Game gets too thin / too short* → density was load-bearing. Revisit only with smaller board (Stack D 8×8) bundled.

=== Q4. Stack J — Injured Trim *(gated on Stack H)*

*Targets:* OQ-57 — does Injured's mechanical chassis (speed cap, Range −1, self/adjacent carve-out) pay for itself in game-feel?

*Variant:* Remove Injured's mechanical downsides. State persists as HP-tracker only.

*Why gated on H:* Armor chassis-volume reduction must land first so Injured-volume signal reads cleaner.

*Routing on result:*
- *Game still reads well, no downside felt* → Injured-as-HP-tracker becomes baseline candidate. Volume cost was unjustified.
- *Injured pieces feel structureless (no threat/penalty)* → keep current Injured rules. OQ-57 closes negative.

#hr

== Dormant

_Waiting on a trigger that hasn't been hit. No internal ordering._

=== Stack C — Pacing

*Trigger:* First Champion kill past Round 20 in any future stack.

*Variants:* King Lifetime HP (unkillable-King → fixed length); Armor Decay (Armor breaks down each round).

*Status:* Rule sheet not yet written. P4 first-kill = R13 → not triggered. OQ-19, OQ-41.

=== Stack D — Board Geometry

*Trigger:* Board size or geometry surfaces as a bottleneck in any future stack.

*Variants:* 8×10 (OQ-52 narrower board), 8×8, hex grid (gated on `/research hex vs square grid in tactical games` per OQ-42).

*Status:* Rule sheet not yet written.

=== Stack E — Draft Flow

*Trigger:* Draft feels stale or under-explored after Stack A G3 lands.

*Variants:* Pool draft (OQ-35 — draft pool first, assign after); placement order (OQ-36 + OQ-48 — equip skills first, then place on board).

*Status:* Rule sheet not yet written.

=== Stack F — Sente Skills

*Trigger:* *Stack A G3 ran but exchange-pit pattern persists.* Sente threats are a different mechanism for the same problem (forcing forward commitment) — not a duplicate.

*Variants:* Cascade trigger (+1 action on kill, OQ-51), Pin / Threatened restriction, midline pressure skills (10 candidates staged in `docs/backpocket.md`).

*Status:* Rule sheet not yet written. Sequenced after Stack A G3 per Session 22 decision.

=== Stack G — Unified AP

*Trigger:* Core systems stable across A G3, H, J, K. Radical structural change — do not test alongside other active experiments.

*Variant:* 3 actions/turn unified action-point model, replacing separate Move and Skill phases.

*Status:* Draft written, not yet run. OQ-26.

#hr

== Resolved

_Outcome known. Listed for historical cross-reference; not active work._

#table(
  columns: (auto, 1fr, 1fr),
  table.header([Stack], [Outcome], [Source]),
  [*Stack A G1 — Attack Nerf*], [*Accepted into baseline (P3, 2026-05-17).* Move-Attack 1 damage. First Champion kill moved R26 → R11. Standoff dissolved.], [`playtest-3-analysis.md`],
  [*Stack A G2 — Combo Bonus*], [*Confirmed in mechanics, design-aligned in feel (P4, 2026-05-28).* Multi-Champion Strike-only counter scales +0/+1/+2. Stays in baseline. Scope-widening discussion produced *Stack A G3* — see Queued.], [`playtest-4-analysis.md` + Session 22 discussion],
  [*Stack B — Bodyguard Fix*], [*Withdrawn (Session 22, 2026-05-29).* Defender-only adjacency change. P4 confirmed Bodyguard tracks standoff state, not the rule (0 triggers when Armor stalling returned). Different solutions would be on the table even if Bodyguard remains broken post-Stack-H. The stack as drafted is not the right fix.], [P4 evidence; Session 22 designer call],
  [*Stack I — Armor Rollback*], [*Folded into Stack H (Session 22, 2026-05-29).* Was a contingency dose, not a distinct stack. Now lives as the smaller dose within Stack H.], [Session 22 restructure],
)

#hr

== Carry-Forward Baseline (in every future stack)

#table(
  columns: (auto, auto, 1fr),
  table.header([Layer], [Status], [What it adds]),
  [Layer 1: Economy fix], [*Accepted* (P2, 2026-04-24)], [6 start Money · +2/turn · +1 every 5 rounds],
  [Stack A G1: Move-Attack 1 damage], [*Accepted* (P3, 2026-05-17)], [Move-Attack deals 1 damage (was 2). Skills become primary damage source.],
  [Stack A G2: Multi-Champion Combo Bonus], [*Accepted* (P4, 2026-05-28)], [+0/+1/+2 counter on 2nd+ Strike on same target by different Champions same turn.],
)

_All future stacks must include these. Use `section-setup(start-money: 6, layer1-accepted: true)` and `section-resource-economy(start-money: 6, layer1-accepted: true)` in `baseline-sections.typ`._

#hr

== Stacks Ready to Print Now

#table(
  columns: (auto, 1fr, auto),
  table.header([File], [Contents], [State]),
  [`shared/game-tracking.pdf`], [Per-player in-game tracking sheet. Print 1 per player per game.], [—],
  [`shared/skill-cards.pdf`], [15 skill reference cards (one per skill, 2×2 range matrix).], [—],
  [`shared/feedback-onboarding.pdf`], [First-game-only feedback form. Stack-independent.], [—],
  [`shared/feedback-baseline.pdf`], [Template for stack-specific feedback forms.], [—],
)

_Stack-specific PDFs (Stack H rule sheet + feedback) will be added here once written._

#hr

== How to Create a New Stack

+ Identify which OQ(s) the new stack targets.
+ Pass the *Justification Rule check*: what specific problem does this fix, or what game-feel improvement does it deliver?
+ Create folder: `docs/test-scenarios/stack-X-<descriptive-slug>/` (e.g. `stack-h-armor-trim/`).
+ Write `stack-X-<slug>.typ` — import `baseline-sections.typ`, call section functions for unchanged rules. For Quick Reference, prefer `#section-quick-reference(overrides: (...), extra-rows: (...))` over inlining the table.
+ Write `stack-X-feedback.typ` — copy `shared/feedback-baseline.typ`, fill `[STACK: ...]` placeholders, add OQ-monitoring questions for this stack's active OQs.
+ Run `zsh docs/test-scenarios/build-pdfs.sh` — the script auto-discovers new `.typ` files; no list maintenance needed.
+ Update this `TESTING_PLAN.typ`: add the stack to *Active* (replacing the previous Active) or *Queued*, depending on its gate state.

#hr

== Naming Convention

Stacks have a *stable letter ID* and a *descriptive name*. The letter ID is what OQs, mechanics-evaluated.md, and historical playtest analyses cross-reference — never renumber, never recycle. The descriptive name is what humans read in this doc and in conversation.

Format in references: `Stack H — Armor Trim` or `Armor Trim (Stack H)`.

Letter IDs are immutable once assigned. Names can be refined; if a name changes, log it in the Resolved table or a Session note.

#hr

== Session Notes

_Latest at top._

#note-box[
  *Session 24 (2026-05-31) — Pole B rule sheet drafted:*
  - First standalone rule sheet for Stack L written at `docs/test-scenarios/stack-l-per-turn-draft/`. Fully inline (does not reference `baseline-sections.typ`) so it stands on its own as the prototype ruleset.
  - *Vocabulary*: "Build Phase" renamed to "Draft Phase" — aligns with the existing "draft" verb already used for skill-pool selection.
  - *Phase structure*: turn split into three distinct phases — Move Phase → Draft Phase → Skill Phase. Move and Draft share a 4-action pool; Skill Phase is free.
  - *Skill-activation model* (active prototype): drafted skills are *consumable* — exhaust on activation, return to the shared pool.
  - Backpocket entries added for Pole B variants: skills-cost-a-resource, per-Skill-Phase activation cap, permanently-equipped (non-consumable) drafted skills. See `docs/backpocket.md`.

  *Session 23 (2026-05-30) — pole framing introduced:*
  - Two-pole game shape declared: *Pole A* (pre-game-draft, current game) and *Pole B* (per-turn-draft, radical alternative).
  - *Stack L — Pole B Per-Turn-Draft Prototype* added as the new Active stack. First 2–3 runs are digital-prototype only (3-week vacation testing window with Jonathan).
  - *Stack H — Armor Trim* deprioritised from Active to Queued. Bundled dose remains the lead variant for when Stack H runs. "Build cheaper than break" risk noted as bigger than originally framed.
  - Cross-pole testing question (do shared fixes run twice or once?) raised as OQ-63 — resolved on first encounter, not in advance.
  - Two new design principles promoted: (6) game length is itself a form of attrition; (7) while core identity is unsettled, prefer fundamental shifts over variable tweaking. See `docs/design-principles.md`.
  - *Multi-Champion Combo Bonus* (Stack A G2) migrated into baseline — concise version of the rule now lives in `shared/baseline-sections.typ` as `section-multi-champion-combo()`.
  - Full design discussion: `docs/research/path-y-defense-redesign.md`.

  *Session 22 (2026-05-29) — restructure:*
  - Stack I dropped — folded into Stack H as the smaller dose. Same OQ, same hypothesis.
  - Stack B withdrawn — defender-only adjacency is not the right fix even if Bodyguard remains broken post-Stack-H.
  - Stack K decoupled from Stack D — K now owns Piece Count Reduction; D owns Board Geometry.
  - Stack A G3 (Dual-Counter Combo) added to Queued — drafted from P4 design discussion.
  - Decision tree (Phase 1 / Phase 2) replaced with per-stack routing rules.
  - State lifecycle: Active / Queued / Dormant / Resolved.
  - Stacks renamed for legibility; letter IDs preserved as stable cross-reference keys.

  *Session 16 (2026-05-19) — historical:* Pre-Stack-A-G2 prep complete. Range system clarification, Focus note, tracking sheet redesign, dependency-correct rule order.

  *Session 15 (2026-05-18) — historical:* P3 confirmed Stack A G1. Bodyguard activated organically without Stack B → Stack B de-prioritised (later withdrawn in Session 22). OQ-52, OQ-53 raised.

  *Session 11 (2026-04-29) — historical:* Checkmate win condition killed. King Lifetime HP becomes Stack C lead. Sente skill design chosen as primary standoff solution (Stack F). G8 guardrail added.
]
