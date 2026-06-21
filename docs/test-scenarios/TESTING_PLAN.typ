#import "/docs/test-scenarios/shared/template.typ": *
#show: template.with(title: "(GAME NAME) — Testing Plan")

= Testing Plan

_Last updated: 2026-06-21 (Session 26 — Stack M rule sheet finalised after holiday-insights brainstorm). Updated by Claude at end of each session._

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

=== Stack M — Game Length Cut (GLC)

*Status: drafted Session 25 (2026-06-21).* Rule sheet + feedback form: `docs/test-scenarios/stack-m-game-length-cut/`.

*What's bundled (six simultaneous changes — intentional methodology deviation):*
+ Board 10×10 → *8×8*
+ Armor cap 3 → *2*
+ Injured state: penalties removed (still 2 HP tracker; no speed cap, no Range -1)
+ Draw conditions removed entirely (not replaced)
+ Steal cost 3 → *4* (both Modes)
+ Multi-Champion Combo Bonus widened on two axes: (a) counter *ticks* on *movement-causing skills* (Tempest, Hook, Blast, Shove, Swap when it relocates an enemy) by a new Champion, not just Strikes; (b) bonus damage applies to *any* skill — Strike or movement-causing — that affects a target with counter > 0. Movement skills become a damage vector once the counter is loaded.

*Hypothesis:* a single coordinated cut to chassis-volume, end conditions, and engagement-geometry produces a 30-60 minute game with a single-climax shape (Principle 8) without breaking the combo fantasy.

*Why bundled (per Principle 7):* core identity unsettled; six sequential isolation stacks cost 6+ playtest sessions. The bundle is a coordinated deployment of independently validated candidate fixes (P3-P5). Follow-up isolation stacks (piece count, unified actions, 6×6 board) defer attribution, do not lose it. Designer call verbatim: *"alles auf einmal — ich will schnellen progress sehen."*

*Watching:* game length (rounds + wall-clock vs 30-60 min target); single-climax shape (Principle 8 KPI); mid-game stalling pattern (P4 R15-R21 cluster — gone?); combo widening reception; Injured-as-tracker feel; no-draws environment (any infinite games?); Steal at cost 4 still must-pick?; 8×8 cramping; felt-PI (OQ-64); cognitive load (OQ-60).

*Routing on result* (full version in `stack-m-game-length-cut.typ`):
- *Length + shape + no-stalling all land* → Stack M accepted into baseline. Next: piece count (Stack K) or 6×6 board (Stack D variant) as isolation stacks.
- *Combo widening dominates / Tempest broken* → roll back movement-counter trigger only. Keep other five.
- *Length still too long, no stalling* → next is piece-count cut.
- *Cleverness gone (pure aggression)* → roll back combo widening OR Steal cost increase.
- *Injured-no-penalty makes pieces disposable* → roll back Injured change only.
- *No-draws causes infinite games* → restore only-Kings-remain draw condition.
- *Bundle uninterpretable* → individual rollback per axis in sequenced next stacks.

*Cross-refs:* OQ-11 (Armor — addressed); OQ-34 (Steal — addressed); OQ-38 (combo widening — addressed); OQ-57 (Injured penalties — addressed via removal); OQ-66 (game length target — primary axis); OQ-68 (draw conditions — resolved by removal); Principle 6 + Principle 7 + Principle 8; `docs/research/game-economy-map.md`.

#hr

== Queued

_Stacks gated on a specific other stack's result, ordered by priority._

=== Q1. Stack H — Armor Trim *(absorbed into Stack M; remains as isolation-fallback)*

*Status:* Armor cap 3→2 is one of Stack M's six bundled changes. Stack H as a standalone is *not* the next stack — but if Stack M's routing produces "rollback Armor only" or "rollback everything except Armor", Stack H steps in as the isolation stack for the Armor lever.

*Targets:* OQ-11 chassis-volume hypothesis (Armor↔Armor-Breaker loop crowds out the combo loop, P4-confirmed; P5 cross-pole confirmed). Now also a *game-length lever* (OQ-66).

*Bundled (lead) dose* (if revived): Armor cap 3→2 *and* Plate +1→+2 (one-shot fortify, not stack-grind). _Note: Stack M only takes the cap change; the Plate buff is not in Stack M and would re-enter via Stack H if needed._

*Smaller dose (rollback within Stack H)*: Armor cap 3→2 only, Plate unchanged. Run if the bundled dose stalls (build cheaper than break).

*Entry conditions:* Stack M result demands isolation of Armor lever, OR Stack M is rolled back entirely and Armor remains the primary lever.

*"Build cheaper than break" risk* — bigger than originally framed (user verbatim Session 23): *"if it is way easier to stack armor then it is to get rid of it... the change can exponetiallise this even more."*

*Cross-refs:* OQ-11, OQ-57 (Injured — also in Stack M), OQ-58, OQ-66, Q-C1, P3 + P4 + P5 evidence in `docs/research/playtest-3-analysis.md`, `playtest-4-analysis.md`, and `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`.

=== Q2. Stack A G3 — Dual-Counter Combo *(gated on Stack H)*

*Targets:* OQ-38 scope-not-strength reframe + OQ-58 exchange-pit + OQ-59 (esp. 59b endgame conversion gap).

*Mechanic summary* (full design in `docs/backpocket.md`):
- *Target counter* (kept from G2): different friendly Champions hit same enemy target → bonus on 2nd+ hit.
- *Attacker counter* (new): same friendly Champion hits different enemy targets → bonus on 2nd+ hit. *Session 25 narrowing note* (`docs/backpocket.md`): attacker counter felt too generous on reflection — narrow before shipping.
- Both counters live in parallel; if a hit qualifies for both, both fire (intuitive stacking — rare in practice).
- Scope widened (target counter): any skill that hits an enemy piece counts. Move-Attacks excluded.
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

=== Q3. Stack K — Piece Count Reduction *(gated on Stack H; also game-length lever OQ-66)*

*Targets:* OQ-27 piece density. *Decoupled from board geometry as of Session 22* — Stack D owns board size. Now also a game-length lever (OQ-66).

*Variant:* Current board (10×10 today) with 3 Champions + 4 Guards + 1 King per side (vs current 5+6+1).

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

=== Stack L — Pole B Per-Turn-Draft Prototype (consumable variant) *(PAUSED Session 25)*

*Trigger to revive:* Pole A revival track stalls AND a clear Pole B variant addresses P5's surfaced problems (felt-PI breakdown OQ-64; pure-reaction play). Candidates in `docs/backpocket.md`: permanently-equipped (non-consumable) drafted skills; per-Skill-Phase activation cap; skills cost a resource to activate.

*Status:* Rule sheet exists at `docs/test-scenarios/stack-l-per-turn-draft/`. P5 ran once (Elias vs Jonathan, digital, 15 rounds). Three structural problems surfaced; Pole A returns as Active. See OQ-61 partial resolution and `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`.

=== Stack C — Pacing

*Trigger:* First Champion kill past Round 20 in any future stack.

*Variants:* King Lifetime HP (unkillable-King → fixed length); Armor Decay (Armor breaks down each round).

*Status:* Rule sheet not yet written. P4 first-kill = R13 → not triggered. OQ-19, OQ-41.

=== Stack D — Board Geometry

*Trigger:* Board size or geometry surfaces as a bottleneck in any future stack. Also a candidate game-length lever (OQ-66).

*Variants:* 8×10 (OQ-52 narrower board), 8×8, hex grid (gated on `/research hex vs square grid in tactical games` per OQ-42).

*Status:* Rule sheet not yet written.

=== Stack E — Draft Flow

*Trigger:* Draft feels stale or under-explored after the Pole A revival stacks land. May be partially subsumed if pre-made loadouts (OQ-65) ship with simultaneous-reveal selection (OQ-62).

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
  [*Stack L — Pole B Per-Turn-Draft (consumable variant)*], [*Paused (Session 25, 2026-06-21) after P5.* Three structural problems surfaced: Armor 3 still felt mandatory (cross-pole confirmation of OQ-11), play collapsed to pure reaction (no multi-turn planning), felt-PI broke under combinatorial breadth (OQ-64). Pole A returns as Active track. Other Pole B variants in `docs/backpocket.md` may be revived if Pole A stalls again — see Dormant.], [P5 notes; OQ-61 partial resolution; Session 25],
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
  *Session 26 (2026-06-21) — holiday-insights gathering + Stack M rule sheet finalised:*
  - Designer returned from holiday and dumped accumulated insights. Multi-pass restructuring of the Stack M rule sheet: baseline section order restored; missing onboarding sections (Introduction + Simple Overview) added; diff-style "_(Baseline: X)_" annotations removed; over-use of `changed-box` callouts cut from 5 to 2 `⚡ CHANGED:` headers.
  - *Substantive design change inside Stack M's combo bonus*: bonus damage now applies to *any* skill (Strike OR movement-causing) that affects a target with counter > 0, not only to Strike hits. This unlocks damage strategies without Strike skills — movement skills become a damage vector once a target is counter-loaded. Counter-tick rules unchanged.
  - *Setup layout corrected*: Kings stand mid-back-row but *not directly opposite each other*; one side of each King has 2 Champions, the other 3; one Guard directly in front of each Champion + King (6 Guards total). Players do not choose tiles.
  - *Rule sheet hygiene*: Skill table column widths fixed (Effect is now the only flex column); Shield/Plate "_(max 2)_" annotations removed; Health & Armor collapsed to 2 short paragraphs; Skill System's "Injured Range penalty" line removed; in-text skill references switched to `sk()` chips; Facilitator Notes wrapped in Typst `/* */` block so the player-facing PDF no longer surfaces them.

  *Session 24 (2026-05-31) — three-goal session: Pole B + vocabulary + template:*
  - *Project-wide vocabulary simplification.* "Runes" → "Money" (existing player-facing rename), plus a broader pass across docs, skills, research, mechanics-evaluated, and Typst rule sheets to remove jargon and align terminology between files. 6 commits.
  - *Pole B rule sheet written* (`docs/test-scenarios/stack-l-per-turn-draft/`). Standalone — does not reference `baseline-sections.typ`. Vocabulary "Build Phase" → "Draft Phase". Three-phase turn (Move → Draft → Skill); Move and Draft share a 4-action pool; Skill Phase is free with consumable activations. Bodyguard sits between Move and Draft.
  - Backpocket entries added for Pole B variants: skills-cost-a-resource, per-Skill-Phase activation cap, permanently-equipped (non-consumable) drafted skills. See `docs/backpocket.md`.
  - *PDF template redesign.* Canonical `shared/template.typ` rebuilt from sample-variant feedback: H1 = 28pt Inter Display title (no eyebrow); H2 = numbered presence with calmer teal numerals + tight SECTION/title pair, sticky to following content; tables = cool grey header + charcoal hairline + light alt rows; new `sk("Lance")` chip helper for in-text skill references (light tinted pill + category-color outline + icon). Pagination fix: outer `breakable: false` wraps removed from `baseline-sections.typ` and `stack-l` (they were forcing half-empty pages); lists/enums now block-unbreakable so sentences don't split mid-bullet. `#hr` separators removed from rule docs (kept in feedback forms / game-tracking).

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
