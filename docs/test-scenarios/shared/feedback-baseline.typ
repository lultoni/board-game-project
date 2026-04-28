// feedback-baseline.typ — base template for all playtest feedback forms
// HOW TO USE:
//   1. Copy this file to layer-N-<desc>/layer-N-feedback.typ
//   2. Replace every line marked with [LAYER: ...] with layer-specific content
//   3. Section C follows the OQ-monitor pattern below — update it per layer
//   4. Do NOT change the Section D free notes block
//   5. Run build-pdfs.sh to compile
//
// ─── OQ-MONITOR PATTERN ──────────────────────────────────────────────────────
//
// Section C must contain one question per open monitoring OQ for this layer.
//
// HOW TO FIND MONITORING OQs FOR A LAYER:
//   Open game-state/OPEN_QUESTIONS.md and search for:
//   - "TRACKING (Layer N)" — OQ explicitly scheduled for this layer
//   - "Evaluation criteria (Session X)" — OQ with a concrete observable to check
//   - "Monitor in Layer N" — OQ with a softer watch flag
//
// For each OQ found, map it to a Section C question:
//   OQ-10 (Injured Champion penalty) → "Do Injured Champions feel meaningfully
//     affected, or basically fine until they're killed?"
//   OQ-11 (Armor Cap) → "Did either player stack a lot of Armor? Did it feel
//     like Armor slowed the game down?"
//   OQ-34 (Rune Theft dominance) → "Did any skill feel like a must-pick or
//     consistently dominant?" (ask INDIRECTLY — let player name it)
//   OQ-40 (Standoff) → "Were you reluctant to move pieces forward?"
//   OQ-41 (Game length vs. nerf) → captured by kill-round fields in Section A
//   OQ-46 (Rune hoarding) → "Did you ever feel like you were sitting on Runes
//     with nothing good to spend them on?"
//
// Keep the standard Section C skeleton (Skill Drafting, Turn Flow, Skills Balance,
// Bodyguard, Favorite Moment, Most Confusing Moment, rating rows) as the base.
// Insert OQ-monitoring questions between the skeleton questions as fq() blocks.
// Use a comment above each OQ question to identify which OQ it serves.
//
// ─────────────────────────────────────────────────────────────────────────────

#import "./template.typ": *
#show: template.with(title: "Playtest Feedback — [LAYER: Layer name]")

#note-box[*TEMPLATE — not for player use.* Copy to `layer-N-feedback.typ` and fill in all `[LAYER: ...]` placeholders before printing.]

= Playtest Feedback — [LAYER: Layer name]

_One form per player. Fill out independently after the game._

// [LAYER: Add a note-box here summarising what changed in this layer. Example:]
// #note-box[Champions and King have *3 HP* in this layer. Guards remain at 2 HP.]

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

_Reference your tracking sheet for round-by-round Rune and skill data._

// [LAYER: Add factual fields tied to what this layer is testing + standard kill
//  timing fields (always include these — they feed OQ-19, OQ-37, OQ-41 kill timing).]
//
//  #grid(
//    columns: (1fr, 1fr),
//    gutter: 8pt,
//    [*First Guard kill:* Round \_\_\_\_\_],
//    [*First Champion kill:* Round \_\_\_\_\_],
//    [*Total captures — you:* \_\_\_ / *opponent:* \_\_\_],
//    [*[Layer-specific observable]:* \_\_\_\_\_],
//    [*Total Armor granted — you:* \_\_\_ / *opponent:* \_\_\_],   // OQ-11
//    [],
//  )

#hr

== B — Layer Questions

// [LAYER: 6–8 questions directly testing this layer's hypothesis.
//  Use fq("N")[...] for each. Number from 1.
//  Keep answers fast: Yes/No, scales, circle options, short blanks.
//  Focus ONLY on the change being tested — system-wide questions go in Section C.]

#hr

== C — Systems & Overall Feel

_These questions cover the full game — not just the layer being tested. Identical structure across all layers; OQ-monitoring questions updated per layer._

// ── STANDARD SKELETON (keep in all layers) ──────────────────────────────────

#fq("N")[
  *Skill Drafting:* Did the draft feel fair and engaging? Did you have a clear plan going in, or did you just pick what seemed good?\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("N")[
  *Turn flow:* Did Move Slots and Skill Slots feel intuitive to manage together?\
  Very intuitive — Mostly yes — Sometimes confusing — Often confusing
]

#fq("N")[
  *Skills — balance:* Any skill that felt too strong or too weak?\
  Too strong: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_ #h(8pt) Too weak: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\
  Best combo you pulled off: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

// OQ-34 (Rune Theft dominance — ask INDIRECTLY, don't name the skill)
#fq("N")[
  *Must-pick skills:* Did any skill feel like everyone should always pick it — like skipping it would be a mistake?\
  Yes: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_ #h(8pt) No — Why: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("N")[
  *Bodyguard Rule:* Did it trigger? Did you actively reposition Guards to use it?\
  Triggered \_\_\_\_\_ times. #h(8pt) Repositioned Guards for it: Yes / No / Sometimes
]

// OQ-10 (Injured Champion penalty — do Injured Champions feel meaningfully punished?)
#fq("N")[
  *Injured Champions:* When a Champion was Injured (not a Guard), did it feel meaningfully weaker — or basically fine?\
  Clearly weaker — Slightly weaker — Barely noticeable — Not different at all\
  What changed for them? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

// OQ-11 (Armor Cap — did armor extend games or feel well-balanced?)
#fq("N")[
  *Armor:* Did either player stack a lot of Armor? Did Armor feel like it slowed the game down, or was it a fair tradeoff?\
  Slowed game noticeably — Slightly extended — Well balanced — Armor rarely mattered
]

// OQ-46 (Rune hoarding — saving for a plan vs. nothing to spend on)
#fq("N")[
  *Rune spending:* Did you ever sit on a lot of Runes with nothing good to spend them on — or were you always wanting to spend more than you had?\
  Always more to spend than Runes — Balanced — Sometimes nothing to spend on — Often sat on Runes\
  If sitting on Runes, why? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

// ── OQ-SPECIFIC QUESTIONS (update per layer) ────────────────────────────────
// [LAYER: Replace this block with the monitoring question for the one carry-over
//  system most relevant to this specific layer. Examples:
//  "Did skill use feel frequent enough throughout the game?" (monitoring economy)
//  "Did pieces spend meaningful time Injured before dying?"   (monitoring HP gradient)
//  "Did Guards feel strategically useful beyond just blocking?" (monitoring Guard value)
//  Delete this comment and insert the actual fq() block.]

#fq("N")[
  *Favorite moment of the game:*\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("N")[
  *Most confusing or frustrating rule or moment:*\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#rating-row("Game length:")[
  1 (way too short) — 2 (a bit short) — 3 (just right) — 4 (a bit long) — 5 (way too long)\
  Circle: *1 · 2 · 3 · 4 · 5*
]

// [LAYER: Set a comparison rating relevant to this layer's change.
//  Label: name the dimension being compared. Examples:
//    Opening feel vs prior playtest:  /  Combat feel vs prior playtest:]
#rating-row("[LAYER: Dimension vs prior playtest]:")[
  Much worse — Worse — Same — Better — Much better
]

#rating-row("Overall enjoyment:")[
  1 — 2 — 3 — 4 — 5
]

#hr

== D — Free Notes

_(Rules questions, edge cases, suggestions, anything else)_

#v(1fr)
