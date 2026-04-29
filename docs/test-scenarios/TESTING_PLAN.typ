#import "./shared/template.typ": *
#show: template.with(title: "(GAME NAME) — Testing Plan & Decision Tree")

= Testing Plan — Dynamic Stack System

_Last updated: 2026-04-29. Updated by Claude at end of each session._

#note-box[
  *How to use this document:* After each playtest, read the decision tree on the next page to find which stack to run next. Pick the stack that addresses the most pressing issue the playtest surfaced. PDFs for all available tests are in their stack folder.
]

#hr

== Testing Stacks

A *stack* = a group of 1–3 test scenarios that all push toward the same experience outcome. Stacks are independent — run them in any order based on what the game needs most.

#table(
  columns: (auto, auto, 1fr, auto),
  table.header([Stack], [Outcome], [Scenarios in Stack], [Status]),
  [*A*], [Make cleverness rewarding], [L2-G1: Attack nerf · L2-G2: + Combo bonus], [*Ready — print now*],
  [*B*], [Make Guards matter more], [L3: Bodyguard fix (defender-only)], [*Ready — print now*],
  [*C*], [Shorten games / accelerate kills], [King Lifetime HP · Armor Decay], [Not yet written],
  [*D*], [Optimise board feel and scale], [8×8 board · Piece count · Hex grid (gated on `/research`)], [Not yet written],
  [*E*], [Improve drafting experience], [Pool draft (OQ-35) · Placement order (OQ-36+48)], [Not yet written],
  [*F*], [More levers for clever plays], [Cascade trigger · Pin/Threatened · Sente skills (10 candidates staged)], [Not yet written],
  [*G*], [Radical structure redesign], [Unified AP framework (no separate Movement/Action phases)], [Draft written — not yet tested],
)

#hr

== Stacks Ready to Print Now

#table(
  columns: (auto, 1fr, auto),
  table.header([File], [Contents], [Stack]),
  [`stack-a-cleverness/stack-a-game1-attack-nerf.pdf`], [Standard attack 1 DMG. Play FIRST.], [A],
  [`stack-a-cleverness/stack-a-game2-attack-nerf-combo.pdf`], [Same + multi-Champion combo bonus. Play SECOND.], [A],
  [`stack-a-cleverness/stack-a-feedback.pdf`], [Feedback form — fill out after both Stack A games.], [A],
  [`stack-b-guards/stack-b-bodyguard-fix.pdf`], [Bodyguard adjacency: defender only.], [B],
  [`stack-b-guards/stack-b-feedback.pdf`], [Feedback form — fill out after Stack B.], [B],
  [`shared/game-tracking.pdf`], [Per-player in-game tracking sheet. Print 1 per player per game.], [—],
)

#hr

== Entry Conditions Per Stack

#table(
  columns: (auto, 1fr),
  table.header([Stack], [When to enter]),
  [*A*], [Always first — foundation for all other stacks. Results drive most downstream decisions.],
  [*B*], [Any time. Independent of Stack A combat results. Run whenever Guard utility feels low.],
  [*C*], [If first Champion kill still past Round 20 after Stack A. Two candidates: King Lifetime HP (unkillable-King) and Armor Decay (infinite-cycling). Independent of Stack B.],
  [*D*], [If Stack A + B data shows board size or piece density as bottleneck. Hex variant gated on running `/research hex vs square grid` first.],
  [*E*], [After Stack B (Layer 3) is accepted. Independent of combat system.],
  [*F*], [After Stack A combo bonus data. Dependent on combo ceiling AND standoff persistence. Gated on Stack A/B combat balance confirmation.],
  [*G*], [After Stacks A, B, and C stabilise. Radical change — do not test alongside other active experiments.],
)

#hr

== Current Priority Sequence

_Result-driven — this ordering updates after each playtest. Last confirmed: Session 11 (2026-04-29)._

#table(
  columns: (auto, 1fr, 1fr),
  table.header([Priority], [Action], [Trigger to advance]),
  [*P1*], [Run Stack A playtest (two games in one session)], [Print and schedule session],
  [*P2*], [Run Stack B playtest (can run same session or separate)], [Independent — any time],
  [*P3*], [Evaluate A + B results → choose next stack via decision tree], [Both A and B data in hand],
  [*P4*], [Skill balance monitoring (Rune Theft, Shadow Shift, Blade Call)], [Ongoing during P1–P3],
)

#note-box[
  *After P3*: the decision tree on the next page determines which of Stacks C, D, E, F, or G is next. There is no fixed ordering beyond this point — it depends on playtest results.
]

#pagebreak()

== Decision Tree

_After each playtest, follow the branch that matches your result to find the highest-value next stack._

=== Phase 1: Stack A + B Evaluation

#table(
  columns: (auto, auto, 1fr, auto),
  table.header([After], [Result], [Meaning], [Next]),
  table.hline(),
  [*Stack A*], [Champion kill before R15], [Combo ceiling raised — attack nerf + bonus working], [Check standoff ↓],
  [], [Champion kill R15–R20], [Partial improvement — monitor], [Check standoff ↓],
  [], [Champion kill after R20], [Pacing problem confirmed — kills too slow], [*→ Stack C*],
  table.hline(),
  [Standoff?], [Yes — standoff persists], [Players still won't commit forward], [*→ Stack F*],
  [], [No — engagement healthy], [Attack nerf dissolved the gap], [*→ Stack B*],
  table.hline(),
  [*Stack B*], [Bodyguard triggers ≥ 3], [Stack B accepted — Guards are useful screens], [Evaluate A+B ↓],
  [], [Bodyguard triggers 1–2], [Partial — consider attacker-only variant], [Evaluate A+B ↓],
  [], [Bodyguard triggers 0], [Still broken — investigate root cause], [Evaluate A+B ↓],
  table.hline(),
  [*A+B done*], [Kill still past R20], [Game too long despite combo bonus], [*→ Stack C*],
  [], [Combo ceiling still low], [Skills still don't feel dominant], [*→ Stack F*],
  [], [Board feels cramped/empty], [Spatial problems surfaced], [*→ Stack D*],
  [], [Draft feels stale], [Not enough variety/agency in draft], [*→ Stack E*],
  [], [Core stable — no urgent issues], [Ready for radical structure test], [*→ Stack G*],
)

#pagebreak()

=== Phase 2: Stack C → G Evaluation

#table(
  columns: (auto, auto, 1fr, auto),
  table.header([After], [Result], [Meaning], [Next]),
  table.hline(),
  [*Stack C*], [Game ends under R25], [Pacing solved], [→ next issue below],
  [_(Pacing)_], [Game still R25+], [Board size / piece count is the problem], [*→ Stack D*],
  table.hline(),
  [*Stack F*], [Standoff dissolved + clever plays rewarded], [Skill variety working], [→ next issue below],
  [_(Cleverness II)_], [Partially — some improvement], [Need more sente skills], [*Iterate in F*],
  [], [No — standoff persists], [Deploy fallback: midline bonus or generators], [→ next issue below],
  table.hline(),
  [*Stack D*], [Board feel improved], [Accept new board config], [→ next issue below],
  [_(Board)_], [Tradeoffs / not improved], [Try other variant (hex, piece count)], [*Iterate in D*],
  table.hline(),
  [*Stack E*], [Draft experience improved], [Accept draft change], [*→ Stack G*],
  [_(Draft)_], [Not improved], [Iterate within E], [*Iterate in E*],
  table.hline(),
  [*Stack G*], [AP system is an improvement], [Accept — becomes new baseline], [Done],
  [_(Structure)_], [AP system not better], [Keep current two-phase turn structure], [Done],
)

#v(1em)

*"Next issue" routing after any stack:*
- Kill timing still past R20 → *Stack C*
- Combo ceiling / standoff → *Stack F*
- Board cramped → *Stack D*
- Draft stale → *Stack E*
- Core stable → *Stack G*

#hr

#note-box[
  *Session 11 context (2026-04-29):* Checkmate win condition killed — King Lifetime HP replaces it as Stack C's lead mechanic. Sente skill design chosen as primary standoff solution (affects Stack F). 10 skill candidates staged in `docs/backpocket.md`. G8 guardrail added: "Players must always want to do more than they can execute." All future stack evaluations should check G8 compliance.
]

#hr

== Accepted Layers (Carry Forward Into All Future Sheets)

#table(
  columns: (auto, auto, 1fr),
  table.header([Layer], [Status], [What it adds]),
  [1: Economy fix], [*Accepted* (Playtest 2, 24.04.2026)], [6 start Runes · +2/turn · +1 every 5 rounds],
)

_All future layers must include Layer 1 economy values. Use `section-setup(start-runes: 6, layer1-accepted: true)` and `section-resource-economy(start-runes: 6, layer1-accepted: true)` in baseline-sections.typ._

#hr

== How to Create a New Test Layer

+ Identify which stack the new layer belongs to.
+ Create folder: `docs/test-scenarios/layer-N-<stack-slug>/`
+ Write `layer-N-<desc>.typ` — import `baseline-sections.typ`, call section functions with only the changed parameters.
+ Write `layer-N-feedback.typ` — copy `shared/feedback-baseline.typ`, fill `[LAYER: ...]` placeholders, add OQ-monitoring questions for this layer's active OQs.
+ Add both files to `build-pdfs.sh`.
+ Update this TESTING_PLAN.typ: add the new scenario to the stack table, entry conditions, and decision tree.
+ Run `zsh docs/test-scenarios/build-pdfs.sh` to compile.
