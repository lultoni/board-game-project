// template.typ — shared design template for all scenario rule sheets
// Import with: #import "/path/to/template.typ": *

#let template(title: "", body) = {
  set document(title: title)
  set page(
    paper: "a4",
    margin: (top: 1.8cm, bottom: 1.8cm, left: 2cm, right: 2cm),
  )
  set text(
    font: "Helvetica Neue",
    size: 9.5pt,
    lang: "en",
  )
  set par(
    justify: true,
    leading: 0.55em,
  )

  // Headings — keep heading together with following content
  show heading.where(level: 1): it => {
    v(0.3em)
    block(
      breakable: false,
      fill: rgb("#1a1a2e"),
      radius: 3pt,
      inset: (x: 8pt, y: 5pt),
      width: 100%,
      text(fill: white, size: 13pt, weight: "bold", it.body)
    )
    v(0.3em)
  }
  show heading.where(level: 2): it => {
    v(0.5em)
    block(breakable: false)[
      #line(length: 100%, stroke: 0.5pt + rgb("#cccccc"))
      #v(4pt)
      #text(size: 10.5pt, weight: "bold", fill: rgb("#1a1a2e"), it.body)
      #v(0.15em)
    ]
  }
  show heading.where(level: 3): it => {
    v(0.25em)
    text(size: 9.5pt, weight: "bold", fill: rgb("#444444"), it.body)
    v(0.05em)
  }

  // Tables — compact, no page break inside small tables
  set table(
    stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("#1a1a2e")) } else { (bottom: 0.4pt + rgb("#e0e0e0")) },
    fill: (x, y) => if y == 0 { rgb("#f0f0f5") } else if calc.odd(y) { white } else { rgb("#fafafa") },
    inset: (x: 6pt, y: 3.5pt),
  )
  set table.header(repeat: true)

  // Monospace blocks
  show raw: it => box(
    fill: rgb("#f5f5f5"),
    radius: 3pt,
    inset: (x: 6pt, y: 4pt),
    text(font: "Menlo", size: 8.5pt, it)
  )

  body
}

// Highlight box for changed sections
#let changed-box(body) = block(
  breakable: false,
  fill: rgb("#fff8e6"),
  stroke: (left: 3pt + rgb("#f0a500")),
  radius: (right: 3pt),
  inset: (left: 10pt, right: 8pt, top: 6pt, bottom: 6pt),
  width: 100%,
  body
)

// Info box for notes/context
#let note-box(body) = block(
  breakable: false,
  fill: rgb("#f0f4ff"),
  stroke: (left: 3pt + rgb("#3a6bc4")),
  radius: (right: 3pt),
  inset: (left: 10pt, right: 8pt, top: 6pt, bottom: 6pt),
  width: 100%,
  body
)

// Horizontal rule shorthand
#let hr = line(length: 100%, stroke: 0.5pt + rgb("#cccccc"))

// Wrap a section (heading + content) so it won't break across pages if it fits
// Usage: #keep-together[== Heading \n\n content]
#let keep-together(body) = block(breakable: false, body)

// Skill table — smaller text to fit 15 rows on one page without wrapping
#let skill-table(..args) = {
  set text(size: 8.5pt)
  block(breakable: false, table(..args))
}

// Skill icon — inline image from /images/ folder, sized for table rows
// Usage: #skill-icon("lance_thrust")
#let skill-icon(name) = {
  box(height: 1.8em, image("../../../images/" + name + ".jpg", height: 1.8em))
}

// Feedback question — numbered, with breathing room
// Usage: #fq("1")[Question text \ _Answer options_]
#let fq(num, body) = {
  v(0.5em)
  grid(
    columns: (1.4em, 1fr),
    gutter: 4pt,
    align(top, text(weight: "bold", num + ".")),
    body,
  )
}

// Rating row — label on left, scale options on right, write-in box after
// Usage: #rating-row("Label:")[1 — 2 — 3 — 4 — 5]
#let rating-row(label, scale) = {
  v(0.3em)
  grid(
    columns: (auto, 1fr),
    gutter: 6pt,
    text(weight: "bold", label),
    text(fill: rgb("#444444"), scale),
  )
}
