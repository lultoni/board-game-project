#!/usr/bin/env python3
"""Migrate 5 playtests into playtests table.

Bodies are summaries — full analyses live as essays (Step 4j) keyed `essay-playtest-N-analysis`.
"""

import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"

playtests = [
    # (id, n, date, players, stack_id, rounds, duration_min, outcome, body, raw_artefacts_path)
    ("playtest-1", 1, "2025-10-31",
     '["Elias", "Pasco"]', None, None, None,
     "Game too long; slow economy; Bodyguard inert",
     """*First-ever playtest of the post-Session-1 ruleset. Establishes the baseline problems that drove Layer 1 (economy fix) and Stack A G1 (attack nerf).*

**Top findings (see `essay-playtest-1-analysis` for full):**
1. Game too long — first Champion kill > R20.
2. Economy too slow — starting Money insufficient; per-turn income lagged decision-points.
3. Bodyguard never triggered organically.
4. Standoff zone identified — Move-Attack 2 damage = free attack, removed engagement incentive.

**Drove:**
- Layer 1 (Economy fix, accepted P2): starting Money 6, +2/turn, +1 every 5 rounds.
- Stack A G1 (Move-Attack 1 damage, accepted P3).
- OQ-23 (standoff).""",
     "playtest-results/elias-vs-pasco-31_10_25/"),

    ("playtest-2", 2, "2026-04-24",
     '["Elias", "Jonathan"]', None, None, None,
     "Layer 1 (Economy) confirmed; mutual-epistemic-exploration observed",
     """*Layer 1 ruleset (6 start, +2/turn, +1 every 5 rounds).*

**Top findings (see `essay-playtest-2-analysis` for full):**
1. Economy felt right — players had spending tension (G8) throughout.
2. Mutual-epistemic-exploration observed organically (both players co-interpreting the same puzzle).
3. Standoff zone persisted — confirmed need for Stack A G1.

**Drove:** Layer 1 accepted into baseline; Stack A G1 (attack nerf) prioritised.""",
     "playtest-results/elias-vs-jonathan-24_04_26/"),

    ("playtest-3", 3, "2026-05-17",
     '["Elias", "Mario"]', "stack-a-g1", None, None,
     "Stack A G1 (Attack Nerf) confirmed — standoff dissolved",
     """*Stack A G1: Move-Attack deals 1 damage (was 2).*

**Top findings (see `essay-playtest-3-analysis` for full):**
1. **Standoff dissolved.** First Champion kill moved R26 → R11.
2. Skills became primary damage source.
3. Bodyguard activated organically (without Stack B).
4. New OQ-52 (board feels long; narrow direction perceived).
5. New OQ-53 (Champion-vs-Guard exchange rate).

**Drove:** Stack A G1 accepted into baseline; Stack B (Bodyguard fix) de-prioritised (later withdrawn).""",
     "playtest-results/elias-vs-mario-17_05_26/"),

    ("playtest-4", 4, "2026-05-28",
     '["Elias", "Niko"]', "stack-a-g2", None, None,
     "Stack A G2 (Multi-Champion Combo) confirmed; mid-game exchange-pit pattern identified",
     """*Stack A G2: Multi-Champion Strike-only combo counter +0/+1/+2. Niko's first game — feedback-onboarding form used.*

**Top findings (see `essay-playtest-4-analysis` for full):**
1. Combo bonus worked as designed; players coordinated multi-Champion sequences.
2. Mid-game exchange-pit pattern (R15-R21 Armor-stack cluster).
3. Late-game offensive lockout (Elias verbatim: "I did not have any other attack champs left").
4. Cross-category crowd-out (#3): Strike-skill counter excluded movement skills from the combo loop.
5. Armor↔Armor-Breaker loop crowded out combo loop (OQ-11 confirmed at chassis-volume level).

**Drove:** Stack A G2 accepted into baseline (migrated in Session 23). Discussion produced Stack A G3 (Dual-Counter Combo); Stack H (Armor Trim) prioritised.""",
     "playtest-results/elias-vs-niko-28_05_26/"),

    ("playtest-5", 5, "2026-06-08",
     '["Elias", "Jonathan"]', "stack-l", 15, None,
     "Stack L (Pole B Per-Turn-Draft) PAUSED — three structural problems",
     """*Stack L (Pole B): per-turn skill drafting. Digital prototype. 15 rounds.*

**Three structural problems surfaced:**
1. **Armor 3 still felt mandatory** — cross-pole confirmation of OQ-11 (chassis-volume problem).
2. **Play collapsed to pure reaction** — no multi-turn planning. The draft cadence destroyed the opportunity to set up sequences in advance.
3. **Felt-PI broke under combinatorial breadth (OQ-64)** — too many simultaneous options each turn for the game to feel "graspable" even though it was formally perfect-info.

**Drove:**
- Pole B paused (Session 25). Pole A returns as Active.
- OQ-64 (felt-PI vs formal-PI) raised.
- OQ-11 cross-pole confirmation strengthens Stack H / Stack M Armor cap arguments.
- Backpocket entries staged for Pole B variants: permanently-equipped (non-consumable) drafted skills; per-Skill-Phase activation cap; skills-cost-a-resource.

**Notes file:** `playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/notes.md`.""",
     "playtest-results/elias-vs-jonathan-pole-b-digital-2026-06/"),
]

conn = sqlite3.connect(DB)
cur = conn.cursor()
ok = 0
fail = []
for r in playtests:
    try:
        cur.execute(
            """INSERT INTO playtests
               (id, n, date, players, stack_id, rounds, duration_min, outcome, body, raw_artefacts_path)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            r,
        )
        ok += 1
    except Exception as e:
        fail.append((r[0], str(e)))
conn.commit()
print(f"Inserted {ok} playtest rows.")
if fail:
    print("Failures:")
    for f in fail:
        print(f"  {f[0]}: {f[1]}")

cur.execute("SELECT id, n, date, stack_id, outcome FROM playtests ORDER BY n;")
for row in cur.fetchall():
    print(f"  {row[0]:<12} n={row[1]} {row[2]} stack={row[3] or '-':<10} {row[4][:60]}")
conn.close()
