// feedback-baseline.typ — base template for all playtest feedback forms
// HOW TO USE:
//   1. Copy this file to stack-X-<slug>/stack-X-feedback.typ
//   2. Replace every line marked with [STACK: ...] with stack-specific content
//   3. Section C follows the OQ-monitor pattern below — update it per stack
//   4. Do NOT change the Section D free notes block
//   5. Run build-pdfs.sh to compile (discovers new files automatically)
//
// ─── WRITING SPACE CONVENTION ────────────────────────────────────────────────
//
// After every #fq("N")[...] block, add #v(1fr) to give the player physical
// writing room. Use #v(1fr) (ratio-based) NOT #v(Ncm) (fixed) — fixed spacers
// pile up at the bottom of pages and create dead-zone empty pages. 1fr
// distributes whatever vertical space is left on the page evenly between
// questions. Use #pagebreak() between sections to control which questions
// land on which page; otherwise let Typst flow naturally.
//
// DO NOT use #v(Ncm) here. See feedback-onboarding.typ for the canonical
// 2-page example.
//
// ─── OQ-MONITOR PATTERN ──────────────────────────────────────────────────────
//
// Section C must contain one question per open monitoring OQ for this stack.
//
// HOW TO FIND MONITORING OQs FOR A STACK:
//   Open game-state/OPEN_QUESTIONS.md and search for:
//   - "TRACKING (Stack X)" — OQ explicitly scheduled for this stack
//   - "Evaluation criteria (Session N)" — OQ with a concrete observable to check
//   - "Monitor in Stack X" — OQ with a softer watch flag
//
// For each OQ found, map it to a Section C question:
//   OQ-10 (Injured Champion penalty) → "Do Injured Champions feel meaningfully
//     affected, or basically fine until they're killed?"
//   OQ-11 (Armor Cap) → "Did either player stack a lot of Armor? Did it feel
//     like Armor slowed the game down?"
//   OQ-34 (Steal dominance) → "Did any skill feel like a must-pick or
//     consistently dominant?" (ask INDIRECTLY — let player name it)
//   OQ-40 (Standoff) → "Were you reluctant to move pieces forward?"
//   OQ-41 (Game length vs. nerf) → captured by kill-round fields in Section A
//   OQ-46 (Money hoarding) → "Did you ever feel like you were sitting on Money
//     with nothing good to spend it on?"
//
// Keep the standard Section C skeleton (Skill Drafting, Turn Flow, Skills Balance,
// Bodyguard, Favorite Moment, Most Confusing Moment, rating rows) as the base.
// Insert OQ-monitoring questions between the skeleton questions as fq() blocks.
// Use a comment above each OQ question to identify which OQ it serves.
//
// ─────────────────────────────────────────────────────────────────────────────

#import "./template.typ": *
#show: template.with(title: "Playtest Feedback — [STACK: Stack name]")

#note-box[*TEMPLATE — not for player use.* Copy to `stack-X-feedback.typ` and fill in all `[STACK: ...]` placeholders before printing.]

= Playtest Feedback — [STACK: Stack name]

_One form per player. Fill out independently after the game._

// [STACK: Add a note-box here summarising what changed in this stack. Example:]
// #note-box[Champions and King have *3 HP* in this stack. Guards remain at 2 HP.]

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  [*Date:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Game length (rounds):* \_\_\_\_\_],
  [*Your name:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Playing as:* P1 / P2 _(circle one)_],
  [*Opponent:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [],
)

#hr

== A — Observational Data

_Reference your tracking sheet for round-by-round Money and skill data._

// [STACK: Add factual fields tied to what this stack is testing + standard kill
//  timing fields (always include these — they feed OQ-19, OQ-37, OQ-41 kill timing).]
//
//  #grid(
//    columns: (1fr, 1fr),
//    gutter: 8pt,
//    [*First Guard kill:* Round \_\_\_\_\_],
//    [*First Champion kill:* Round \_\_\_\_\_],
//    [*Total captures — you:* \_\_\_ / *opponent:* \_\_\_],
//    [*[Stack-specific observable]:* \_\_\_\_\_],
//    [*Total Armor granted — you:* \_\_\_ / *opponent:* \_\_\_],   // OQ-11
//    [],
//  )

#hr

== B — Stack Questions

// [STACK: 6–8 questions directly testing this stack's hypothesis.
//  Use fq("N")[...] for each. Number from 1.
//  Keep answers fast: Yes/No, scales, circle options, short blanks.
//  Focus ONLY on the change being tested — system-wide questions go in Section C.]

#hr

== C — Systems & Overall Feel

_These questions cover the full game — not just the stack being tested. Identical structure across all stacks; OQ-monitoring questions updated per stack._

// ── STANDARD SKELETON (keep in all stacks) ──────────────────────────────────

#fq("N")[
  *Skill Drafting:* Did the draft feel fair and engaging? Did you draft with specific skill pairings in mind — thinking about how skills could work together — or did you evaluate each skill individually for what it does on its own?\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

#fq("N")[
  *Turn flow:* Did Move Phase actions and Skill Phase actions feel intuitive to manage together?\
  Very intuitive — Mostly yes — Sometimes confusing — Often confusing
]
#v(1fr)

#fq("N")[
  *Skills — balance:* Any skill that felt too strong or too weak?\
  Too strong: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_ #h(8pt) Too weak: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\
  Best combo you pulled off: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

// OQ-34 (Steal dominance — ask INDIRECTLY, don't name the skill)
#fq("N")[
  *Must-pick skills:* Did any skill feel like everyone should always pick it — like skipping it would be a mistake?\
  Yes: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_ #h(8pt) No — Why: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

#fq("N")[
  *Bodyguard Rule:* Did it trigger? Did you actively reposition Guards to use it?\
  Triggered \_\_\_\_\_ times. #h(8pt) Repositioned Guards for it: Yes / No / Sometimes
]
#v(1fr)

// OQ-10 (Injured Champion penalty — do Injured Champions feel meaningfully punished?)
#fq("N")[
  *Injured Champions:* When a Champion was Injured (not a Guard), did it feel meaningfully weaker — or basically fine?\
  Clearly weaker — Slightly weaker — Barely noticeable — Not different at all\
  What changed for them? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

// OQ-11 (Armor Cap — did armor extend games or feel well-balanced?)
// OQ-11 Armor: ask about ATTENTIONAL COST, not just pacing.
// Add second line: "Did tracking Armor take focus away from planning skill combos?"
#fq("N")[
  *Armor:* Did either player stack a lot of Armor? Did Armor feel like it slowed the game down, or was it a fair tradeoff?\
  Slowed game noticeably — Slightly extended — Well balanced — Armor rarely mattered\
  _Did tracking and managing Armor take up a noticeable part of your mental focus — time you might otherwise have spent planning skill combos?_\
  Yes, a lot — Somewhat — Not really — No
]
#v(1fr)

// OQ-46 (Money hoarding — saving for a plan vs. nothing to spend on)
#fq("N")[
  *Money spending:* Did you ever sit on a lot of Money with nothing good to spend it on — or were you always wanting to spend more than you had?\
  Always more to spend than Money — Balanced — Sometimes nothing to spend on — Often sat on Money\
  If sitting on Money, why? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

// ── OQ-SPECIFIC QUESTIONS (update per stack) ────────────────────────────────
// [STACK: Replace this block with the monitoring question for the one carry-over
//  system most relevant to this specific stack. Examples:
//  "Did skill use feel frequent enough throughout the game?" (monitoring economy)
//  "Did pieces spend meaningful time Injured before dying?"   (monitoring HP gradient)
//  "Did Guards feel strategically useful beyond just blocking?" (monitoring Guard value)
//  Delete this comment and insert the actual fq() block.]

#fq("N")[
  *Favorite moment of the game — describe what happened:*\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

// ADR-004 (Framing B) — parallel puzzle signal. Keep in all stacks.
#fq("N")[
  *Opponent's game:* Did you feel like you and your opponent were both solving a similar kind of puzzle — figuring out the same game from opposite sides? Or more of a direct "stop them from winning" contest?\
  Same puzzle — Bit of both — Direct contest — Hard to say
]
#v(1fr)

#fq("N")[
  *Most confusing or frustrating rule or moment:*\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]
#v(1fr)

#rating-row("Game length:")[
  1 (way too short) — 2 (a bit short) — 3 (just right) — 4 (a bit long) — 5 (way too long)\
  Circle: *1 · 2 · 3 · 4 · 5*
]

// [STACK: Set a comparison rating relevant to this stack's change.
//  Label: name the dimension being compared. Examples:
//    Opening feel vs prior playtest:  /  Combat feel vs prior playtest:]
#rating-row("[STACK: Dimension vs prior playtest]:")[
  Much worse — Worse — Same — Better — Much better
]

#rating-row("Overall enjoyment:")[
  1 — 2 — 3 — 4 — 5
]

#hr

== D — Free Notes

_(Rules questions, edge cases, suggestions, anything else)_

#v(1fr)
