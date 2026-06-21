#!/usr/bin/env python3
"""Migrate OPEN_QUESTIONS.md + OPEN_QUESTIONS_ARCHIVE.md → open_questions table.

Strategy: parse `### OQ-X: Title — STATUS` headers; capture body until next `### `.
Map status text to schema enum; infer priority from section header (Critical/High/Medium/Deferred/Open).
"""

import re
import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"
LIVE = ROOT / "game-state" / "OPEN_QUESTIONS.md"
ARCH = ROOT / "game-state" / "OPEN_QUESTIONS_ARCHIVE.md"

# Status mapping rules: header text → (status_enum, priority_int_or_None)
SECTION_TO_PRIORITY = {
    "Critical": ("critical", 1),
    "High Priority": ("high", 2),
    "Medium Priority": ("medium", 3),
    "Deferred": ("deferred", 4),
    "Open": ("open", 5),
    "Resolved": ("resolved", None),
    "Closed": ("closed", None),
    "Scrapped": ("scrapped", None),
    "Parked Indefinitely": ("parked", None),
}

# Resolution-keyword overrides on the header line itself (e.g. "RESOLVED via Stack M")
HEADER_STATUS_OVERRIDES = [
    ("RESOLVED", "resolved"),
    ("CLOSED", "closed"),
    ("SCRAPPED", "scrapped"),
    ("PARKED", "parked"),
    ("ARCHIVED", "archived"),
]

# Where each OQ was *created* (best-effort from text — N/A for many; populate when explicit).
# Leave NULL when uncertain.

def parse_file(path: Path, archive: bool):
    """Return list of (id, title, status, priority, body, created_in_hint, resolved_in_hint)."""
    text = path.read_text()
    # Split on `## ` section headers (Critical / High Priority / etc.)
    section_re = re.compile(r"(?m)^## (.+)$")
    parts = section_re.split(text)
    # parts[0] preamble, then alternating section_name, body
    results = []
    for i in range(1, len(parts), 2):
        section_name = parts[i].strip()
        section_body = parts[i + 1]
        # Determine default status/priority from section
        status_enum, priority = None, None
        for key, (s, p) in SECTION_TO_PRIORITY.items():
            if section_name.startswith(key):
                status_enum, priority = s, p
                break
        if status_enum is None:
            # archive sections like "Resolved" / "Closed" / "Scrapped" / "Parked Indefinitely"
            continue  # shouldn't reach here

        # Split on `### OQ-X:` headers
        oq_chunks = re.split(r"(?m)^### (OQ-[^\s:]+):\s*(.+)$", section_body)
        # pattern: preamble, then groups of [id, title_rest, body]
        for j in range(1, len(oq_chunks), 3):
            oq_id = oq_chunks[j].strip().lower()
            title_line = oq_chunks[j + 1].strip()
            body = oq_chunks[j + 2].rstrip()
            # Strip trailing horizontal rules
            body = re.sub(r"\n---\s*$", "", body).rstrip()
            body = re.sub(r"\n---\n[\s\S]*$", "", body).rstrip()

            # Status: start with section default, but check title for override keywords
            current_status = status_enum
            up = title_line.upper()
            for kw, override in HEADER_STATUS_OVERRIDES:
                if kw in up:
                    current_status = override
                    break

            # Title = portion before " — " separator if present (strip status tags)
            title = re.split(r"\s+—\s+", title_line, maxsplit=1)[0].strip()
            if not title:
                title = title_line.strip()

            results.append({
                "id": oq_id,
                "title": title,
                "status": current_status,
                "priority": priority,
                "body": body,
                "archive_source": archive,
            })
    return results


live = parse_file(LIVE, archive=False)
arch = parse_file(ARCH, archive=True)

# Merge with dedupe: if the same id appears in both, prefer live (which holds residual entries pointing back).
seen = {}
for r in live + arch:
    if r["id"] in seen:
        # Keep live version; append archive note to its body if needed
        continue
    seen[r["id"]] = r

rows = list(seen.values())

# Hand-curated created_in / resolved_in for rows where we know with certainty.
# (Best-effort only — Step 4l link sweep will fill more cross-refs.)
known_anchors = {
    # OQ ID: (created_in, resolved_in)
    "oq-58":  ("session-22", None),
    "oq-59":  ("session-22", None),
    "oq-60":  ("session-22", None),
    "oq-61":  ("session-23", None),
    "oq-62":  ("session-23", None),
    "oq-63":  ("session-23", "session-25"),
    "oq-64":  ("session-25", None),
    "oq-65":  ("session-25", None),
    "oq-66":  ("session-25", None),
    "oq-67":  ("session-25", None),
    "oq-68":  ("session-25", None),
    "oq-39":  (None, "session-20"),
    "oq-1":   ("session-1", None),
    "oq-3":   (None, None),
    "oq-5":   (None, "session-2"),
    "oq-9":   (None, None),
    "oq-13":  (None, None),
    "oq-17":  (None, "session-5"),
    "oq-20":  (None, "session-3"),
    "oq-29":  (None, "session-5"),
    "oq-30":  (None, "session-3"),
    "oq-31":  (None, "session-3"),
    "oq-32":  (None, "session-3"),
    "oq-33":  (None, "session-3"),
    "oq-37":  (None, "session-15"),
    "oq-40":  (None, "session-15"),
    "oq-46":  (None, "session-15"),
    "oq-10":  (None, "session-18"),
    "oq-11":  ("session-22", None),  # reopened session 20, broad scope evolved
    "oq-25":  (None, "session-8"),
    "oq-47":  (None, "session-8"),
    "oq-18":  (None, "session-6"),
    "oq-15":  (None, "session-1"),
    "oq-28":  (None, None),
    "oq-43":  (None, None),
    "oq-44":  (None, None),
    "oq-45":  (None, None),
}

conn = sqlite3.connect(DB)
cur = conn.cursor()
ok = 0
fail = []
for r in rows:
    created_in, resolved_in = known_anchors.get(r["id"], (None, None))
    try:
        cur.execute(
            """INSERT INTO open_questions
               (id, title, status, priority, affected_systems, body, created_in, resolved_in)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            (r["id"], r["title"], r["status"], r["priority"], None, r["body"], created_in, resolved_in),
        )
        ok += 1
    except Exception as e:
        fail.append((r["id"], str(e)))
conn.commit()
print(f"Inserted {ok} OQ rows.")
if fail:
    print("Failures:")
    for f in fail:
        print(f"  {f[0]}: {f[1]}")

cur.execute("SELECT status, COUNT(*) FROM open_questions GROUP BY status ORDER BY status;")
print("\nCount by status:")
for s, c in cur.fetchall():
    print(f"  {s:<12} {c}")
cur.execute("SELECT COUNT(*) FROM open_questions;")
print(f"\nTotal: {cur.fetchone()[0]}")
conn.close()
