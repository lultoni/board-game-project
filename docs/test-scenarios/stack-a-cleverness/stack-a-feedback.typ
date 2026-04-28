#import "../shared/template.typ": *
#show: template.with(title: "Playtest Feedback — Layer 2: Standard Attack Nerf + Combo Bonus")

= Playtest Feedback — Layer 2: Standard Attack Nerf + Combo Bonus

_One form per player. Fill out after *both* games. Compare Game 1 (nerf only) vs Game 2 (nerf + combo)._

#note-box[*Game 1:* Standard Attacks deal 1 DMG (was 2). *Game 2:* Same, plus multi-Champion combo bonus (+1 DMG on 2nd Champion Strike to same target).]

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  [*Date:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Game 1 length (rounds):* \_\_\_\_\_ #h(6pt) *Game 2:* \_\_\_\_\_],
  [*Your name:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Playing as:* P1 / P2 _(circle one per game)_],
  [*Opponent:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [],
)

#hr

== A — Observational Data

_Reference your tracking sheets for both games._

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  [*Game 1 — first Guard kill:* Round \_\_\_\_\_],
  [*Game 2 — first Guard kill:* Round \_\_\_\_\_],
  [*Game 1 — first Champion kill:* Round \_\_\_\_\_],
  [*Game 2 — first Champion kill:* Round \_\_\_\_\_],
  [*Game 1 — total captures (you/opp):* \_\_/\_\_],
  [*Game 2 — total captures (you/opp):* \_\_/\_\_],
  [*Game 1 — total Armor granted (you/opp):* \_\_/\_\_],
  [*Game 2 — total Armor granted (you/opp):* \_\_/\_\_],
)

#hr

== B — Layer Questions: Game 1 (Nerf Only)

#fq("1")[
  *Did pieces spend meaningful time Injured before dying?*\
  Almost always — Often — Sometimes — Rarely — Never
]

#fq("2")[
  *Was it frustrating that Standard Attacks only Injure (not kill)?*\
  Very frustrating — A bit frustrating — Neutral — Felt right — Very satisfying
]

#fq("3")[
  *Did you use Strike skills to finish off Injured pieces?*\
  Constantly — Often — Sometimes — Rarely — Never\
  Which skills? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("4")[
  *Did the no-man's-land / standoff feel different from Playtest 2?*\
  Much less standoff — Somewhat less — About the same — Worse standoff
]

#fq("5")[
  *Were you reluctant to move pieces forward? Why?*\
  Not reluctant — Somewhat — Very reluctant\
  Reason: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#hr

== B2 — Layer Questions: Game 2 (Nerf + Combo Bonus)

#fq("6")[
  *Did you attempt multi-Champion combos?* How many succeeded?\
  Attempted: \_\_\_\_\_ times. #h(8pt) Succeeded: \_\_\_\_\_ times.\
  What blocked the failed attempts? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("7")[
  *Did the combo bonus feel like a meaningful reward for coordination?*\
  Very rewarding — Somewhat — Neutral — Barely noticeable — Never triggered
]

#fq("8")[
  *Did you change how you positioned Champions compared to Game 1?*\
  Yes, significantly — Somewhat — Not really — No\
  How? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("9")[
  *Comparing Game 1 vs Game 2 — which felt better?*\
  Game 1 much better — Game 1 slightly better — About the same — Game 2 slightly better — Game 2 much better\
  Why? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("10")[
  *Did the combo bonus make any skill or combination feel broken?*\
  No — Maybe: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#hr

== C — Systems & Overall Feel

_These questions cover the full game — not just the layer being tested._

#fq("11")[
  *Skill Drafting:* Did the draft feel fair and engaging? Did you have a clear plan going in, or did you just pick what seemed good?\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("12")[
  *Turn flow:* Did Move Slots and Skill Slots feel intuitive to manage together?\
  Very intuitive — Mostly yes — Sometimes confusing — Often confusing
]

#fq("13")[
  *Skills — balance:* Any skill that felt too strong or too weak?\
  Too strong: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_ #h(8pt) Too weak: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\
  Best combo you pulled off: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

// OQ-34: Ask indirectly — let player name Rune Theft if dominant, don't lead them
#fq("14")[
  *Must-pick skills:* Did any skill feel like everyone should always pick it — like skipping it would be a mistake?\
  Yes: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_ #h(8pt) No — Why: \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("15")[
  *Bodyguard Rule:* Did it trigger? Did you actively reposition Guards to use it?\
  Triggered \_\_\_\_\_ times. #h(8pt) Repositioned Guards for it: Yes / No / Sometimes
]

// OQ-10: Injured Champion penalty — do Injured Champions feel meaningfully weaker?
#fq("16")[
  *Injured Champions:* When a Champion (not a Guard) was Injured, did it feel meaningfully weaker — or basically fine until killed?\
  Clearly weaker — Slightly weaker — Barely noticeable — Not different at all\
  What changed for them? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

// OQ-11: Armor cap — did armor extend games or feel well-balanced?
#fq("17")[
  *Armor:* Did either player stack a lot of Armor? Did Armor feel like it slowed the game down, or was it a fair tradeoff?\
  Slowed game noticeably — Slightly extended — Well balanced — Armor rarely mattered
]

// OQ-46: Rune hoarding — saving for a plan vs. nothing to spend on vs. always wanting more
#fq("18")[
  *Rune spending:* Did you ever sit on a lot of Runes with nothing good to spend them on — or were you always wanting more?\
  Always wanted more — Balanced — Sometimes nothing to spend on — Often sat on Runes\
  If sitting on Runes, why? \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("19")[
  *Did Rune economy feel different with 1-DMG attacks?* (More pressure to spend on skills?)\
  Much more spending — Somewhat more — About the same — Less spending\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("20")[
  *Favorite moment across both games:*\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#fq("21")[
  *Most confusing or frustrating rule or moment:*\
  \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
]

#rating-row("Game 1 length:")[
  1 (way too short) — 2 (a bit short) — 3 (just right) — 4 (a bit long) — 5 (way too long)\
  Circle: *1 · 2 · 3 · 4 · 5*
]

#rating-row("Game 2 length:")[
  1 (way too short) — 2 (a bit short) — 3 (just right) — 4 (a bit long) — 5 (way too long)\
  Circle: *1 · 2 · 3 · 4 · 5*
]

#rating-row("Combat feel vs Playtest 2:")[
  Much worse — Worse — Same — Better — Much better
]

#rating-row("Overall enjoyment:")[
  1 — 2 — 3 — 4 — 5
]

#hr

== D — Free Notes

_(Rules questions, edge cases, suggestions, anything else)_

#v(1fr)
