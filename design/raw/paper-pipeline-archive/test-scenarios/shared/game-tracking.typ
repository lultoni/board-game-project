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
  [*Stack / Variant:* \_\_\_\_\_\_\_\_\_\_],
  [*Opponent:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Playing as:* P1 / P2 _(circle)_],
  [*Winner:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
)

#v(0.4em)
#hr
#v(0.4em)

== Round Log

_Money: start + gain − spent = end. Skills: name only (e.g. "Lance, Charge"). Atk: number of Move-Attacks this turn._

#set text(size: 8.5pt)

#let grey = text.with(fill: rgb("#aaaaaa"))

#table(
  columns: (auto, auto, auto, auto, 4.5cm, 1fr),
  table.header(
    [*R+*],
    [*Act*],
    [*R*],
    [*Atk*],
    [*Money* (start + gain − spent = end)],
    [*Skills used / Events / Notes*],
  ),
  // R1
  grey[0],  grey[2],  [1],  [], [], [],
  // R2 — Money gain changes to +2
  grey[+2], grey[|],  [2],  [], [], [],
  grey[|],  grey[|],  [3],  [], [], [],
  grey[|],  grey[|],  [4],  [], [], [],
  // R5 — Money gain changes to +3
  grey[+3], grey[|],  [5],  [], [], [],
  grey[|],  grey[|],  [6],  [], [], [],
  grey[|],  grey[|],  [7],  [], [], [],
  grey[|],  grey[|],  [8],  [], [], [],
  grey[|],  grey[|],  [9],  [], [], [],
  // R10 — Money gain changes to +4
  grey[+4], grey[|],  [10], [], [], [],
  // R11 — Skill Phase actions change to 3
  grey[|],  grey[3],  [11], [], [], [],
  grey[|],  grey[|],  [12], [], [], [],
  grey[|],  grey[|],  [13], [], [], [],
  grey[|],  grey[|],  [14], [], [], [],
  // R15 — Money gain changes to +5
  grey[+5], grey[|],  [15], [], [], [],
  grey[|],  grey[|],  [16], [], [], [],
  grey[|],  grey[|],  [17], [], [], [],
  grey[|],  grey[|],  [18], [], [], [],
  grey[|],  grey[|],  [19], [], [], [],
  // R20 — Money gain changes to +6
  grey[+6], grey[|],  [20], [], [], [],
  // R21 — Skill Phase actions change to 4
  grey[|],  grey[4],  [21], [], [], [],
  grey[|],  grey[|],  [22], [], [], [],
  grey[|],  grey[|],  [23], [], [], [],
  grey[|],  grey[|],  [24], [], [], [],
  // R25 — Money gain changes to +7
  grey[+7], grey[|],  [25], [], [], [],
  grey[|],  grey[|],  [26], [], [], [],
  grey[|],  grey[|],  [27], [], [], [],
  grey[|],  grey[|],  [28], [], [], [],
  grey[|],  grey[|],  [29], [], [], [],
  // R30 — Money gain changes to +8
  grey[+8], grey[|],  [30], [], [], [],
  // R31 — Skill Phase actions change to 5
  grey[|],  grey[5],  [31], [], [], [],
  grey[|],  grey[|],  [32], [], [], [],
  grey[|],  grey[|],  [33], [], [], [],
  grey[|],  grey[|],  [34], [], [], [],
  // R35 — Money gain changes to +9
  grey[+9], grey[|],  [35+], [], [], [],
)
