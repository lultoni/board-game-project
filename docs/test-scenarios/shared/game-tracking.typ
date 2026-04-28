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

_Runes: start + gain − spent = end. Skills: name · cost (e.g. "Lance · 2, Blade Call · 2"). Events/Notes: captures, combos, key moments — anything worth remembering._

#set text(size: 8.5pt)

#table(
  columns: (1.2em, 3.5cm, 4cm, 1fr),
  table.header(
    [*R*],
    [*Runes* (start + gain − spent = end)],
    [*Skills used* (name · cost)],
    [*Events / Notes*],
  ),
  [1], [], [], [],
  [2], [], [], [],
  [3], [], [], [],
  [4], [], [], [],
  [5], [], [], [],
  [6], [], [], [],
  [7], [], [], [],
  [8], [], [], [],
  [9], [], [], [],
  [10], [], [], [],
  [11], [], [], [],
  [12], [], [], [],
  [13], [], [], [],
  [14], [], [], [],
  [15], [], [], [],
  [16], [], [], [],
  [17], [], [], [],
  [18], [], [], [],
  [19], [], [], [],
  [20], [], [], [],
  [21], [], [], [],
  [22], [], [], [],
  [23], [], [], [],
  [24], [], [], [],
  [25], [], [], [],
  [26], [], [], [],
  [27], [], [], [],
  [28], [], [], [],
  [29], [], [], [],
  [30], [], [], [],
  [31], [], [], [],
  [32], [], [], [],
  [33], [], [], [],
  [34], [], [], [],
  [35+], [], [], [],
)
