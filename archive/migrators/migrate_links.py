#!/usr/bin/env python3
"""Populate links table with explicit cross-references.

Hand-curated. Covers:
- Stack ↔ OQ (addresses)
- Stack ↔ Mechanic (related-to)
- Playtest ↔ Stack (evidence-for) — already via stacks.playtested_in, but linked too for graph queries
- ADR ↔ Principle (parent-of)
- Stack absorbed-into Stack M
- OQ ↔ session (opened-by / resolved-by — already via FK, mirrored for graph)
- Principle ↔ Principle (related-to)
- Backpocket promoted-to Stack / Mechanic
"""

import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"

links = [
    # === Stack ↔ OQ (addresses) ===
    ("stack-m", "oq-11", "addresses", "Armor chassis-volume — addressed via cap 3→2"),
    ("stack-m", "oq-34", "addresses", "Steal cost — addressed via 3→4"),
    ("stack-m", "oq-38", "addresses", "Combo widening — addressed via dual-axis widening"),
    ("stack-m", "oq-57", "addresses", "Injured penalties — addressed via removal"),
    ("stack-m", "oq-66", "addresses", "Game length 30-60 min target — primary axis"),
    ("stack-m", "oq-68", "addresses", "Draw conditions — addressed via removal"),
    ("stack-m", "oq-60", "related-to", "Cognitive load — watched"),
    ("stack-m", "oq-64", "related-to", "Felt-PI — watched on 8×8"),
    ("stack-h", "oq-11", "addresses", "Armor chassis-volume hypothesis (P4 + P5 cross-pole confirmed)"),
    ("stack-h", "oq-57", "addresses", "Injured chassis volume"),
    ("stack-h", "oq-58", "addresses", "Exchange-pit pattern"),
    ("stack-h", "oq-66", "addresses", "Game length lever"),
    ("stack-a-g3", "oq-38", "addresses", "Scope-not-strength reframe"),
    ("stack-a-g3", "oq-58", "addresses", "Mid-game exchange-pit pattern"),
    ("stack-a-g3", "oq-59", "addresses", "Esp. 59b endgame conversion gap"),
    ("stack-k", "oq-27", "addresses", "Piece density"),
    ("stack-k", "oq-66", "addresses", "Game length lever"),
    ("stack-j", "oq-57", "addresses", "Does Injured chassis pay for itself?"),
    ("stack-c", "oq-19", "addresses", "Game length cap"),
    ("stack-c", "oq-41", "addresses", "Pacing"),
    ("stack-d", "oq-52", "addresses", "Narrower board (8×10)"),
    ("stack-d", "oq-42", "addresses", "Hex variant (gated on research)"),
    ("stack-d", "oq-66", "addresses", "Game length lever"),
    ("stack-e", "oq-35", "addresses", "Pool draft"),
    ("stack-e", "oq-36", "addresses", "Placement order"),
    ("stack-e", "oq-48", "addresses", "Placement order"),
    ("stack-f", "oq-51", "addresses", "Cascade trigger"),
    ("stack-g", "oq-26", "addresses", "Unified AP model"),
    ("stack-l", "oq-61", "addresses", "Two-pole framing — Pole B prototype"),
    ("stack-l", "oq-64", "addresses", "Felt-PI broke under combinatorial breadth"),

    # === Stack absorbed into Stack M ===
    ("stack-h", "stack-m", "absorbed-into", "Armor cap 3→2 (one of six bundled changes)"),
    ("stack-j", "stack-m", "absorbed-into", "Injured penalties removed (one of six)"),
    ("stack-i", "stack-h", "absorbed-into", "Smaller-dose Armor cap reduction (Session 22)"),

    # === Playtest evidence ===
    ("playtest-3", "stack-a-g1", "evidence-for", "Standoff dissolved; first kill R26→R11"),
    ("playtest-4", "stack-a-g2", "evidence-for", "Combo mechanic confirmed; exchange-pit pattern identified"),
    ("playtest-4", "oq-11", "evidence-for", "Armor↔Armor-Breaker loop crowds out combo loop"),
    ("playtest-4", "oq-58", "evidence-for", "Mid-game exchange-pit pattern (R15-R21)"),
    ("playtest-5", "stack-l", "evidence-for", "Three structural problems — Pole B paused"),
    ("playtest-5", "oq-11", "evidence-for", "Cross-pole confirmation of Armor chassis volume"),
    ("playtest-5", "oq-64", "evidence-for", "Felt-PI broke under combinatorial breadth"),
    ("playtest-5", "oq-61", "evidence-for", "Pole B paused; Pole A continues"),

    # === ADRs → Principles (parent-of) ===
    ("adr-001", "constraint-grid-based", "parent-of", "Grid-based hard constraint"),
    ("adr-002", "constraint-perfect-info", "parent-of", "Perfect information hard constraint"),
    ("adr-002", "constraint-no-terrain", "parent-of", "No terrain hard constraint"),
    ("adr-003", "principle-1", "parent-of", "Every archetype has a moment"),
    ("adr-003", "principle-2", "parent-of", "KPI Principle"),
    ("adr-003", "principle-3", "parent-of", "Players play how they want"),
    ("adr-003", "principle-4", "parent-of", "Cleverness > grinding"),
    ("adr-003", "principle-5", "parent-of", "Shared puzzle is byproduct"),
    ("adr-004", "principle-high-concept-framing", "parent-of", "Two minds, one puzzle"),

    # === Principle relationships ===
    ("principle-6", "principle-4", "related-to", "Extends cleverness-over-attrition to meta-experience"),
    ("principle-6", "principle-chassis-and-engine", "related-to", "Chassis bloat is also a length problem"),
    ("principle-7", "principle-incremental-testing", "supersedes", "Suspends methodology while core unsettled (conditional)"),
    ("principle-8", "principle-6", "related-to", "Short games + single climax = current game-shape target"),
    ("principle-chassis-and-engine", "principle-core-fantasy", "related-to", "Engine is where Core Fantasy lives"),
    ("principle-high-concept-framing", "principle-core-fantasy", "related-to", "Framing wraps Core Fantasy"),

    # === Essays ↔ Playtests / Stacks (evidence-for) ===
    ("essay-playtest-1-analysis", "playtest-1", "evidence-for", "Full P1 analysis"),
    ("essay-playtest-2-analysis", "playtest-2", "evidence-for", "Full P2 analysis"),
    ("essay-playtest-3-analysis", "playtest-3", "evidence-for", "Full P3 analysis"),
    ("essay-playtest-4-analysis", "playtest-4", "evidence-for", "Full P4 analysis"),
    ("essay-game-economy-map", "stack-m", "evidence-for", "12-economy structural justification for Stack M"),
    ("essay-path-y-defense-redesign", "stack-l", "evidence-for", "Pole A vs Pole B framing established"),
    ("essay-path-y-defense-redesign", "principle-6", "evidence-for", "Principle 6 established here"),
    ("essay-path-y-defense-redesign", "principle-7", "evidence-for", "Principle 7 established here"),

    # === OQ relationships ===
    ("oq-58", "oq-11", "related-to", "Exchange-pit feeds off Armor stalling"),
    ("oq-64", "oq-61", "related-to", "Felt-PI surfaced via Pole B"),
    ("oq-66", "principle-6", "derived-from", "Game length attrition principle"),
    ("oq-67", "oq-11", "related-to", "Bodyguard removal candidate"),
    ("oq-68", "principle-8", "derived-from", "Single-climax shape"),
]

conn = sqlite3.connect(DB)
cur = conn.cursor()
ok = 0
fail = []
for (frm, to, rel, note) in links:
    try:
        cur.execute(
            "INSERT INTO links (from_id, to_id, relation, note) VALUES (?, ?, ?, ?)",
            (frm, to, rel, note),
        )
        ok += 1
    except Exception as e:
        fail.append((frm, to, rel, str(e)))
conn.commit()
print(f"Inserted {ok} link rows.")
if fail:
    print("Failures:")
    for f in fail[:10]:
        print(f"  {f[0]} -> {f[1]} ({f[2]}): {f[3]}")

cur.execute("SELECT relation, COUNT(*) FROM links GROUP BY relation ORDER BY 2 DESC;")
print("\nBy relation:")
for r, c in cur.fetchall():
    print(f"  {r:<14} {c}")
conn.close()
