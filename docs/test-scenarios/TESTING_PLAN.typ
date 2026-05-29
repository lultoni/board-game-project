#import "./shared/template.typ": *
#show: template.with(title: "(GAME NAME) — Testing Plan")

= Testing Plan

_Last updated: 2026-05-29 (Session 22, post-Playtest-4). Updated by Claude at end of each session._

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

=== Stack H — Armor Trim

*Targets:* OQ-11 chassis-volume hypothesis. Armor↔Armor-Breaker loop crowds out the combo loop (P4-confirmed).

*Variants (within-stack doses):*
- *Bundled (lead)*: Armor cap 3→2 *and* Armorsmith +1→+2 (one-shot fortify, not stack-grind).
- *Smaller dose (rollback)*: Armor cap 3→2 only, Armorsmith unchanged. Run as next iteration of Stack H if the bundled dose stalls (build cheaper than break).

*Status:* Rule sheet not yet written. Folder pending: `docs/test-scenarios/stack-h-armor-trim/`.

*Entry conditions:* Two experienced players required (chassis-volume read needs both players able to plan combos). Standard Attack 1 DMG (Layer 1 Stack A G1) and current combo bonus (Stack A G2) carry forward as baseline.

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

#hr

== Queued

_Stacks gated on a specific other stack's result, ordered by priority._

=== Q1. Stack A G3 — Dual-Counter Combo *(gated on Stack H)*

*Targets:* OQ-38 scope-not-strength reframe + OQ-58 exchange-pit + OQ-59 (esp. 59b endgame conversion gap).

*Mechanic summary* (full design in `docs/backpocket.md`):
- *Target counter* (kept from G2): different friendly Champions hit same enemy target → bonus on 2nd+ hit.
- *Attacker counter* (new): same friendly Champion hits different enemy targets → bonus on 2nd+ hit.
- Both counters live in parallel; if a hit qualifies for both, both fire (intuitive stacking — rare in practice).
- Scope widened: any skill that hits an enemy piece counts. Standard Attacks excluded.
- Multi-target skills (Blade Tempest) tick the counter on every hit piece. Watch flag — first rollback if dual-counter proves OP.

*Justifications:*
- (a) Cross-category crowd-out (P4 #3, Q-D3-risk).
- (b) Late-game offensive lockout (P4 #6 — Elias verbatim "I did not have any other attack champs left").
- (c) Mid-game exchange-pit pattern (P4 OQ-58 — attacker counter rewards distributing pressure across multiple fronts, not one-piece-at-a-time attrition).

*Teaching-cost flag (G4 / OQ-60):* two parallel counters is strictly more complex than current Stack A G2 — will likely need physical tokens or board-side trackers. Watched for cognitive-load violation.

*Routing on result:*
- *Exchange-pit dissolves* → resolve OQ-58 + OQ-38 dual-counter accepted. Next: monitor OQ-59 (opening + endgame dead-air).
- *Exchange-pit persists, dual-counter scope OK* → keep dual-counter, advance to *Stack F — Sente Skills* for a different mechanism.
- *Cognitive load too high (G4 violation)* → roll back to single-counter widened scope (Option A: Move-into-Strike).

=== Q2. Stack K — Piece Count Reduction *(gated on Stack H)*

*Targets:* OQ-27 piece density. *Decoupled from board geometry as of Session 22* — Stack D owns board size.

*Variant:* Current board (10×10 today) with 3 Champions + 3 Guards + 1 King per side (vs current 5+6+1).

*Entry conditions:* Two experienced players, full session.

*Routing on result:*
- *Density feels right, decisions sharper* → OQ-27 leans toward fewer pieces. Folds into Phase B baseline candidate.
- *Game gets too thin / too short* → density was load-bearing. Revisit only with smaller board (Stack D 8×8) bundled.

=== Q3. Stack J — Injured Trim *(gated on Stack H)*

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

*Variants:* Cascade trigger (+1 Skill Slot on kill, OQ-51), Pin / Threatened restriction, midline pressure skills (10 candidates staged in `docs/backpocket.md`).

*Status:* Rule sheet not yet written. Sequenced after Stack A G3 per Session 22 decision.

=== Stack G — Unified AP

*Trigger:* Core systems stable across A G3, H, J, K. Radical structural change — do not test alongside other active experiments.

*Variant:* 3 AP/turn unified action-point model, replacing separate Movement and Action phases.

*Status:* Draft written, not yet run. OQ-26.

#hr

== Resolved

_Outcome known. Listed for historical cross-reference; not active work._

#table(
  columns: (auto, 1fr, 1fr),
  table.header([Stack], [Outcome], [Source]),
  [*Stack A G1 — Attack Nerf*], [*Accepted into baseline (P3, 2026-05-17).* Standard attack 1 DMG. First Champion kill moved R26 → R11. Standoff dissolved.], [`playtest-3-analysis.md`],
  [*Stack A G2 — Combo Bonus*], [*Confirmed in mechanics, design-aligned in feel (P4, 2026-05-28).* Multi-Champion Strike-only counter scales +0/+1/+2. Stays in baseline. Scope-widening discussion produced *Stack A G3* — see Queued.], [`playtest-4-analysis.md` + Session 22 discussion],
  [*Stack B — Bodyguard Fix*], [*Withdrawn (Session 22, 2026-05-29).* Defender-only adjacency change. P4 confirmed Bodyguard tracks standoff state, not the rule (0 triggers when Armor stalling returned). Different solutions would be on the table even if Bodyguard remains broken post-Stack-H. The stack as drafted is not the right fix.], [P4 evidence; Session 22 designer call],
  [*Stack I — Armor Rollback*], [*Folded into Stack H (Session 22, 2026-05-29).* Was a contingency dose, not a distinct stack. Now lives as the smaller dose within Stack H.], [Session 22 restructure],
)

#hr

== Carry-Forward Baseline (in every future stack)

#table(
  columns: (auto, auto, 1fr),
  table.header([Layer], [Status], [What it adds]),
  [Layer 1: Economy fix], [*Accepted* (P2, 2026-04-24)], [6 start Runes · +2/turn · +1 every 5 rounds],
  [Stack A G1: Standard Attack 1 DMG], [*Accepted* (P3, 2026-05-17)], [Standard attack deals 1 DMG (was 2). Skills become primary damage source.],
  [Stack A G2: Multi-Champion Combo Bonus], [*Accepted* (P4, 2026-05-28)], [+0/+1/+2 counter on 2nd+ Strike on same target by different Champions same turn.],
)

_All future stacks must include these. Use `section-setup(start-runes: 6, layer1-accepted: true)` and `section-resource-economy(start-runes: 6, layer1-accepted: true)` in `baseline-sections.typ`._

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
  *Session 22 (2026-05-29) — restructure:*
  - Stack I dropped — folded into Stack H as the smaller dose. Same OQ, same hypothesis.
  - Stack B withdrawn — defender-only adjacency is not the right fix even if Bodyguard remains broken post-Stack-H.
  - Stack K decoupled from Stack D — K now owns Piece Count Reduction; D owns Board Geometry.
  - Stack A G3 (Dual-Counter Combo) added to Queued — drafted from P4 design discussion.
  - Decision tree (Phase 1 / Phase 2) replaced with per-stack routing rules.
  - State lifecycle: Active / Queued / Dormant / Resolved.
  - Stacks renamed for legibility; letter IDs preserved as stable cross-reference keys.

  *Session 16 (2026-05-19) — historical:* Pre-Stack-A-G2 prep complete. Range system clarification, Focus Strike note, tracking sheet redesign, dependency-correct rule order.

  *Session 15 (2026-05-18) — historical:* P3 confirmed Stack A G1. Bodyguard activated organically without Stack B → Stack B de-prioritised (later withdrawn in Session 22). OQ-52, OQ-53 raised.

  *Session 11 (2026-04-29) — historical:* Checkmate win condition killed. King Lifetime HP becomes Stack C lead. Sente skill design chosen as primary standoff solution (Stack F). G8 guardrail added.
]
