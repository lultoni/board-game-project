#!/usr/bin/env python3
"""Migrate the table rows in mechanics-evaluated.md → mechanics table.

Each table section maps to a verdict:
  - Accepted — In Current Baseline → 'accepted' (or 'baseline')
  - Accepted — Pending Test → 'staged'
  - Deferred → 'pending'
  - Withdrawn / Rejected → 'rejected' (or 'withdrawn')
  - Reopened / Under Review → 'pending'
  - Methodology / Design Decisions → 'accepted' (treated as accepted methodology decisions)
"""

import re
import sqlite3
from pathlib import Path

ROOT = Path("/Users/I750319/passion-projects/board-game-project")
DB = ROOT / "design" / "design.db"

# Hand-curated extraction from the mechanics-evaluated.md tables.
# (id, name, verdict, source_oq, body, decided_in)

rows = [
    # ---------- Accepted in baseline ----------
    ("mech-10x10-board", "10×10 board (baseline)", "baseline", "oq-1",
     "10×10 board accepted as baseline after P1 confirmed size felt right for both players.", "session-1"),
    ("mech-unlinked-move-skill", "Unlinked Move + Skill phases", "baseline", "oq-3",
     "Move Phase and Skill Phase are unlinked. Intuitive and appreciated by both players in P1.", "session-1"),
    ("mech-path-blocked-all", "Skill path blocked by all pieces (ally + opponent)", "baseline", "oq-5",
     "Both players confirmed in P1. Ruling logged Session 3.", "session-3"),
    ("mech-money-start-own-turn", "Money income at start of each player's OWN turn", "baseline", None,
     "Resolves ambiguity — not end of round. Session 3 ruling.", "session-3"),
    ("mech-round-1-no-income", "Round 1 has no Money income (starting Money only)", "baseline", None,
     "Clarifies first turn. Session 3 ruling.", "session-3"),
    ("mech-move-attack-survival", "Move-attack survival: attacker stops before target", "baseline", None,
     "Attacker occupies target tile only if target is removed. Session 3 ruling.", "session-3"),
    ("mech-bodyguard-move-attacks-only", "Bodyguard intercepts Move-Attacks only, not skills", "baseline", None,
     "Skills always hit directly. Session 3 ruling.", "session-3"),
    ("mech-healing-no-cap", "Healing: no cap", "baseline", None,
     "Keeps it simple. Session 3 ruling.", "session-3"),
    ("mech-money-no-cap", "Money cap: none", "baseline", "oq-8",
     "No hoarding observed across P1, P2, P3. Naturally spends down. Closed from monitoring after P3.", "session-3"),
    ("mech-free-pathing-movement", "Free pathing for movement (any route ≤ speed)", "baseline", "oq-28",
     "Cannot pass through any piece. Session 3 ruling.", "session-3"),
    ("mech-defender-chooses-guard", "Defender chooses which Guard intercepts", "baseline", "oq-21",
     "P1 suggestion accepted as ruling.", "session-1"),
    ("mech-no-terrain", "No terrain effects", "baseline", "oq-15",
     "Confirmed overhead complexity; removed. Reversible as 'map variant' expansion.", "session-1"),
    ("mech-economy-fix-l1", "Layer 1 economy: 6 start Money, +2/turn, scaling every 5 rounds", "baseline", "oq-17",
     "Fixes dead opening; skills come online Turn 1. Accepted from Playtest 2 analysis.", "session-5"),
    ("mech-one-move-per-phase", "Each piece moved at most once per Move Phase", "baseline", "oq-29",
     "Now explicit in all rule sheets. P2 mid-game ruling.", "session-5"),
    ("mech-default-range-2", "Default Skill Range = 2 (Range 0=self, 1=adjacent, 2=2 tiles)", "baseline", "oq-30",
     "Now defined in all rule sheets. P2 mid-game ruling.", "session-5"),
    ("mech-focus-charge-timing", "Focus / Charge timing & scope", "baseline", "oq-31",
     "Focus must come first; Charge retroactive. P2 mid-game ruling.", "session-5"),
    ("mech-move-skills-no-damage", "Skills that move pieces deal NO damage", "baseline", "oq-32",
     "General rule in Skill System section. P2 mid-game ruling.", "session-5"),
    ("mech-tempest-no-self-affect", "Blade Tempest does NOT affect caster", "baseline", "oq-33",
     "Only target takes 1 damage. P2 mid-game ruling.", "session-5"),
    ("mech-move-attack-1-damage", "Move-attack deals 1 damage (instead of 2)", "baseline", "oq-37",
     "P3: first Champion kill R11 vs P2's R26; standoff dissolved; combat feel 'Better/Much Better'. Accepted into baseline Session 15.", "session-15"),
    ("mech-range-system-clarification", "Range system clarification (self/adjacent/default)", "baseline", "oq-10",
     "All skills default to Range 2 unless text names 'self' or 'adjacent.' Range modifiers apply from default. Self/adjacent cannot be shifted inward by Range buffs. Injured penalty does not affect self/adjacent skills.", "session-16"),
    ("mech-focus-self-adjacent", "Focus boosting self/adjacent skills", "baseline", "oq-31",
     "Can boost self→adjacent and adjacent→Range 2. Range and Injured calculated independently. Session 16 ruling.", "session-16"),
    ("mech-focus-on-move-skills", "Focus on Move skills: caster chooses activation OR effect range", "baseline", None,
     "The +1 from Focus applies to either the activation range or the effect range — caster's choice at activation, not both. Resolves ambiguity for Move-skill / Focus interaction; preserves combo variety without making any single combo dominant.", "session-18"),
    ("mech-high-concept-framing-b", "High-concept framing: 'Two minds, one puzzle' (Framing B)", "accepted", "oq-39",
     "Decided in ADR-004. See `adrs` table. Locks design intent: 2-player nature is load-bearing, combo legibility must work in both directions, asymmetry biased against. Design intent only — no immediate rule changes.", "session-20"),
    ("mech-move-attack-reframe", "Move-Attack reframed as 'a Move that ends on an enemy tile'", "baseline", None,
     "Reworded Move Phase intro and Move-Attack opening to make move-attack unity explicit and plant skill-first thinking. Survival-stop rule strengthened with explicit attacker-speed cases (Guard speed 2 → 1 tile moved; Champion/King speed 1 → 0 tiles moved). Pure framing change — no mechanical change. BASELINE_VERSION bumped 2026-05-26.", "session-20"),
    ("mech-combo-bonus-in-baseline", "Multi-Champion Combo Bonus migrated into baseline", "baseline", "oq-38",
     "Concise version of the rule promoted into `section-multi-champion-combo()` — and added as a row in the quick-reference table. Niko (P4 first-time player) skipped reading the long version and still understood the rule, proving the dense version is sufficient. Worked examples + tracking tables stay out of baseline. BASELINE_VERSION bumped 2026-05-26 → 2026-05-30.", "session-23"),

    # ---------- Accepted — Pending Test ----------
    ("mech-combo-bonus-strike-only", "Multi-Champion combo bonus — Strike-only, single-counter (Stack A G2)", "staged", "oq-38",
     "+1 damage on 2nd+ Strike hit when 2+ different Champions target same enemy this turn. Stack A G2 data was mixed: confirmed in mechanics, weak in feel. Session 22 reframe: Q3 softness is design-aligned (few-times-a-game payoff). Lever is scope, not strength.", "session-22"),
    ("mech-combo-dual-counter-stack-a-g3", "Combo bonus dual-counter + widened scope (Stack A G3)", "staged", "oq-38",
     "Two parallel counters per turn: target counter (different friendly Champions hitting same enemy) + attacker counter (same friendly Champion hitting different enemy targets). Any skill that hits an enemy piece counts (not Strike-only). Multi-target ticks all hits. Move-Attacks excluded. Both counters stack. Justification: cross-category crowd-out, late-game offensive lockout, exchange-pit pattern. Methodology: gated behind Stack H so chassis trim lands first.", "session-22"),
    ("mech-unified-ap-system", "Unified AP system (3 AP/turn) — Stack G", "staged", "oq-26",
     "Cleaner decisions; merges Move + Skill phases into one action pool. Stack G is dormant.", "session-9"),
    ("mech-armor-cap-2-plate-2", "Armor cap 3→2 + Plate +1→+2 (bundled) — Stack H", "staged", "oq-11",
     "Reduces chassis-loop volume (Framing B alignment); Plate becomes one-shot fortify rather than stack-grind. P4 confirmed Q-C1 chassis-volume hypothesis. Session 25: ABSORBED INTO STACK M — only the cap-2 component, not Plate +2. Stack H exists as isolation-fallback only.", "session-25"),
    ("mech-injured-no-penalties", "Injured: remove mechanical downsides (no speed cap, no Range −1) — Stack J → Stack M", "staged", "oq-57",
     "Tests whether Injured's chassis volume (speed cap, Range −1, Range-modifier interactions, self/adjacent carve-out) pays for itself in game-feel terms. State still exists as HP-tracker. P4 partially confirmed: Niko named Injured as confusion source on first read; payoff felt thin to experienced player. Absorbed into Stack M Session 25.", "session-25"),
    ("mech-stack-m-game-length-cut", "Stack M — Game Length Cut (six bundled changes)", "staged", "oq-66",
     "Six simultaneous changes: (1) board 10×10→8×8; (2) Armor cap 3→2; (3) Injured penalties removed (still 2 HP tracker); (4) draw conditions removed entirely; (5) Steal cost 3→4 both Modes; (6) Multi-Champion Combo Bonus also ticks on movement-causing skills (Tempest push, Hook pull, Blast push, Shove, Swap when relocating enemy). Hypothesis: single coordinated cut delivers 30-60 min length + single-climax shape (Principle 8) without breaking combo fantasy. Each lever maps to a specific compounding curve in the 12-economy map. Per-axis rollback routing in the rule sheet handles each failure mode surgically. Session 26 expanded the combo lever further: bonus damage also applies to any skill on a counter-loaded target, not only Strikes.", "session-25"),

    # ---------- Deferred ----------
    ("mech-damage-escalation-late", "Damage escalation after Round X", "pending", "oq-19",
     "Only if games are still 25+ rounds after Stacks A–C.", "session-1"),
    ("mech-three-move-actions", "3 Move actions per turn (vs current 2)", "pending", "oq-23",
     "May be superseded by AP system (Stack G).", "session-1"),
    ("mech-restricted-movement", "Restricted movement (straight-line only)", "rejected", "oq-28",
     "Layer 6 candidate — would make Move skills stronger. Now parked indefinitely.", "session-8"),
    ("mech-board-8x8-plus-fewer-pieces", "Board 8×8 with fewer pieces", "pending", "oq-1b",
     "Decoupled Session 22: 8×8 lives in Stack D (Board Geometry); piece count reduction lives in Stack K. Test independently. Note: 8×8 absorbed into Stack M.", "session-22"),
    ("mech-pool-draft-variant", "Pool draft variant", "pending", "oq-35",
     "Test after Stack A/B accepted.", "session-8"),
    ("mech-flexible-piece-placement", "Flexible piece placement", "pending", "oq-36",
     "Bundled with OQ-48; test after Stack A/B.", "session-8"),
    ("mech-piece-placement-post-draft", "Piece placement order — post-draft", "pending", "oq-48",
     "Bundled with OQ-36; test after Stack A/B.", "session-8"),
    ("mech-path-idea-2-guards-only", "Path obstruction Idea 2 (only opponent Guards block)", "pending", "oq-49",
     "Conditional on Stack A/B frustration data.", "session-8"),
    ("mech-minor-major-skill-cost", "Minor / Major skill action cost", "pending", "oq-50",
     "Pre-work needed: design ultimate-skill candidates first.", "session-8"),
    ("mech-cascade-trigger", "Cascade trigger (+1 action on kill)", "pending", "oq-51",
     "Anti-snowball one-turn tempo bonus on kill. Backpocket; test in Stack F.", "session-11"),
    ("mech-pin-threatened", "Pin / Threatened restriction", "pending", "oq-51",
     "Restriction-as-reward. Backpocket; needs own stack.", "session-11"),
    ("mech-collision-damage", "Collision damage (universal push-into-piece = 1 damage)", "pending", "oq-51",
     "Conditional on standoff resolution.", "session-11"),

    # ---------- Withdrawn / Rejected ----------
    ("mech-stack-l-pole-b-consumable", "Stack L — Pole B per-turn-draft prototype (consumable variant)", "withdrawn", "oq-61",
     "Session 25 (2026-06-21) after P5. First playtest surfaced three structural problems: Armor 3 still felt mandatory (cross-pole confirmation of OQ-11); pure-reaction play with no multi-turn planning (breaks Principle 4); felt-PI broke under combinatorial breadth even though formal-PI held. Pole A returns as Active. Pole B as a *direction* is paused, not killed.", "session-25"),
    ("mech-bodyguard-defender-adjacency", "Bodyguard: adjacent to defender only; Guard takes damage (Stack B)", "withdrawn", "oq-21",
     "Session 22 (2026-05-29). P4 confirmed Bodyguard tracks standoff state, not the rule (0 triggers when Armor stalling returned). Defender-only adjacency unlikely to be the right fix.", "session-22"),
    ("mech-yinsh-capture-penalty", "YINSH-inspired capture penalty", "rejected", "oq-19",
     "Creates asymmetric cost — if one player runs out of Guards, only the other player pays the penalty. Punishes playing correctly.", "session-1"),
    ("mech-no-board-card-fighter", "No board (Direction B — card fighter)", "rejected", None,
     "Loses spatial skills (push/pull/swap). 'Just another card game.' Designer rejected.", "session-1"),
    ("mech-zone-lane-system", "Zone/lane system (Direction C — spatial hybrid)", "rejected", None,
     "Direction A+ chosen instead; grid preserved.", "session-1"),
    ("mech-terrain-effects", "Terrain effects (Water/Forest/Plains/Mountains)", "rejected", "oq-15",
     "Confirmed overhead complexity; removed. Reversible as 'map variant' expansion.", "session-1"),
    ("mech-linked-move-action", "Linked Movement-Action (move to act)", "rejected", "oq-3",
     "Unlinked preferred; likely superseded by AP system.", "session-1"),
    ("mech-3-hp-champions", "3 HP for Champions/King (Guards stay 2 HP)", "rejected", "oq-18",
     "Would extend game (first Champion kill at R26 with 2 HP). Guards at 2 HP vs Champions at 3 HP creates artificial tier.", "session-6"),
    ("mech-performance-money-gain", "Performance-based Money gain", "rejected", "oq-47",
     "Forces single playstyle, constrains creative expression. Auto-economy is strategy-neutral. KPI problem — rewards symptoms, not systems.", "session-8"),
    ("mech-economy-skills-as-actions", "Economy skills as actions", "rejected", "oq-25",
     "2-slot scarcity makes this unworkable. Could be post-v1 expansion variant.", "session-8"),
    ("mech-3rd-skill-slot", "3rd skill slot per Champion", "rejected", None,
     "2 slots forces specialist builds; 3 risks generalist meta. Fix for 'narrow variety' is better skill design, not more slots.", "session-8"),
    ("mech-cr-draft-picks", "CR-style draft picks (strict interleaving)", "rejected", "oq-43",
     "Restricts free strategy with small catalogue. Variant material.", "session-8"),
    ("mech-ban-phase-draft", "Ban phase in draft", "rejected", "oq-44",
     "From older game version with unique fixed-skill Champions. Needs 20+ skills and a different draft model.", "session-8"),
    ("mech-starting-player-bid", "Starting player bid (hidden Money auction)", "rejected", "oq-45",
     "No first-player advantage observed. If surfaces, use Go-style komi instead.", "session-8"),
    ("mech-path-idea-1-opp-blocks", "Path Idea 1 (only opponent pieces block)", "rejected", "oq-49",
     "Creates turtle meta — cluster all pieces, use skills from safety.", "session-7"),
    ("mech-coordinated-movement-bonus", "Coordinated movement bonus (−1 Money if pieces move to same zone)", "rejected", "oq-51",
     "Too easy to trigger accidentally; doesn't reward cleverness.", "session-11"),
    ("mech-breakthrough-bonus", "Breakthrough bonus (+1 Slot on first Champion hit)", "rejected", "oq-51",
     "Subsumed into cascade trigger; arbitrary 'first hit' trigger less elegant.", "session-11"),
    ("mech-checkmate-win-condition", "Checkmate-style win condition", "rejected", "oq-19",
     "Verification burden too high — too many defensive options (heal, armor, push, LoS block) to prove '100% lost' at the table.", "session-11"),
    ("mech-class-based-skill-pools", "Class-based skill pools (Champion's class determines drafting)", "rejected", None,
     "Restricts strategy freedom for minimal complexity reduction. Contradicts blank-slate Champion design.", "session-10"),
    ("mech-champion-pre-naming", "Champion pre-naming (Blacksmith, Necromancer, etc.)", "rejected", None,
     "Champions are blank slates — identity emerges from equipped skills.", "session-10"),
    ("mech-scouting-skill", "Information/scouting skill (reveal Money count)", "rejected", None,
     "Irrelevant in a perfect-information game.", "session-10"),

    # ---------- Reopened ----------
    ("mech-hex-grid", "Hex grid", "pending", "oq-42",
     "Original Session 1 rejection was by omission: ADR-001 confirmed 'grid over card-fighter,' not 'square over hex.' Hex IS a grid variant. Reopened Session 8. Research needed before scheduling a test stack.", "session-8"),

    # ---------- Methodology / Design Decisions ----------
    ("mech-armor-diagnosis-survival-tax", "Defense / Armor's role — diagnosis: late-game survival tax", "accepted", "oq-11",
     "Three diagnoses tested for the late-game Armor problem. A (Money curve too steep): KILLED. B (HP too thin): KILLED. C (Armor's shape is wrong, functions as late-game survival tax / mandatory upkeep): CONFIRMED. User verbatim: 'i 100% agree that armor is like the tax you have to pay.' Diagnosis anchor — not a candidate fix.", "session-23"),
    ("mech-pole-framing", "Pole framing — parallel design tracks (Pole A pre-game-draft, Pole B per-turn-draft)", "accepted", "oq-61",
     "While core identity is unsettled, design proceeds along two parallel tracks rather than tweaking variables inside one rule set. Pole A = current game. Pole B = radical alternative (skills added during play; reusable while equipped; 12 equipped cap; shared action slots; no Money activation gate). Pairs with Principle 7. Pole B paused after P5 (Session 25) but direction not killed.", "session-23"),
    ("mech-pole-b-outcome-pole-a-return", "Pole B prototype outcome → return to Pole A track", "accepted", "oq-61",
     "First Pole B digital prototype run surfaced three structural problems (Armor still felt mandatory; pure-reaction play; felt-PI broke). Designer call: return to Pole A as Active track with two sub-goals (onboard new players via pre-made loadouts; drastically shorten game to 30-60 min). Pole B paused, not abandoned; other Pole B variants remain alive.", "session-25"),
    ("mech-cross-pole-fix-per-pole-revival", "Cross-pole shared fix — per-pole-revival (not once-and-carry)", "accepted", "oq-63",
     "P5 confirmed Armor 3 feels mandatory in both Pole A (P4) and Pole B (P5). Designer's lean was 'twice for cleanness.' With Pole B paused, Stack H runs in Pole A only. If Pole B is ever revived, Stack H (or its successor) runs again there. Carry-forward is per-pole-revival, not once-and-carry.", "session-25"),
    ("mech-stack-m-bundled-deviation-justified", "Stack M bundled-deviation methodology justification", "accepted", None,
     "Stack M intentionally violates the Incremental Testing Methodology by bundling six simultaneous changes. Justified by Principle 7 (fundamental shifts while core unsettled), schedule cost of 6 sequential isolation stacks, prior independent validation of each component, and per-axis rollback routing that preserves attribution. Methodology recovers on the next stack.", "session-25"),
    ("mech-stack-m-combo-scope-expansion", "Stack M combo-bonus scope expansion: any skill on a counter-loaded target", "staged", "oq-38",
     "The Session 25 Stack M draft had movement-causing skills tick the target counter but reserved *bonus damage* for Strike hits only. Session 26 expanded this: any skill (Strike OR movement-causing) on a counter-loaded target deals +counter bonus damage. Unlocks damage strategies without Strike skills. Counter-tick rules unchanged. First rollback if dominant: revert to Session 25 draft (movement ticks counter, Strike-only deals bonus damage).", "session-26"),
]

conn = sqlite3.connect(DB)
cur = conn.cursor()
ok = 0
fail = []
for r in rows:
    try:
        cur.execute(
            "INSERT INTO mechanics (id, name, verdict, source_oq, body, decided_in) VALUES (?, ?, ?, ?, ?, ?)",
            r,
        )
        ok += 1
    except Exception as e:
        fail.append((r[0], str(e)))
conn.commit()
print(f"Inserted {ok} mechanic rows.")
if fail:
    print("Failures:")
    for f in fail:
        print(f"  {f[0]}: {f[1]}")

cur.execute("SELECT verdict, COUNT(*) FROM mechanics GROUP BY verdict ORDER BY verdict;")
print("\nBy verdict:")
for v, c in cur.fetchall():
    print(f"  {v:<14} {c}")
conn.close()
