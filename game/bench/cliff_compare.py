#!/usr/bin/env python3
"""Compare two cliff-corpus sweep result-sets (search-cliff Phase 2 A/B tool).

Unlike compare.py (which diffs a results/ prefix against the top-level
baseline-*.json and expects a fixed-depth6 file), this diffs two arbitrary
results/ prefixes over the TIME budgets only, and surfaces the metrics that
matter for the depth cliff: depth reached under the clock, ebf, and the
quiescence-node blowup (qs_nodes) that is the cliff's signature.

Usage:
  cliff_compare.py <base-prefix> <new-prefix>
  # e.g. cliff_compare.py baseline-cliff-custom s1-cliff-custom
  # reads game/bench/results/<prefix>-time{100,500,1000,3000}ms.json

Exit status is always 0; this is a reporting tool. Read the per-position
lines: a fix WINS on the cliff if depth goes UP on the cliff-danger positions
with qs_nodes DOWN, and nothing on the calm/midgame positions regresses.
"""
import json, sys, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RESULTS = os.path.join(ROOT, "bench", "results")
TAGS = ("time100ms", "time500ms", "time1000ms", "time3000ms")


def load(prefix, tag):
    with open(os.path.join(RESULTS, f"{prefix}-{tag}.json")) as f:
        return {p["id"]: p for p in json.load(f)["positions"]}


def qs(p):
    return p.get("counters", {}).get("qs_nodes", 0)


def main():
    base_prefix, new_prefix = sys.argv[1], sys.argv[2]
    print(f"BASE = {base_prefix}   NEW = {new_prefix}\n")
    for tag in TAGS:
        try:
            b, n = load(base_prefix, tag), load(new_prefix, tag)
        except FileNotFoundError:
            print(f"=== {tag} ===  (missing, skipped)")
            continue
        ids = sorted(b.keys())
        deeper = sum(1 for i in ids if n[i]["depth"] > b[i]["depth"])
        shallower = sum(1 for i in ids if n[i]["depth"] < b[i]["depth"])
        mb = sum(b[i]["depth"] for i in ids) / len(ids)
        mn = sum(n[i]["depth"] for i in ids) / len(ids)
        print(f"=== {tag} ===  mean depth {mb:.2f} -> {mn:.2f}   deeper {deeper} / shallower {shallower}")
        # Per-position: depth + qs-node delta. Show every position where depth
        # changed OR qs-nodes moved >=20%.
        for i in ids:
            bd, nd = b[i]["depth"], n[i]["depth"]
            bq, nq = qs(b[i]), qs(n[i])
            qd = (nq / bq - 1) if bq > 0 else (0.0 if nq == 0 else float("inf"))
            if bd != nd or (bq > 0 and abs(qd) >= 0.20):
                arrow = "↑" if nd > bd else ("↓" if nd < bd else " ")
                qpct = f"{100*qd:+.0f}%" if qd != float("inf") else "new"
                print(f"     {i:<16} d {bd:>2}->{nd:<2} {arrow}   qs {bq:>10,}->{nq:>10,} ({qpct})")
    print()
    print("WIN on cliff = depth ↑ on cliff-danger positions, qs ↓, nothing else shallower.")


if __name__ == "__main__":
    main()
