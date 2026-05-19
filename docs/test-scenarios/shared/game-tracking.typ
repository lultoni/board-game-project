// game-tracking.typ — per-player in-game tracking sheet
// Print one copy per player per game.
#import "./template.typ": *
#show: template.with(title: "Game Tracking Sheet")

= Game Tracking Sheet

_One sheet per player. Fill in as you play._

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 8pt,
  [*Date:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Your name:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Layer / Variant:* \_\_\_\_\_\_\_\_\_\_],
  [*Opponent:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Playing as:* P1 / P2 _(circle)_],
  [*Winner:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
)

#v(0.4em)
#hr
#v(0.4em)

== Round Log

_Runes: start + gain − spent = end. Skills: name only (e.g. "Lance, Blade Call"). Atk: number of Standard Attacks this turn._

#set text(size: 8.5pt)

#table(
  columns: (1.2em, 1.2em, 1.2em, 1.2em, 3.5cm, 1fr),
  table.header(
    [*R*],
    [*R+*],
    [*SS*],
    [*Atk*],
    [*Runes* (start + gain − spent = end)],
    [*Skills used / Events / Notes*],
  ),
  // R1
  [1],  [0],  [2],  [], [], [],
  // R2 — Rune gain changes to +2
  [2],  [+2], [|],  [], [], [],
  [3],  [|],  [|],  [], [], [],
  [4],  [|],  [|],  [], [], [],
  // R5 — Rune gain changes to +3
  [5],  [+3], [|],  [], [], [],
  [6],  [|],  [|],  [], [], [],
  [7],  [|],  [|],  [], [], [],
  [8],  [|],  [|],  [], [], [],
  [9],  [|],  [|],  [], [], [],
  // R10 — Rune gain changes to +4
  [10], [+4], [|],  [], [], [],
  // R11 — Skill Slots change to 3
  [11], [|],  [3],  [], [], [],
  [12], [|],  [|],  [], [], [],
  [13], [|],  [|],  [], [], [],
  [14], [|],  [|],  [], [], [],
  // R15 — Rune gain changes to +5
  [15], [+5], [|],  [], [], [],
  [16], [|],  [|],  [], [], [],
  [17], [|],  [|],  [], [], [],
  [18], [|],  [|],  [], [], [],
  [19], [|],  [|],  [], [], [],
  // R20 — Rune gain changes to +6
  [20], [+6], [|],  [], [], [],
  // R21 — Skill Slots change to 4
  [21], [|],  [4],  [], [], [],
  [22], [|],  [|],  [], [], [],
  [23], [|],  [|],  [], [], [],
  [24], [|],  [|],  [], [], [],
  // R25 — Rune gain changes to +7
  [25], [+7], [|],  [], [], [],
  [26], [|],  [|],  [], [], [],
  [27], [|],  [|],  [], [], [],
  [28], [|],  [|],  [], [], [],
  [29], [|],  [|],  [], [], [],
  // R30 — Rune gain changes to +8
  [30], [+8], [|],  [], [], [],
  // R31 — Skill Slots change to 5
  [31], [|],  [5],  [], [], [],
  [32], [|],  [|],  [], [], [],
  [33], [|],  [|],  [], [], [],
  [34], [|],  [|],  [], [], [],
  // R35 — Rune gain changes to +9
  [35+],[+9], [|],  [], [], [],
)
