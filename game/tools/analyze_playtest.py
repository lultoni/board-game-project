#!/usr/bin/env python3
"""
analyze_playtest.py — extract design-relevant player-behaviour metrics from a
`boardgame-bundle-v1` telemetry export (the "send to designer" blob).

The bundle is one giant single-line JSON. This tool never prints the raw blob;
it walks the structured ply records and the FEN fingerprints and emits a compact
report you can read into a playtest analysis.

WHAT IT READS (schema owned by game/crates/core_engine/src/telemetry.rs):
  bundle.logs[]                    one MatchLog per game
    .config, .start_fen
    .final_result                  "P1Win" | "P2Win" | "Draw" | "Aborted"
    .plies[]                       one PlyRecord per applied action
      .seat_player  "P1"|"P2"
      .seat_kind    "Human"|"Ai"
      .thought_ms
      .legal_count                 branching factor at this ply
      .action { kind, src, target, skill_id, skill_name, picks }
      .prev_fen / .post_fen        full position (see fen.rs grammar)
      .prev_static_eval / .post_static_eval
      .prev_breakdown / .post_breakdown   25-field eval decomposition
      .post_phase   "Move"|"Skill"|"Draft"
      .post_round
      .post_p1_money / .post_p2_money
      .post_tracked_enemies[]      combo counters live on enemy squares
      .post_tracked_casters[]      which casters already ticked this turn

FEN square index: sq = rank*8 + file, file0=a rank0=P1-back. sq%8=file, sq//8=rank.
Piece token: K/C/G (P1 upper), k/c/g (P2 lower), optional [hp/armor/combo/s1/s2].

Usage:
  python3 game/tools/analyze_playtest.py <bundle.json>
  python3 game/tools/analyze_playtest.py <bundle.json> --game 2      # one game
  python3 game/tools/analyze_playtest.py <bundle.json> --combo-trace # audit combo bonus
  python3 game/tools/analyze_playtest.py <bundle.json> --json        # machine-readable
"""

import argparse
import json
import sys
from collections import Counter, defaultdict

# --- skill id -> name (game/crates/core_engine/src/game_logic/skills.rs) -----
SKILL_NAME = {
    1: "Lance", 2: "Hook", 3: "Break", 4: "Steal", 5: "Tempest",
    6: "Shield", 7: "Heal", 8: "Plate", 9: "Dash", 10: "Blast",
    11: "Shove", 12: "Swap", 13: "Retreat", 14: "Focus", 15: "Charge",
}
SKILL_MONEY = {
    1: 2, 2: 3, 3: 2, 4: 4, 5: 4, 6: 2, 7: 3, 8: 3, 9: 3,
    10: 2, 11: 3, 12: 4, 13: 4, 14: 1, 15: 3,
}
STRIKE = {"Lance", "Hook", "Break", "Steal", "Tempest"}
MOVEMENT = {"Blast", "Shove", "Swap", "Hook", "Tempest", "Dash", "Retreat"}  # movement-causing
MOVE_TARGET = {"Blast", "Shove", "Swap", "Hook", "Tempest"}  # moves an ENEMY (combo-tick eligible)
BUFF = {"Shield", "Plate", "Focus", "Charge", "Heal"}


def sq_name(sq):
    if sq is None or sq >= 64:
        return "?"
    return f"{chr(ord('a') + sq % 8)}{sq // 8 + 1}"


# --- FEN decode -------------------------------------------------------------
class Piece:
    __slots__ = ("sq", "p1", "kind", "hp", "armor", "combo", "s1", "s2")

    def __init__(self, sq, p1, kind, hp, armor, combo, s1, s2):
        self.sq, self.p1, self.kind = sq, p1, kind
        self.hp, self.armor, self.combo = hp, armor, combo
        self.s1, self.s2 = s1, s2


def parse_fen_board(fen):
    """Return {sq: Piece} for the board portion of a FEN string."""
    board = fen.split(" ", 1)[0]
    # split ranks respecting [...] brackets
    ranks, depth, start = [], 0, 0
    for i, c in enumerate(board):
        if c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
        elif c == "/" and depth == 0:
            ranks.append(board[start:i]); start = i + 1
    ranks.append(board[start:])
    pieces = {}
    for top_idx, rank_str in enumerate(ranks):
        rank = 7 - top_idx
        file = 0
        i = 0
        while i < len(rank_str):
            c = rank_str[i]
            if c.isdigit():
                file += int(c); i += 1; continue
            p1 = c.isupper()
            kind = {"k": "K", "c": "C", "g": "G"}[c.lower()]
            hp, armor, combo, s1, s2 = 2, 0, 0, 0, 0
            if i + 1 < len(rank_str) and rank_str[i + 1] == "[":
                close = rank_str.index("]", i)
                nums = [int(x) for x in rank_str[i + 2:close].split("/")]
                hp, armor, combo, s1, s2 = nums
                i = close + 1
            else:
                i += 1
            sq = rank * 8 + file
            pieces[sq] = Piece(sq, p1, kind, hp, armor, combo, s1, s2)
            file += 1
    return pieces


def piece_census(pieces):
    """Counts + total HP + total armor per side."""
    out = {
        "p1": {"K": 0, "C": 0, "G": 0, "hp": 0, "armor": 0},
        "p2": {"K": 0, "C": 0, "G": 0, "hp": 0, "armor": 0},
    }
    for pc in pieces.values():
        side = "p1" if pc.p1 else "p2"
        out[side][pc.kind] += 1
        out[side]["hp"] += pc.hp
        out[side]["armor"] += pc.armor
    return out


# --- turn / round grouping --------------------------------------------------
def skill_name_of(action):
    if action.get("skill_name"):
        return action["skill_name"]
    sid = action.get("skill_id", 0)
    return SKILL_NAME.get(sid)


def analyze_game(log, idx):
    plies = log.get("plies", [])
    cfg = log.get("config", {})
    result = log.get("final_result")
    start = parse_fen_board(log["start_fen"])
    start_census = piece_census(start)

    # per-player accumulators
    pdata = {
        "P1": new_player_acc(),
        "P2": new_player_acc(),
    }

    draft_picks = {"P1": [], "P2": []}
    max_round = 0
    move_attacks = {"P1": 0, "P2": 0}
    skill_acts = {"P1": 0, "P2": 0}
    endphase = {"P1": 0, "P2": 0}
    endturn = {"P1": 0, "P2": 0}
    thought_total = {"P1": 0, "P2": 0}
    thought_max = {"P1": (0, 0), "P2": (0, 0)}  # (ms, ply)
    legal_samples = []  # (round, phase, legal_count)

    # capture / event log built from FEN census diffs
    events = []  # (round, ply_no, seat, text)
    prev_census = start_census
    combo_bonus_events = []  # plies where bonus damage likely applied

    for p in plies:
        seat = p["seat_player"]
        kind = p["action"]["kind"]
        rnd = p.get("post_round", 0)
        max_round = max(max_round, rnd)
        tm = p.get("thought_ms", 0)
        if seat in thought_total:
            thought_total[seat] += tm
            if tm > thought_max[seat][0]:
                thought_max[seat] = (tm, p["ply_no"])
        legal_samples.append((rnd, p.get("post_phase"), p.get("legal_count", 0), seat))

        if kind == "DraftTurn":
            for pick in (p["action"].get("picks") or []):
                nm = pick.get("skill_name") or SKILL_NAME.get(pick.get("skill_id"), "?")
                draft_picks[seat].append((nm, sq_name(pick.get("sq")), pick.get("slot")))
            continue
        if kind == "Move":
            # move-attack vs plain move: did an enemy lose hp / piece count drop?
            post = parse_fen_board(p["post_fen"])
            pre = parse_fen_board(p["prev_fen"])
            if is_move_attack(pre, post, seat):
                move_attacks[seat] += 1
                pdata[seat]["move_attacks"] += 1
            continue
        if kind == "Skill":
            nm = skill_name_of(p["action"]) or "?"
            skill_acts[seat] += 1
            acc = pdata[seat]
            acc["skill_uses"][nm] += 1
            acc["skill_rounds"][nm].add(rnd)
            acc["money_spent"] += SKILL_MONEY.get(p["action"].get("skill_id", 0), 0)
            # combo-bonus detection: target had counter>0 in prev_tracked_enemies
            tgt = p["action"].get("target")
            pre_tracked = set(p.get("prev_tracked_enemies") or [])
            # prev_tracked isn't in schema; reconstruct from previous ply below
            continue
        if kind == "EndPhase":
            endphase[seat] += 1
        elif kind == "EndTurn":
            endturn[seat] += 1

        # census diff -> capture events
        post = parse_fen_board(p["post_fen"])
        cur = piece_census(post)
        for side in ("p1", "p2"):
            for k in ("K", "C", "G"):
                if cur[side][k] < prev_census[side][k]:
                    n = prev_census[side][k] - cur[side][k]
                    events.append((rnd, p["ply_no"], seat,
                                   f"{n}x {side.upper()} {k} removed"))
        prev_census = cur

    # first blood timings from events
    first_guard_death = next((e[0] for e in events if "G removed" in e[3]), None)
    first_champ_death = next((e[0] for e in events if "C removed" in e[3]), None)

    final = parse_fen_board(log["final_fen"]) if log.get("final_fen") else {}
    final_census = piece_census(final) if final else prev_census

    return {
        "idx": idx,
        "config": {"p1": cfg.get("p1"), "p2": cfg.get("p2")},
        "result": result,
        "rounds": max_round,
        "plies": len(plies),
        "start_census": start_census,
        "final_census": final_census,
        "draft_picks": draft_picks,
        "move_attacks": move_attacks,
        "skill_acts": skill_acts,
        "endphase": endphase,
        "endturn": endturn,
        "thought_total": thought_total,
        "thought_max": thought_max,
        "legal_samples": legal_samples,
        "events": events,
        "first_guard_death": first_guard_death,
        "first_champ_death": first_champ_death,
        "pdata": pdata,
    }


def new_player_acc():
    return {
        "skill_uses": Counter(),
        "skill_rounds": defaultdict(set),
        "move_attacks": 0,
        "money_spent": 0,
    }


def is_move_attack(pre, post, seat):
    """A Move ply is a move-attack if an enemy piece lost HP or was removed."""
    enemy_is_p1 = (seat == "P2")
    # total enemy hp before/after
    def enemy_hp(pieces):
        return sum(pc.hp for pc in pieces.values() if pc.p1 == enemy_is_p1)
    def enemy_count(pieces):
        return sum(1 for pc in pieces.values() if pc.p1 == enemy_is_p1)
    return enemy_hp(post) < enemy_hp(pre) or enemy_count(post) < enemy_count(pre)


# --- combo-bonus audit: reconstruct tracked_enemies across a turn -----------
def combo_trace(log):
    """
    Walk plies, following post_tracked_enemies + per-square combo counter in the
    FEN, and flag every Skill ply where the target already carried a counter > 0
    (=> bonus damage should apply). Also reports HP delta on the target so you
    can eyeball whether the engine actually applied the bonus.
    """
    lines = []
    for p in log.get("plies", []):
        if p["action"]["kind"] != "Skill":
            continue
        tgt = p["action"].get("target")
        if tgt is None:
            continue
        pre = parse_fen_board(p["prev_fen"])
        post = parse_fen_board(p["post_fen"])
        tpre = pre.get(tgt)
        tpost = post.get(tgt)
        # combo counter carried on the target square in prev FEN:
        counter = tpre.combo if tpre else 0
        # hp lost by whatever now sits at tgt (may be removed)
        hp_pre = tpre.hp if tpre else None
        hp_post = tpost.hp if tpost else 0  # removed => 0
        armor_pre = tpre.armor if tpre else 0
        armor_post = tpost.armor if tpost else 0
        removed = (tgt in pre) and (tgt not in post)
        nm = skill_name_of(p["action"]) or "?"
        # damage dealt to target = armor lost + hp lost (+ removal)
        if tpre is not None:
            dmg = (armor_pre - armor_post) + (hp_pre - (0 if removed else hp_post))
        else:
            dmg = None
        lines.append({
            "ply": p["ply_no"], "round": p.get("post_round"), "seat": p["seat_player"],
            "skill": nm, "target": sq_name(tgt),
            "target_combo_pre": counter,
            "hp_pre": hp_pre, "hp_post": (None if removed else hp_post), "removed": removed,
            "armor_pre": armor_pre, "armor_post": armor_post,
            "est_damage": dmg,
            "post_tracked_enemies": p.get("post_tracked_enemies"),
            "post_tracked_casters": p.get("post_tracked_casters"),
        })
    return lines


# --- reporting --------------------------------------------------------------
def pct(n, d):
    return f"{100*n/d:.0f}%" if d else "-"


def print_game_report(g):
    p1k, p2k = g["config"]["p1"], g["config"]["p2"]
    print(f"\n{'='*70}\nGAME {g['idx']}  ({p1k} vs {p2k})   result={g['result']}   "
          f"rounds={g['rounds']}   plies={g['plies']}")
    print(f"{'='*70}")

    # length / shape
    print("\n-- Length & material arc --")
    sc, fc = g["start_census"], g["final_census"]
    for side, lbl in (("p1", "P1"), ("p2", "P2")):
        s, f = sc[side], fc[side]
        print(f"  {lbl}: K {s['K']}->{f['K']}  C {s['C']}->{f['C']}  "
              f"G {s['G']}->{f['G']}   (HP {s['hp']}->{f['hp']}, armor final {f['armor']})")
    print(f"  first Guard death: R{g['first_guard_death']}   "
          f"first Champion death: R{g['first_champ_death']}")

    # captures timeline (compressed)
    print("\n-- Capture timeline --")
    if g["events"]:
        for (rnd, ply, seat, txt) in g["events"]:
            print(f"  R{rnd:<3} ply{ply:<4} by {seat}: {txt}")
    else:
        print("  (no captures)")

    # action balance
    print("\n-- Action balance (per side) --")
    for seat in ("P1", "P2"):
        ma = g["move_attacks"][seat]
        sa = g["skill_acts"][seat]
        ratio = f"{ma/sa:.2f}" if sa else "-"
        print(f"  {seat}: move-attacks={ma}  skill-activations={sa}  MA/skill={ratio}")

    # skill usage
    print("\n-- Skill usage (per side) --")
    for seat in ("P1", "P2"):
        acc = g["pdata"][seat]
        uses = acc["skill_uses"]
        print(f"  {seat}  (money on skills≈{acc['money_spent']}):")
        if not uses:
            print("    (none)")
        for nm, n in uses.most_common():
            rounds = sorted(acc["skill_rounds"][nm])
            rng = f"R{rounds[0]}-R{rounds[-1]}" if len(rounds) > 1 else f"R{rounds[0]}" if rounds else "-"
            over = "  <-- 3+ uses" if n >= 3 else ""
            print(f"    {nm:<9} {n:>2}x  ({rng}){over}")
        unused = sorted(set(SKILL_NAME.values()) - set(uses))
        # only report skills the player actually drafted as "unused"
        drafted = {pk[0] for pk in g["draft_picks"][seat]}
        never = sorted(drafted - set(uses))
        if never:
            print(f"    drafted-but-never-used: {', '.join(never)}")

    # draft
    print("\n-- Draft loadouts --")
    for seat in ("P1", "P2"):
        picks = g["draft_picks"][seat]
        if picks:
            byskill = Counter(pk[0] for pk in picks)
            summ = ", ".join(f"{n}x{s}" if c > 1 else s for (s, c), n in
                             [((s, c), c) for s, c in byskill.items()])
            print(f"  {seat}: {summ}")

    # thinking / branching
    print("\n-- Thinking & branching --")
    for seat in ("P1", "P2"):
        tt = g["thought_total"][seat]
        tmax, tply = g["thought_max"][seat]
        print(f"  {seat}: total think {tt/1000:.0f}s   longest single {tmax/1000:.1f}s (ply {tply})")
    # branching factor by phase
    move_bf = [lc for (_, ph, lc, _) in g["legal_samples"] if ph == "Move"]
    skill_bf = [lc for (_, ph, lc, _) in g["legal_samples"] if ph == "Skill"]
    def stats(xs):
        if not xs:
            return "-"
        return f"avg {sum(xs)/len(xs):.0f}, max {max(xs)}"
    print(f"  legal-action count: Move-phase {stats(move_bf)}  |  Skill-phase {stats(skill_bf)}")


def print_combo_trace(log, idx):
    print(f"\n{'#'*70}\nCOMBO-BONUS AUDIT — GAME {idx}")
    print("Flagging Skill plies whose target already carried a combo counter>0")
    print("in the pre-move FEN. est_damage = armor lost + hp lost. Compare vs the")
    print("skill's base damage to see whether the +counter bonus was applied.")
    print(f"{'#'*70}")
    rows = combo_trace(log)
    flagged = [r for r in rows if (r["target_combo_pre"] or 0) > 0]
    if not flagged:
        print("  No skill targeted a counter-loaded enemy (counter stayed 0 in all pre-FENs).")
        print("  NOTE: the FEN 'combo' field is turn-scoped; if the engine clears it before")
        print("  the snapshot, use post_tracked_enemies below instead.")
    for r in flagged:
        print(f"  R{r['round']} ply{r['ply']} {r['seat']} {r['skill']}->{r['target']}: "
              f"counter_pre={r['target_combo_pre']} "
              f"hp {r['hp_pre']}->{r['hp_post']} armor {r['armor_pre']}->{r['armor_post']} "
              f"removed={r['removed']} est_dmg={r['est_damage']}")
    # Also dump the tracked-enemy evolution so combo mechanics can be audited
    print("\n  -- tracked_enemies / tracked_casters evolution (Skill plies) --")
    for r in rows:
        te = r["post_tracked_enemies"]
        tc = r["post_tracked_casters"]
        if te or tc:
            print(f"    R{r['round']} ply{r['ply']} {r['seat']} {r['skill']}->{r['target']}: "
                  f"tracked_enemies={[sq_name(x) for x in (te or [])]} "
                  f"tracked_casters={[sq_name(x) for x in (tc or [])]}")


def main():
    ap = argparse.ArgumentParser(description="Analyze a boardgame-bundle-v1 telemetry export.")
    ap.add_argument("bundle", help="path to the *.json bundle")
    ap.add_argument("--game", type=int, default=None, help="1-based game index (default: all)")
    ap.add_argument("--combo-trace", action="store_true", help="audit combo-bonus application")
    ap.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    args = ap.parse_args()

    with open(args.bundle) as fh:
        bundle = json.load(fh)

    if bundle.get("schema") != "boardgame-bundle-v1":
        print(f"WARNING: unexpected schema {bundle.get('schema')!r}", file=sys.stderr)

    logs = bundle.get("logs", [])
    games = []
    for i, log in enumerate(logs, 1):
        if args.game and i != args.game:
            continue
        games.append((i, log, analyze_game(log, i)))

    if args.json:
        # strip the un-serialisable pieces for JSON output
        out = []
        for (i, log, g) in games:
            gg = dict(g)
            gg["pdata"] = {s: {"skill_uses": dict(a["skill_uses"]),
                               "move_attacks": a["move_attacks"],
                               "money_spent": a["money_spent"]}
                           for s, a in g["pdata"].items()}
            gg.pop("legal_samples", None)
            out.append(gg)
        print(json.dumps(out, indent=2, default=list))
        return

    print(f"Bundle: {len(logs)} game(s), engine {logs[0].get('engine_version') if logs else '?'}")
    for (i, log, g) in games:
        print_game_report(g)
        if args.combo_trace:
            print_combo_trace(log, i)


if __name__ == "__main__":
    main()
