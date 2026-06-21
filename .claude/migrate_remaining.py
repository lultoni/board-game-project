#!/usr/bin/env python3
"""Migrate remaining tables: essays, design_docs, next_steps, backpocket.

Essays: full bodies of every docs/research/*.md file.
Design docs: docs/game-identity-visual-naming.md.
Next steps: parsed from game-state/NEXT_STEPS.md (Priority sections only).
Backpocket: docs/backpocket.md (one row per `## ` section).
"""

import re
import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"

conn = sqlite3.connect(DB)
cur = conn.cursor()

# ============================================================
# ESSAYS — full bodies of docs/research/*.md
# ============================================================
ESSAY_META = {
    # filename stem: (title, topic, date)
    "checkmate-win-conditions": ("Checkmate Win Conditions", "win-conditions", None),
    "cognitive-load-game-design": ("Cognitive Load in Game Design", "cognitive-load", None),
    "competitive-card-fighters-landscape": ("Competitive Card Fighters Landscape", "comparable-games", None),
    "cooperative-feel-competitive-games": ("Cooperative Feel in Competitive Games", "player-psychology", None),
    "forward-positioning-incentives": ("Forward Positioning Incentives", "spatial-design", None),
    "game-economy-map": ("Game Economy Map", "economy-analysis", "2026-06-21"),
    "high-concept-open-questions": ("High Concept Open Questions", "high-concept", None),
    "ios-touch-drag-tap": ("iOS Touch — Drag vs Tap", "ux-research", None),
    "mechanical-reward-clever-play": ("Mechanical Reward for Clever Play", "incentive-design", None),
    "old-versions-ideas": ("Old Versions — Ideas Backlog", "archive", None),
    "path-y-defense-redesign": ("Path Y — Defense Redesign", "defense-design", "2026-05-30"),
    "perfect-info-tactical-games": ("Perfect Info Tactical Games — Genre Landscape", "comparable-games", None),
    "playtest-1-analysis": ("Playtest 1 — Full Analysis", "playtest-analysis", "2025-10-31"),
    "playtest-2-analysis": ("Playtest 2 — Full Analysis", "playtest-analysis", "2026-04-24"),
    "playtest-3-analysis": ("Playtest 3 — Full Analysis", "playtest-analysis", "2026-05-17"),
    "playtest-4-analysis": ("Playtest 4 — Full Analysis", "playtest-analysis", "2026-05-28"),
    "skill-catalogue-balance": ("Skill Catalogue Balance", "skill-design", None),
    "spending-incentive-balance": ("Spending Incentive Balance", "economy-design", None),
    "system-audit-vs-high-concept": ("System Audit vs High Concept", "high-concept", None),
    "wizard-chess-genre-landscape": ("Wizard Chess Genre Landscape", "comparable-games", None),
    "youtube-transcript-high-concept": ("YouTube Transcript — High Concept", "transcript", None),
}

essay_count = 0
for md in (ROOT / "docs" / "research").glob("*.md"):
    stem = md.stem
    if stem not in ESSAY_META:
        print(f"SKIP essay (no meta): {md.name}")
        continue
    title, topic, date = ESSAY_META[stem]
    body = md.read_text()
    eid = f"essay-{stem}"
    try:
        cur.execute(
            "INSERT INTO essays (id, title, topic, body, date) VALUES (?, ?, ?, ?, ?)",
            (eid, title, topic, body, date),
        )
        essay_count += 1
    except Exception as e:
        print(f"  FAIL {eid}: {e}")

# Also: youtube transcript files in docs/research/youtube-transcripts/ if present
yt_dir = ROOT / "docs" / "research" / "youtube-transcripts"
if yt_dir.exists():
    for f in yt_dir.glob("*.md"):
        eid = f"essay-yt-{f.stem}"
        try:
            cur.execute(
                "INSERT INTO essays (id, title, topic, body, date) VALUES (?, ?, ?, ?, ?)",
                (eid, f.stem.replace("-", " ").title(), "youtube-transcript", f.read_text(), None),
            )
            essay_count += 1
        except Exception as e:
            print(f"  FAIL {eid}: {e}")

# Session 26 brainstorm scans (text-content README/notes if any)
s26 = ROOT / "docs" / "research" / "session-26-brainstorm-scans"
if s26.exists():
    for f in s26.glob("*.md"):
        eid = f"essay-s26-{f.stem}"
        try:
            cur.execute(
                "INSERT INTO essays (id, title, topic, body, date) VALUES (?, ?, ?, ?, ?)",
                (eid, "Session 26 Brainstorm — " + f.stem, "brainstorm", f.read_text(), "2026-06-21"),
            )
            essay_count += 1
        except Exception as e:
            print(f"  FAIL {eid}: {e}")

print(f"Inserted {essay_count} essay rows.")

# ============================================================
# DESIGN DOCS — docs/game-identity-visual-naming.md
# ============================================================
gid = ROOT / "docs" / "game-identity-visual-naming.md"
if gid.exists():
    cur.execute(
        """INSERT INTO design_docs (id, title, domain, status, body, established_in)
           VALUES (?, ?, ?, ?, ?, ?)""",
        ("design-doc-game-identity-visual-naming",
         "Game Identity, Visual Direction, Naming (Phase B)",
         "identity",
         "active",
         gid.read_text(),
         None),
    )
    print("Inserted 1 design_docs row (game-identity-visual-naming).")

# Also: docs/systems-and-mechanics.md as a design doc (system-design domain)
sam = ROOT / "docs" / "systems-and-mechanics.md"
if sam.exists():
    cur.execute(
        """INSERT INTO design_docs (id, title, domain, status, body, established_in)
           VALUES (?, ?, ?, ?, ?, ?)""",
        ("design-doc-systems-and-mechanics",
         "Systems and Mechanics — Per-System Rationale",
         "system-design",
         "active",
         sam.read_text(),
         None),
    )
    print("Inserted 1 design_docs row (systems-and-mechanics).")

# ============================================================
# NEXT STEPS — parse Priority sections from NEXT_STEPS.md
# ============================================================
ns_text = (ROOT / "game-state" / "NEXT_STEPS.md").read_text()
priority_re = re.compile(r"(?m)^## Priority (\d+) — (.+?)$")
parts = priority_re.split(ns_text)
# parts: [preamble, "1", "title", body, "2", "title", body, ...]
ns_count = 0
for i in range(1, len(parts), 3):
    prio = int(parts[i])
    title = parts[i + 1].strip()
    body = parts[i + 2]
    # Stop at next `## ` header
    body = re.split(r"(?m)^## (?!Priority)", body, maxsplit=1)[0].rstrip()
    try:
        cur.execute(
            """INSERT INTO next_steps (priority, title, status, body, created_in)
               VALUES (?, ?, ?, ?, ?)""",
            (prio, title, "todo", body.strip(), "session-26"),
        )
        ns_count += 1
    except Exception as e:
        print(f"  FAIL next_step P{prio}: {e}")
print(f"Inserted {ns_count} next_steps rows.")

# ============================================================
# BACKPOCKET — one row per `## ` section in docs/backpocket.md
# ============================================================
bp_text = (ROOT / "docs" / "backpocket.md").read_text()
# Split on `## ` headers (level-2)
sections = re.split(r"(?m)^## ", bp_text)
# First chunk is preamble
bp_count = 0
for section in sections[1:]:
    lines = section.split("\n", 1)
    title = lines[0].strip()
    body = lines[1].rstrip() if len(lines) > 1 else ""
    # Slug for ID
    slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")[:60]
    bp_id = f"bp-{slug}"
    # Guess category based on title keywords
    tl = title.lower()
    if "guardrail" in tl:
        category = "guardrail"
    elif "skill" in tl and ("candidate" in tl or "catalogue" in tl or "expansion" in tl):
        category = "skill-candidate"
    elif "tooling" in tl or "prototype" in tl or "digital" in tl:
        category = "tooling"
    elif "methodology" in tl or "process" in tl or "brainstorm" in tl:
        category = "process"
    elif "discuss" in tl or "note" in tl:
        category = "note"
    else:
        category = "staged-fix"
    # Guess status
    ul = body.upper()
    if "PROMOTED" in ul:
        status = "promoted"
    elif "WITHDRAWN" in ul or "PARKED" in ul:
        status = "parked"
    else:
        status = "active"
    try:
        cur.execute(
            """INSERT INTO backpocket (id, name, category, status, body, created_in)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (bp_id, title, category, status, body, None),
        )
        bp_count += 1
    except sqlite3.IntegrityError as e:
        # Likely duplicate slug — append numeric suffix
        for suffix in range(2, 10):
            try:
                cur.execute(
                    """INSERT INTO backpocket (id, name, category, status, body, created_in)
                       VALUES (?, ?, ?, ?, ?, ?)""",
                    (f"{bp_id}-{suffix}", title, category, status, body, None),
                )
                bp_count += 1
                break
            except sqlite3.IntegrityError:
                continue
        else:
            print(f"  FAIL backpocket {bp_id}: {e}")
print(f"Inserted {bp_count} backpocket rows.")

conn.commit()
conn.close()
print("\n=== Summary ===")
conn = sqlite3.connect(DB)
cur = conn.cursor()
for t in ("essays", "design_docs", "next_steps", "backpocket"):
    cur.execute(f"SELECT COUNT(*) FROM {t};")
    print(f"  {t:<14} {cur.fetchone()[0]}")
conn.close()
