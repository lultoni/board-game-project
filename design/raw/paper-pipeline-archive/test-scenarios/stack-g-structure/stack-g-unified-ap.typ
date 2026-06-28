#import "/docs/test-scenarios/shared/template.typ": *
#import "/docs/test-scenarios/shared/baseline-sections.typ": *
#show: template.with(title: "Test Layer 4: Unified Action Point System")

= Test Layer 4: Unified Action Point System

_Version: 2026-04-25 framework. To be finalised after Layers 1–3 are tested._

#note-box[
  *Framework — to be finalised after Layers 1–3 are tested.*
  Incorporates whichever Layer 1–3 changes were accepted. Sections marked [FROM LAYER X] indicate where those results plug in.
]

*What we're testing:* Does collapsing two turn phases into a single Action Point pool create better decisions and reduce cognitive overhead?

*Hypothesis:* When moving and casting draw from the same resource (AP), every action involves an opportunity cost trade-off. This creates more "agonising choices" and makes spells feel more central — they compete directly with movement rather than living in a separate phase.

*Watch for:*
- Do turns feel more fluid (one decision space) or more overwhelming (too many options)?
- Do players spread AP across pieces or focus on one?
- Does anyone rush a piece across the board using all 3 AP for movement?
- Are "pure caster" turns (0 movement, 3 skills) viable and interesting?
- Is 3 AP the right number?

#section-goal()

== Components

- *Board:* [FROM LAYER results — likely 10×10, no terrain.]
- *Per player:* [Current piece count — likely 5 Champions + 6 Guards + 1 King unless changed.]
- *Shared:* Skill tokens/cards, Money tokens, Round tracker.

== Setup

_[Same as current after Layer 1–3 results.]_

== ⚡ CHANGED: Turn Structure — Unified Action Points

#changed-box[
  *Old system (two phases):*
  + Move Phase: 2 actions
  + Skill Phase: N actions

  *New system (unified):*

  Each turn you receive *3 Action Points (AP).* You may spend each AP on one of:
  - *Move:* Move one piece (by its speed).
  - *Skill:* Activate one equipped skill on a Champion or King (pay Money cost).
  - *Attack:* Perform a Move-Attack (move onto enemy tile, deal 1 damage).

  You may use 0, 1, 2, or 3 AP. Unused AP are lost — no banking.

  *The separate per-phase action budgets are removed.* Each skill activation costs 1 AP + its Money cost. The former Skill Phase progression (was +1 every 10 rounds) is replaced by the flat 3 AP per turn. If the game needs late-game escalation, consider increasing AP to 4 after Round 15 (test separately).
]

=== Piece Freedom — Test All Four Models

#table(
  columns: (auto, 1fr, 1.5fr),
  table.header([Model], [Rule], [Rationale]),
  [*A*], [Each piece can use at most 1 skill per turn. Moving the same piece multiple times is allowed.], [Spreads skill usage. Doesn't prevent movement rushing.],
  [*B*], [Each piece can receive at most 1 AP per turn.], [Forces spreading across army. Most restrictive — every AP goes to a different piece.],
  [*C*], [Uncapped normally. When ≤ 2 pieces remain, that player's pieces can receive unlimited AP.], [Standard play spreads actions. Desperation mode enables comeback.],
  [*D*], [A single piece can receive at most 2 AP per turn. The 3rd AP must go to a different piece.], [Allows Move + Skill on one piece but prevents full-AP rushing. Moderate.],
)

*Recommended test order:* Start with Model D (moderate, intuitive). If too restrictive → test A. If rushing is a problem → test B. Test C as a variant on whichever base model works.

== Combat

_[FROM LAYERS 2–3: Use 3 HP system if accepted. Use simplified Bodyguard if accepted.]_

*Move-Attack:* Spend 1 AP to move your piece onto an enemy-occupied tile. Deal 1 damage. If target removed, occupy tile. If target survives, your piece stops on the tile before.

== Resource Economy

_[FROM LAYER 1 results.]_

#section-skill-system()
#section-health-armor()

== Progression

*AP per turn:*
#table(
  columns: (auto, auto),
  table.header([Round], [AP per turn]),
  [All rounds], [3],
  [After Round 15 (optional test)], [4],
)

#pagebreak()

#section-skill-reference()

== Post-Game Questions

+ Did turns feel more fluid or more overwhelming?
+ How often did you want a 4th AP? How often did you have AP left over?
+ Did you ever rush a single piece across the board? Was it effective or punished?
+ Did "Move + Skill on the same piece" feel like a natural combo?
+ Which piece freedom model did you test? How did it feel?
+ Do you miss the two-phase structure? Or is unified AP better?
+ Should AP increase in later rounds, or is 3 flat the right number?

== Layer 5: Board Size & Piece Count (Placeholder)

#note-box[
  Cannot be fully specified until Layer 4 results are in. The AP budget and piece count interact too heavily.
]

*Direction:* Shrink to 8×8. Reduce pieces (3 Champions + 3 Guards + 1 King is the proposal, but the exact ratio depends on how AP and HP systems interact).

*Key questions for Layer 5:*
- Is 8×8 with 14 pieces the right density?
- Is 3:3 Champion:Guard ratio correct? (King makes 4 skill-carriers vs 3 Guards — asymmetric)
- Does starting placement matter significantly on 8×8? Should it become a strategic choice?
- Is no terrain the right call, or does the smaller board need terrain to create interesting zones?

_Will be written as a full rule sheet after Layers 1–4 are resolved._
