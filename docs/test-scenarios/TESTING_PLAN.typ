#import "./shared/template.typ": *
#import "@preview/mmdr:0.2.1": mermaid
#show: template.with(title: "(GAME NAME) — Testing Plan & Decision Tree")

= Testing Plan — Dynamic Stack System

_Last updated: 2026-04-28. Updated by Claude at end of each session._

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
  [*C*], [Shorten games / accelerate kills], [Checkmate win condition · L5: Board + piece count], [Not yet written],
  [*D*], [Optimise board feel], [8×8 board · Hex grid (needs /research first)], [Not yet written],
  [*E*], [Improve drafting experience], [Pool draft (OQ-35) · Placement order (OQ-36+48)], [Not yet written],
  [*F*], [More levers for clever plays], [OQ-51: Cascade trigger · Positional payoff · Ultimate skills], [Not yet written],
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
  [*C*], [If first Champion kill is still past Round 20 after Stack A. Checkmate layer is independent. Board/piece count waits for L2+L3 kill data.],
  [*D*], [If Stack A + B data shows board size as bottleneck. Hex variant gated on running `/research hex vs square grid` first.],
  [*E*], [After Stack B (Layer 3) is accepted. Independent of combat system.],
  [*F*], [After Stack A combo bonus data is in. Dependent on whether combo ceiling still feels low.],
)

#pagebreak()

== Decision Tree

_After each playtest, follow the branch that matches your result to find the highest-value next stack._

#figure(
  mermaid("flowchart TD
    START([Session start]) --> L2

    subgraph StackA [Stack A — Cleverness]
      L2[L2-G1: Attack nerf\n1 DMG standard attack]
      L2G2[L2-G2: + Combo bonus\nmulti-Champion coordination]
    end

    L2 --> L2G2
    L2G2 --> A_EVAL{Champion kill\nround?}

    A_EVAL -->|before R15| A_STRONG[Stack A strong result\nCombo ceiling raised]
    A_EVAL -->|R15-R20| A_PARTIAL[Stack A partial\nMonitor further]
    A_EVAL -->|after R20| A_WEAK[Pacing problem confirmed\nStack C urgent]

    A_STRONG --> CHOOSE1{Choose next stack}
    A_PARTIAL --> CHOOSE1
    A_WEAK --> STACKC

    CHOOSE1 --> StackB
    CHOOSE1 --> STACKC
    CHOOSE1 --> STACKF

    subgraph StackB [Stack B — Guards]
      L3[L3: Bodyguard fix\nadjacent to defender only]
    end

    L3 --> B_EVAL{Bodyguard\ntriggers per game?}
    B_EVAL -->|3 or more| B_STRONG[Stack B accepted\nGuards are useful screens]
    B_EVAL -->|1-2| B_PARTIAL[Stack B partial\nConsider attacker-only variant]
    B_EVAL -->|zero| B_WEAK[Bodyguard still broken\nInvestigate cause]

    B_STRONG --> CHOOSE2{Choose next stack}
    B_PARTIAL --> CHOOSE2
    B_WEAK --> CHOOSE2

    CHOOSE2 --> STACKE[Stack E: Draft\nPool draft + Placement]
    CHOOSE2 --> STACKF[Stack F: Cleverness II\nOQ-51 levers]
    CHOOSE2 --> STACKD[Stack D: Board\n8x8 or Hex]

    subgraph StackC [Stack C — Pacing]
      STACKC[Checkmate win condition\nOR board + piece count]
    end

    STACKC --> CHOOSE3{Which first?}
    CHOOSE3 -->|game length primary| CHECKMATE[Checkmate layer\nindependent of combat]
    CHOOSE3 -->|piece count primary| BOARDSCALE[L5: Board + pieces\nbundle with Stack D]
  "),
  caption: [Dynamic testing decision tree. Update this diagram after each session.],
)

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
+ Update this TESTING_PLAN.typ: add the new scenario to the stack table and decision tree.
+ Run `zsh docs/test-scenarios/build-pdfs.sh` to compile.
