#!/usr/bin/env python3
"""One-shot migrator: parse game-state/SESSION_LOG.md → sessions table."""

import re
import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
LOG = ROOT / "game-state" / "SESSION_LOG.md"
DB = ROOT / "design" / "design.db"

text = LOG.read_text()

# Split on `### ` headers
chunks = re.split(r"(?m)^### ", text)
# chunks[0] is preamble; rest each start with "<date_or_label> — Session N: Title\n\nbody..."

month_map = {
    "January": "01", "February": "02", "March": "03", "April": "04",
    "May": "05", "June": "06", "July": "07", "August": "08",
    "September": "09", "October": "10", "November": "11", "December": "12",
}

def parse_date(date_str: str) -> str:
    """Best-effort ISO-8601 from header date strings."""
    date_str = date_str.strip()
    # "June 21, 2026"
    m = re.match(r"^([A-Z][a-z]+)\s+(\d+),\s+(\d{4})$", date_str)
    if m:
        return f"{m.group(3)}-{month_map[m.group(1)]}-{int(m.group(2)):02d}"
    # "April 27" (assume 2026)
    m = re.match(r"^([A-Z][a-z]+)\s+(\d+)$", date_str)
    if m:
        return f"2026-{month_map[m.group(1)]}-{int(m.group(2)):02d}"
    # "April 29"
    # "May 2026"
    m = re.match(r"^([A-Z][a-z]+)\s+(\d{4})$", date_str)
    if m:
        return f"{m.group(2)}-{month_map[m.group(1)]}-01"
    # "October 2025"
    return None  # caller handles


rows = []  # (id, n, date, title, body)

for chunk in chunks[1:]:
    # First line up to newline is the header
    lines = chunk.split("\n", 1)
    header = lines[0]
    body = lines[1].strip() if len(lines) > 1 else ""
    # Strip trailing horizontal-rule "---" sections from body
    body = re.sub(r"\n---\s*$", "", body).rstrip()
    body = re.sub(r"\n---\n.*$", "", body, flags=re.DOTALL).rstrip()

    # Parse "<date> — Session N: <title>" or "<date> — Sessions N–M: <title>" or "<date> — Playtest N: ..." or "Session 4: ..."

    # Skip playtest-only entries (no Session N anchor)
    if "Playtest" in header and "Session" not in header:
        print(f"SKIP (playtest entry): {header}")
        continue

    # Try standard "<date> — Session N: <title>"
    m = re.match(r"^(.+?)\s+—\s+Session\s+(\d+):\s+(.+)$", header)
    if m:
        date = parse_date(m.group(1))
        n = int(m.group(2))
        title = m.group(3).strip()
        rows.append((f"session-{n}", n, date, title, body))
        continue

    # Try "<date> — Sessions N–M: <title>" (en-dash) or "Sessions N-M"
    m = re.match(r"^(.+?)\s+—\s+Sessions?\s+(\d+)[–-](\d+):\s+(.+)$", header)
    if m:
        date = parse_date(m.group(1))
        n_start = int(m.group(2))
        n_end = int(m.group(3))
        title = m.group(4).strip()
        # Split into individual session rows; full body attached to each (annotated)
        annotated = f"*Sessions {n_start}–{n_end} were logged jointly. Full joint entry below; per-session detail is interleaved within.*\n\n{body}"
        for n in range(n_start, n_end + 1):
            rows.append((f"session-{n}", n, date, f"{title} (joint log {n_start}–{n_end})", annotated))
        continue

    # Try bare "Session N: <title>" (Session 4 has no date prefix)
    m = re.match(r"^Session\s+(\d+):\s+(.+)$", header)
    if m:
        n = int(m.group(1))
        title = m.group(2).strip()
        # No date; use placeholder near sibling sessions
        rows.append((f"session-{n}", n, "2026-04-20", title, body))  # Session 4 between Session 3 (Apr 19) and Session 5 (Apr 25)
        continue

    print(f"UNMATCHED: {header}")

# Insert
conn = sqlite3.connect(DB)
cur = conn.cursor()
for r in rows:
    cur.execute(
        "INSERT INTO sessions (id, n, date, title, body) VALUES (?, ?, ?, ?, ?)",
        r,
    )
conn.commit()

print(f"\nInserted {len(rows)} session rows:")
for r in rows:
    print(f"  {r[0]:<14} n={r[1]:<3} date={r[2]}  title={r[3][:60]}")

cur.execute("SELECT COUNT(*) FROM sessions;")
print(f"\nTotal rows in sessions table: {cur.fetchone()[0]}")
conn.close()
