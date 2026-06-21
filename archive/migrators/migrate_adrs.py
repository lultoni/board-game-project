#!/usr/bin/env python3
"""Insert ADR rows synthesised from cross-references in design-principles, sessions, mechanics-evaluated."""

import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"

adrs = [
    ("adr-001", 1, "Grid-based tactical board (Direction A) over card-fighter (B) or zone/lane hybrid (C)",
     "accepted",
     """*Decided Session 1 (2026-04-17).*

**Decision**: The game is a grid-based 2-player tactical board game. Direction A+ (Streamlined Tactical Grid, Onitama model) chosen over Direction B (card-fighter, BattleCon-style) and Direction C (zone/lane hybrid).

**Reasoning**:
- Grid preserves spatial skills (push/pull/swap) that make spells interesting. Without a board, those skills lose their meaning.
- Direction B was rejected because "it's just another card game" — without the spatial layer, the game can't differentiate.
- Direction C was rejected as a hybrid that compromises the spatial depth without delivering the engagement of a full grid.

**Consequences**:
- Hard constraint: grid-based, spatial positioning stays. Any future structural rework must preserve a positional substrate.
- Hex grid is reopened as a *grid variant* (OQ-42) — the original rejection was by omission, not by evaluation.
- Card-fighter mechanics can inspire skill design but cannot replace the board.""",
     "session-1", None),

    ("adr-002", 2, "Perfect information; no terrain effects; no dice / no randomness",
     "accepted",
     """*Decided Session 1 (2026-04-17). YINSH-inspired capture penalty rejected via designer feedback.*

**Decision**: The game is perfect information. No dice, no hidden cards, no randomness. Board is uniform — no terrain effects.

**Reasoning**:
- Perfect information is non-negotiable per designer mandate. Pure strategy, no luck.
- Terrain effects added overhead complexity without commensurate strategic depth — confirmed in pre-Session-1 versions.
- YINSH-style capture penalty (later considered for game-length acceleration) creates asymmetric cost: if one player runs out of Guards, only that player pays the penalty, which punishes correct play. Withdrawn.

**Consequences**:
- Hard constraints: perfect information; grid-based; no terrain.
- Felt-PI (the *feeling* of perfect information) becomes a load-bearing design concern even when formal-PI is preserved — see OQ-64 (Session 25).
- Information loss is permitted only in narrow pre-game windows (e.g. simultaneous-reveal drafting per OQ-62) where the in-game commitment to PI stands.""",
     "session-1", None),

    ("adr-003", 3, "Cleverness over Attrition — the five core design principles",
     "accepted",
     """*Decided Session 7 (2026-04-27). The "turning point" session.*

**Decision**: Adopt five design principles that orient the game toward *cleverness* (multi-turn setup rewarded with payoffs exceeding grinding) and away from *attrition* (free standard-attacks-as-best-strategy).

**Reasoning**:
A full system-by-system audit found the game consistently rewarded attrition over clever play. Standard attack: 2 damage, 0 Money, infinite efficiency. The best combo in the game: 2 damage, 6 Money, 3 Skill Slots — merely matching what a free attack does.

Research on cooperative feel in competitive games surfaced "mutual epistemic exploration" — both players co-interpreting the same puzzle. This was already happening organically in Playtest 2.

**The five principles** (now in the `principles` table as `principle-1` … `principle-5`):
1. Every strategy archetype should have a moment where it's the best option on the board.
2. Don't reward symptoms, reward the system (the KPI Principle).
3. Players should be allowed to play how they want (don't ban strategies; make alternatives competitive).
4. Cleverness = multi-turn positional setup rewarded with a payoff that exceeds what grinding achieves.
5. The shared-puzzle feel is a byproduct of good design, not a mechanic.

**Consequences**:
- Layer 2 redefined: standard attack nerf (1 DMG, eventually accepted as Stack A) + multi-Champion combo bonus.
- All future stacks evaluated against these five principles.
- Sessions 23 + 25 added Principles 6, 7, and 8 — they extend but do not override these five.""",
     "session-7", None),

    ("adr-004", 4, "High-concept framing: \"Two minds, one puzzle\" (Framing B)",
     "accepted",
     """*Decided Session 19 (2026-05-26). Reversal criterion updated via Q-D2 same session.*

**Decision**: The Core Fantasy ("discovering and executing clever spell/skill combos") is delivered under the **"Two minds, one puzzle"** framing (Framing B): two players race to discover and execute clever skill combos in a *shared* combinatorial space.

**Alternative considered (Framing A)**: solo wizard fantasy delivered via 1v1 competition; opponent is a constraint generator. Rejected because the parallel-solving *is* the experience — replacing the opponent with an AI under Framing A would still deliver the core; under Framing B it kills the core.

**Reasoning** (three load-bearing reasons):
1. The 2-player nature is *load-bearing* — design choices that weaken the shared-pool structure or the legibility of the opponent's combos damage the framing.
2. Asymmetry (factions, starting conditions) is biased against — symmetric or near-symmetric drafts and setups preferred.
3. Combo legibility must work in both directions — caster *and* observer must read the elegance.

**Consequences**:
- Shared draft pool becomes a *load-bearing chassis feature*, not decoration.
- Phase B (theme/identity) is briefed under "two minds reading the same combinatorial space" — rules out soloist-wizard themes and faction-versus-faction war themes.
- Future mechanical decisions get a soft preference for B-aligned over A-aligned when otherwise equal.
- The Chassis/Engine lens (established same session) becomes a companion diagnostic.

**Reversal criterion** (updated via Q-D2, 2026-05-26): combined Q-D1 + Q-D2 must both fail across the validation window AND on-ramp interventions (Q-B1, Q-B2, Q-B4) tested without improving either result. A weak game-1 signal followed by a strong game-2 signal counts as "lands at game 2 cadence", not failure.

This is **design intent**, not a mechanical mandate. No immediate rule changes follow from it.""",
     "session-20", None),
]

conn = sqlite3.connect(DB)
cur = conn.cursor()
for r in adrs:
    cur.execute(
        "INSERT INTO adrs (id, n, title, status, body, decided_in, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
        r,
    )
conn.commit()
print(f"Inserted {len(adrs)} ADR rows.")
cur.execute("SELECT id, n, title FROM adrs ORDER BY n;")
for row in cur.fetchall():
    print(f"  {row[0]} (n={row[1]}): {row[2][:60]}")
conn.close()
