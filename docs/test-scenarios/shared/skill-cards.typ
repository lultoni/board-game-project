// skill-cards.typ — printable physical reference cards for all skills
// Layout: 3 columns × 5 rows = 15 cards on one A4 page.
// Cards are color-coded by category: Strike (red), Shield (blue), Move (green), Mystic (purple).
// Each card: large icon + name + cost • effect text (left) • 2×2 range matrix (right).
// The 2×2 matrix shows Default / +Focus / Injured / Injured+Focus, each as a tile grid
// centered on the caster with reachable tiles highlighted.
//
// Print recommendation: print twice for two-set play (one per player).
// Cuts: along the page grid lines. Card size: ~63mm × 55mm.
//
// SOURCE OF TRUTH: skill data must match `baseline-sections.typ` skill table.
// If a skill's cost/text changes there, update it here too.

#set document(title: "Skill Cards")
#set page(
  paper: "a4",
  margin: (x: 1cm, y: 1cm),
)
#set text(font: "Helvetica Neue", size: 9pt)

// ── CATEGORY COLORS ─────────────────────────────────────────────────────────
#let cat-color(cat) = {
  if cat == "Strike" { rgb("#c44") }
  else if cat == "Shield" { rgb("#46a") }
  else if cat == "Move" { rgb("#4a6") }
  else if cat == "Mystic" { rgb("#86c") }
  else { black }
}

#let cat-bg(cat) = {
  if cat == "Strike" { rgb("#fde6e6") }
  else if cat == "Shield" { rgb("#e6edf6") }
  else if cat == "Move" { rgb("#e6f3e9") }
  else if cat == "Mystic" { rgb("#efe9f6") }
  else { rgb("#f5f5f5") }
}

#let icon-name(name) = {
  if name == "Lance" { "lance_thrust" }
  else if name == "Hook" { "hook_pull" }
  else if name == "Break" { "armor_breaker" }
  else if name == "Steal" { "rune_theft" }
  else if name == "Tempest" { "blade_tempest" }
  else if name == "Shield" { "rust_shield" }
  else if name == "Heal" { "field_medic" }
  else if name == "Plate" { "armor_smith" }
  else if name == "Dash" { "quick_dash" }
  else if name == "Blast" { "air_blast" }
  else if name == "Shove" { "precision_thrust" }
  else if name == "Swap" { "shadow_shift" }
  else if name == "Retreat" { "retreat_plan" }
  else if name == "Focus" { "focus_strike" }
  else if name == "Charge" { "blade_call" }
}

// ── RANGE GRID ──────────────────────────────────────────────────────────────
// Draws a (2*radius+1) × (2*radius+1) grid centered on the caster.
// `range`: queen-line reach in tiles past the caster (-1 = cannot fire,
//          0 = self only, 1 = adjacent, 2+ = standard).
// `radius`: half-width of the grid in tiles (so a radius-3 grid is 7×7).
// `color`: category color used for reachable tile fill.
// `cell-size`: edge length of each tile.
//
// We use a single `table` with an explicit grid stroke so the borders are
// drawn once at the table level, not once-per-cell — that fixes the
// "tile-to-tile border overlap" problem (where adjacent fills painted
// over each other's strokes).
#let range-grid(range, radius: 3, color: black, cell-size: 2.4mm, can-fire: true) = {
  let n = radius * 2 + 1
  let solid-fill = color.lighten(55%)
  let blank-fill = white
  let caster-fill = black

  let cells = ()
  let dy = -radius
  while dy <= radius {
    let dx = -radius
    while dx <= radius {
      let on-queen-line = (dx == 0) or (dy == 0) or (calc.abs(dx) == calc.abs(dy))
      let r = calc.max(calc.abs(dx), calc.abs(dy))
      let is-caster = (dx == 0 and dy == 0)

      let fill = if not can-fire {
        // Whole grid is greyed out — caster cannot cast.
        if is-caster { rgb("#666") } else { rgb("#f4f4f4") }
      } else if is-caster {
        caster-fill
      } else if on-queen-line and r <= range {
        solid-fill
      } else {
        blank-fill
      }

      let content = if is-caster and can-fire {
        align(center + horizon, text(size: 4.5pt, fill: white, weight: "bold", "C"))
      } else if is-caster and not can-fire {
        align(center + horizon, text(size: 4.5pt, fill: white, weight: "bold", "✕"))
      } else { [] }

      cells.push(rect(width: 100%, height: 100%, fill: fill, stroke: none, content))
      dx += 1
    }
    dy += 1
  }

  table(
    columns: (cell-size,) * n,
    rows: (cell-size,) * n,
    stroke: 0.3pt + rgb("#888"),
    inset: 0pt,
    ..cells
  )
}

// ── RANGE MATRIX (2×2) ──────────────────────────────────────────────────────
// Returns the four (range, can-fire) pairs in order:
//   [Default, +Focus, Injured, Injured+Focus]
// based on a `kind` string. The grid `radius` is sized so that the largest
// reach in any cell still fits on-grid.
#let kind-data(kind) = {
  if kind == "self" {
    // Self skills are immune to Injured and to Focus Range buffs.
    // (Per baseline: Range buffs do not collapse Self skills inward, but
    //  Focus DOES extend a Self skill: "Range 0 → 1" per baseline
    //  Quick Reference. So +Focus on Self → Range 1.)
    (
      radius: 1,
      cells: (
        ("Default",     0, true),
        ("+ Focus",     1, true),
        ("Injured",     0, true),
        ("Inj. +Focus", 1, true),
      ),
    )
  } else if kind == "adjacent" {
    // Adjacent skills: Injured does not affect them. Focus extends Range 1 → 2.
    (
      radius: 2,
      cells: (
        ("Default",     1, true),
        ("+ Focus",     2, true),
        ("Injured",     1, true),
        ("Inj. +Focus", 2, true),
      ),
    )
  } else if kind == "default" {
    // Standard Range 2 skill. +Focus → 3. Injured → 1. Inj+Focus → 2.
    (
      radius: 3,
      cells: (
        ("Default",     2, true),
        ("+ Focus",     3, true),
        ("Injured",     1, true),
        ("Inj. +Focus", 2, true),
      ),
    )
  } else if kind == "minus1" {
    // Lance: default Range 2 with -1 modifier → effective 1.
    // +Focus → 2. Injured → effective 0 (self), but the skill targets others
    // and does not name "self" in its text, so it cannot fire while Injured.
    // Inj+Focus → effective 1 (back in range).
    (
      radius: 2,
      cells: (
        ("Default",     1, true),
        ("+ Focus",     2, true),
        ("Injured",     0, false),  // cannot fire
        ("Inj. +Focus", 1, true),
      ),
    )
  } else if kind == "plus1" {
    // Shove: default Range 3, +Focus → 4, Injured → 2, Inj+Focus → 3.
    (
      radius: 4,
      cells: (
        ("Default",     3, true),
        ("+ Focus",     4, true),
        ("Injured",     2, true),
        ("Inj. +Focus", 3, true),
      ),
    )
  } else if kind == "plus1-self" {
    // Retreat: Self with Range+1 path. Self skill so Injured doesn't apply.
    // The "+1" governs how far the path can extend. We render the grid based
    // on path length (default 3, +Focus 4 — same as plus1).
    (
      radius: 4,
      cells: (
        ("Default",     3, true),
        ("+ Focus",     4, true),
        ("Injured",     3, true),  // Self skill — unaffected by Injured
        ("Inj. +Focus", 4, true),
      ),
    )
  }
}

#let range-matrix(kind, color) = {
  if kind == "mystic" {
    align(center + horizon)[
      #text(size: 6.5pt, fill: rgb("#666"), style: "italic")[
        Affects your turn —\
        no target range.
      ]
    ]
  } else {
    let data = kind-data(kind)
    let rad = data.radius
    // Pick a cell size so the four mini-grids fit comfortably in the right
    // half of a 63mm-wide card. Right column is roughly ~28mm wide; with
    // a 2×2 matrix and 2mm gutter, each mini-grid gets ~13mm. Divide by
    // (2*radius+1) tiles.
    let mini-grid-width = 13mm
    let cell-sz = mini-grid-width / (rad * 2 + 1)

    let cells = data.cells.map(((label, range, can-fire)) => {
      stack(
        spacing: 0.3mm,
        range-grid(range, radius: rad, color: color, cell-size: cell-sz, can-fire: can-fire),
        align(center, text(size: 5pt, fill: if can-fire { rgb("#444") } else { rgb("#a44") },
                           weight: if can-fire { "regular" } else { "bold" },
                           label)),
      )
    })

    grid(
      columns: (1fr, 1fr),
      rows: (auto, auto),
      gutter: 1.2mm,
      ..cells
    )
  }
}

// ── CARD ────────────────────────────────────────────────────────────────────
#let card(name, category, cost, effect, range-kind, focus-note: none) = {
  block(
    width: 100%,
    height: 100%,
    fill: cat-bg(category),
    stroke: 1pt + cat-color(category),
    inset: 2.2mm,
    breakable: false,
  )[
    // Header: big icon + name + big cost circle
    #grid(
      columns: (auto, 1fr, auto),
      gutter: 2mm,
      align: (left + horizon, left + horizon, right + horizon),
      box(height: 9mm, image("../../../images/" + icon-name(name) + ".jpg", height: 9mm)),
      text(size: 10.5pt, weight: "bold", fill: cat-color(category), name),
      box(
        width: 8mm, height: 8mm,
        radius: 4mm,
        fill: cat-color(category),
        stroke: none,
        align(center + horizon, text(size: 11pt, fill: white, weight: "bold", str(cost))),
      ),
    )
    #v(1mm)
    #line(length: 100%, stroke: 0.4pt + cat-color(category))
    #v(1mm)
    // Body: text on the left, range matrix on the right
    #grid(
      columns: (1fr, auto),
      gutter: 2mm,
      align: (left + top, center + top),
      stack(
        spacing: 1.5mm,
        text(size: 7pt, effect),
        if focus-note != none {
          text(size: 5.5pt, fill: rgb("#444"), style: "italic", focus-note)
        },
      ),
      box(width: 28mm, range-matrix(range-kind, cat-color(category))),
    )
  ]
}

// ── PAGE 1: 3×5 GRID ────────────────────────────────────────────────────────
#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (1fr, 1fr, 1fr, 1fr, 1fr),
  gutter: 2mm,

  // Row 1 — Strike
  card("Lance",   "Strike", 2, [Target within Range −1 takes 1 damage.], "minus1"),
  card("Hook",    "Strike", 3, [Target takes 1 damage, pulled 1 tile toward caster along the Path.], "default"),
  card("Break",   "Strike", 2, [Remove 1 Armor from target. _(No HP-damage unless boosted by Charge.)_], "default"),

  // Row 2 — Strike + Shield
  card("Steal",   "Strike", 3, [Target takes 1 damage. Steal 1 Money from opponent.], "default"),
  card("Tempest", "Strike", 4, [Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away. Caster unaffected.], "default"),
  card("Shield",  "Shield", 2, [Self: gain +1 Armor.], "self"),

  // Row 3 — Shield + Move
  card("Heal",    "Shield", 3, [Remove Injured from one adjacent ally.], "adjacent"),
  card("Plate",   "Shield", 3, [Adjacent ally gains +1 Armor.], "adjacent"),
  card("Dash",    "Move",   3, [Self: move up to 2 tiles along the Path.], "self",
    focus-note: "+Focus: cast on adjacent ally, or move 3 tiles instead of 2."),

  // Row 4 — Move
  card("Blast",   "Move",   2, [Push target enemy 1 tile directly away from caster.], "default",
    focus-note: "+Focus: cast at Range 3, or push 2 tiles instead of 1."),
  card("Shove",   "Move",   3, [Push target enemy 1 tile in any direction (caster chooses). *Range +1.*], "plus1",
    focus-note: "+Focus: cast at Range 4, or push 2 tiles instead of 1."),
  card("Swap",    "Move",   4, [Swap position with an allied piece. Requires unobstructed Path.], "default",
    focus-note: "+Focus: swap with ally up to Range 3 (no effect-range option)."),

  // Row 5 — Move + Mystic
  card("Retreat", "Move",   4, [Self: move along the Path to land adjacent to one of your Guards. *Range +1.*], "plus1-self",
    focus-note: "+Focus: path length 4 instead of 3."),
  card("Focus",   "Mystic", 1, [The next skill used by *any of your pieces* this turn gains +1 Range.], "mystic"),
  card("Charge",  "Mystic", 3, [One Strike skill used by *any of your pieces* this turn deals +1 damage.], "mystic"),
)

// ── LEGEND PAGE ─────────────────────────────────────────────────────────────
#pagebreak()

#align(center)[
  #text(size: 14pt, weight: "bold", "Skill Card Reference")
  #v(2mm)
  #text(size: 9pt, fill: rgb("#666"), [How to read each card])
]

#v(6mm)

#text(size: 11pt, weight: "bold", "Card colors mark the skill category")
#v(2mm)

#grid(
  columns: (auto, 1fr),
  gutter: 4mm,
  row-gutter: 2mm,

  box(width: 8mm, height: 4mm, fill: cat-bg("Strike"), stroke: 0.5pt + cat-color("Strike")),
  [*Strike* — damages enemy pieces.],

  box(width: 8mm, height: 4mm, fill: cat-bg("Shield"), stroke: 0.5pt + cat-color("Shield")),
  [*Shield* — protects or heals allies.],

  box(width: 8mm, height: 4mm, fill: cat-bg("Move"), stroke: 0.5pt + cat-color("Move")),
  [*Move* — repositions pieces. Movement-via-skill deals no damage.],

  box(width: 8mm, height: 4mm, fill: cat-bg("Mystic"), stroke: 0.5pt + cat-color("Mystic")),
  [*Mystic* — buffs one of your other skills this turn.],
)

#v(6mm)

#text(size: 11pt, weight: "bold", "The 2×2 range matrix")
#v(2mm)

The four mini-grids on each card show the skill's reach in four states. Each grid is centered on the *caster* and shows every tile the skill could target along a queen-line (horizontal, vertical, or diagonal). Paths are blocked by any piece in the way.

#v(3mm)

#grid(
  columns: (auto, 1fr),
  gutter: 4mm,
  row-gutter: 3mm,

  [*Default*],     [Full health, no buffs active.],
  [*+ Focus*],     [Caster's side has activated Focus (+1 Range to next skill).],
  [*Injured*],     [Caster is Injured (Range −1). Self and Adjacent skills are unaffected.],
  [*Inj. +Focus*], [Both effects combined.],
)

#v(4mm)

#text(size: 9pt, fill: rgb("#444"))[
  *Move skills + Focus:* On Move skills, the caster chooses, when activating the skill, whether the +1 applies to the *activation range* (how far the skill can target) or the *effect range* (how far it moves/pushes). Not both. The matrix on each Move card shows the activation-range outcome; the italic note under the effect text spells out both options for that specific skill.
]

#v(6mm)

#text(size: 11pt, weight: "bold", "Tile colors")
#v(2mm)

#grid(
  columns: (auto, 1fr),
  gutter: 4mm,
  row-gutter: 3mm,

  box(width: 6mm, height: 6mm, fill: black, stroke: 0.3pt + rgb("#888"), align(center + horizon, text(size: 6pt, fill: white, weight: "bold", "C"))),
  [*Caster.* The piece using the skill.],

  box(width: 6mm, height: 6mm, fill: rgb("#c44").lighten(55%), stroke: 0.3pt + rgb("#888")),
  [*Reachable tile* (color matches the card category).],

  box(width: 6mm, height: 6mm, fill: white, stroke: 0.3pt + rgb("#888")),
  [*Not reachable* — off the queen-line or out of range.],

  box(width: 6mm, height: 6mm, fill: rgb("#666"), stroke: 0.3pt + rgb("#888"), align(center + horizon, text(size: 6pt, fill: white, weight: "bold", "✕"))),
  [*Cannot fire.* The whole grid is greyed out — the skill cannot be cast in that state. (Lance while Injured.)],
)
