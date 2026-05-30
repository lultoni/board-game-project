// template.typ — shared design template for all scenario rule sheets
// Import with: #import "/path/to/template.typ": *

#let accent = rgb("#0d5b6e")
#let ink = rgb("#0a0d12")
#let dim = rgb("#5a6470")
#let muted = rgb("#7a8694")
#let rule-color = rgb("#cfd6dd")
#let strong-rule = rgb("#1f2933")

// Skill category colors — used by sk() chip helper and any in-text references.
#let cat-color(cat) = {
  let c = lower(cat)
  if c == "strike" { rgb("#c8412c") }
  else if c == "shield" { rgb("#2563a8") }
  else if c == "move" { rgb("#1f8a5c") }
  else if c == "mystic" { rgb("#7a3da8") }
  else { ink }
}

#let cat-bg(cat) = {
  let c = lower(cat)
  if c == "strike" { rgb("#fdeae5") }
  else if c == "shield" { rgb("#e3edf7") }
  else if c == "move" { rgb("#dcf0e6") }
  else if c == "mystic" { rgb("#eddcf4") }
  else { rgb("#eeeeee") }
}

// Skill registry — name → (category, icon-filename in /images/)
#let skill-registry = (
  "Lance":   ("Strike", "lance_thrust"),
  "Hook":    ("Strike", "hook_pull"),
  "Break":   ("Strike", "armor_breaker"),
  "Steal":   ("Strike", "rune_theft"),
  "Tempest": ("Strike", "blade_tempest"),
  "Shield":  ("Shield", "rust_shield"),
  "Heal":    ("Shield", "field_medic"),
  "Plate":   ("Shield", "armor_smith"),
  "Dash":    ("Move",   "quick_dash"),
  "Blast":   ("Move",   "air_blast"),
  "Shove":   ("Move",   "precision_thrust"),
  "Swap":    ("Move",   "shadow_shift"),
  "Retreat": ("Move",   "retreat_plan"),
  "Focus":   ("Mystic", "focus_strike"),
  "Charge":  ("Mystic", "blade_call"),
)

// In-text skill chip — light tinted pill, category-colored outline + text, with icon
// Usage: #sk("Lance")
#let sk(name) = {
  let entry = skill-registry.at(name, default: ("", ""))
  let cat = entry.at(0)
  let icon = entry.at(1)
  box(
    fill: cat-bg(cat),
    stroke: 0.5pt + cat-color(cat),
    radius: 9pt,
    inset: (x: 7pt, y: 2pt),
    baseline: 2pt,
  )[
    #set align(horizon)
    #grid(
      columns: (auto, auto),
      column-gutter: 4pt,
      align: horizon,
      box(height: 0.95em, image("../../../images/" + icon + ".jpg", height: 0.95em)),
      text(weight: "semibold", fill: cat-color(cat), size: 0.9em, name),
    )
  ]
}

#let section-counter = counter("section")

#let template(title: "", body) = {
  set document(title: title)
  set page(
    paper: "a4",
    margin: (top: 2.2cm, bottom: 2cm, left: 2.2cm, right: 2.2cm),
  )
  set text(
    font: "Inter",
    size: 9.5pt,
    lang: "en",
    fill: ink,
  )
  set par(
    justify: false,
    leading: 0.65em,
    spacing: 0.95em,
  )

  // Keep lists and enums together — break between items is fine, but a single
  // list shouldn't split across pages, and a single bullet item shouldn't split
  // across pages mid-sentence.
  show list: set block(breakable: false)
  show enum: set block(breakable: false)

  // H1 — large display title only (no eyebrow)
  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    section-counter.update(0)
    block[
      #text(
        font: "Inter Display",
        size: 28pt,
        weight: "black",
        tracking: -0.8pt,
        fill: ink,
        it.body,
      )
    ]
    v(6pt)
    line(length: 100%, stroke: 0.6pt + rule-color)
    v(0.6em)
  }

  // H2 — F's numbered presence, but numeral in calmer accent so titles don't
  // outshine the rules. Generous space above.
  show heading.where(level: 2): it => {
    section-counter.step()
    v(1.6em)
    block(breakable: false, sticky: true)[
      #grid(
        columns: (auto, 1fr),
        column-gutter: 10pt,
        align: (right + horizon, horizon),
        text(
          font: "Inter Display",
          size: 30pt,
          weight: "bold",
          fill: accent,
          tracking: -0.5pt,
        )[#context section-counter.display("1")],
        block(spacing: 0pt)[
          #text(
            font: "Inter",
            size: 7.5pt,
            weight: "medium",
            tracking: 2.5pt,
            fill: muted,
            top-edge: "cap-height",
            bottom-edge: "baseline",
            upper("Section"),
          )
          #v(-8pt, weak: false)
          #text(
            font: "Inter Display",
            size: 16pt,
            weight: "bold",
            tracking: -0.2pt,
            fill: ink,
            top-edge: "cap-height",
            bottom-edge: "baseline",
            it.body,
          )
        ],
      )
      #v(2pt)
      #line(length: 100%, stroke: 0.6pt + rule-color)
    ]
    v(0.3em)
  }

  // H3 — E-style: small caps eyebrow in accent
  show heading.where(level: 3): it => {
    v(0.6em)
    text(
      font: "Inter",
      size: 8.5pt,
      weight: "bold",
      tracking: 1.5pt,
      fill: accent,
      upper(it.body),
    )
    v(0.05em)
  }

  // Tables — E-style: tinted header, charcoal hairline under, light alt rows
  set table(
    stroke: (x, y) => if y == 0 {
      (bottom: 1.2pt + strong-rule)
    } else {
      (bottom: 0.4pt + rule-color)
    },
    fill: (x, y) => if y == 0 {
      rgb("#f4f5f7")
    } else if calc.odd(y) {
      none
    } else {
      rgb("#fafafb")
    },
    inset: (x: 8pt, y: 5.5pt),
  )
  set table.header(repeat: true)
  show table.cell.where(y: 0): set text(weight: "bold", size: 9pt, tracking: 0.3pt, fill: ink)

  show raw: it => box(
    fill: rgb("#f4f5f7"),
    radius: 3pt,
    inset: (x: 5pt, y: 3pt),
    text(font: "Menlo", size: 8.5pt, it),
  )

  body
}

// Note box — calm accent (teal), matches eyebrow + numerals
#let note-box(body) = block(
  breakable: false,
  fill: rgb("#f0f6f8"),
  stroke: (left: 2.5pt + accent),
  inset: (left: 14pt, right: 12pt, top: 9pt, bottom: 9pt),
  width: 100%,
)[
  #text(
    font: "Inter",
    size: 7.5pt,
    weight: "bold",
    tracking: 2pt,
    fill: accent,
    upper("Note"),
  )
  #v(2pt)
  #body
]

// Changed box — amber/orange (calm, not red)
#let changed-box(body) = block(
  breakable: false,
  fill: rgb("#fff7e6"),
  stroke: (left: 2.5pt + rgb("#d97706")),
  inset: (left: 14pt, right: 12pt, top: 9pt, bottom: 9pt),
  width: 100%,
)[
  #text(
    font: "Inter",
    size: 7.5pt,
    weight: "bold",
    tracking: 2pt,
    fill: rgb("#d97706"),
    upper("Changed"),
  )
  #v(2pt)
  #body
]

// Designer-only notes — muted, visually backgrounded
#let designer-box(body) = block(
  breakable: false,
  stroke: (left: 2pt + muted),
  inset: (left: 14pt, right: 12pt, top: 7pt, bottom: 7pt),
  width: 100%,
)[
  #text(
    font: "Inter",
    size: 7pt,
    weight: "bold",
    tracking: 2pt,
    fill: muted,
    upper("Designer"),
  )
  #v(2pt)
  #text(size: 8.5pt, fill: dim, body)
]

#let hr = {
  v(0.4em)
  line(length: 100%, stroke: 0.4pt + rule-color)
  v(0.4em)
}

#let keep-together(body) = block(breakable: false, body)

// Skill table — slightly smaller text to fit 15 rows on one page
#let skill-table(..args) = {
  set text(size: 9pt)
  block(breakable: false, table(..args))
}

// Skill icon — inline image from /images/ folder, sized for table rows
#let skill-icon(name) = {
  box(height: 1.8em, image("../../../images/" + name + ".jpg", height: 1.8em))
}

// Feedback question — numbered, with breathing room
#let fq(num, body) = {
  v(0.5em)
  grid(
    columns: (1.4em, 1fr),
    gutter: 4pt,
    align(top, text(weight: "bold", fill: accent, num + ".")),
    body,
  )
}

// Rating row — label left, scale right
#let rating-row(label, scale) = {
  v(0.3em)
  grid(
    columns: (auto, 1fr),
    gutter: 6pt,
    text(weight: "bold", label),
    text(fill: dim, scale),
  )
}
