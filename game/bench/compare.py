#!/usr/bin/env python3
"""Compare a sweep result-set against the committed baseline.

Usage: compare.py <prefix>        # compares game/bench/results/<prefix>-*.json
                                  # vs game/bench/baseline-*.json

Reports, per budget:
  - depth6:  geo-mean NPS, total d6 wall-clock, count positions over 1000ms,
             and any position that crossed the 1s line in either direction.
  - timeNms: mean depth, positions deeper/shallower vs baseline.
"""
import json, sys, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BENCH = os.path.join(ROOT, "bench")

def load(path):
    with open(path) as f:
        return json.load(f)

def pos_map(doc):
    return {p["id"]: p for p in doc["positions"]}

def cmp_depth6(prefix):
    base = load(os.path.join(BENCH, "baseline-depth6.json"))
    new = load(os.path.join(BENCH, "results", f"{prefix}-depth6.json"))
    b, n = pos_map(base), pos_map(new)
    b_over = sorted([p for p in b.values() if p["time_ms"] > 1000], key=lambda x: -x["time_ms"])
    n_over = sorted([p for p in n.values() if p["time_ms"] > 1000], key=lambda x: -x["time_ms"])
    b_total = sum(p["time_ms"] for p in b.values())
    n_total = sum(p["time_ms"] for p in n.values())
    print("=== depth6 ===")
    print(f"geo-NPS   base {base['aggregate']['geometric_mean_nps']:>12,.0f}  ->  new {new['aggregate']['geometric_mean_nps']:>12,.0f}  ({100*(new['aggregate']['geometric_mean_nps']/base['aggregate']['geometric_mean_nps']-1):+.1f}%)")
    print(f"total d6  base {b_total:>10,.0f}ms  ->  new {n_total:>10,.0f}ms  ({100*(n_total/b_total-1):+.1f}%)")
    print(f"over-1s   base {len(b_over):>2}  ->  new {len(n_over):>2}")
    # per-position crossings and big deltas
    crossed_over, crossed_under, big = [], [], []
    for id_ in b:
        bt, nt = b[id_]["time_ms"], n[id_]["time_ms"]
        if bt <= 1000 < nt: crossed_over.append((id_, bt, nt))
        if nt <= 1000 < bt: crossed_under.append((id_, bt, nt))
        if bt > 0 and abs(nt/bt - 1) >= 0.20:
            big.append((id_, bt, nt, nt/bt - 1))
    if crossed_over:
        print("  !! CROSSED OVER 1s (regressions):")
        for id_, bt, nt in crossed_over: print(f"     {id_:<28} {bt:>8.0f} -> {nt:>8.0f}ms")
    if crossed_under:
        print("  :) crossed UNDER 1s:")
        for id_, bt, nt in crossed_under: print(f"     {id_:<28} {bt:>8.0f} -> {nt:>8.0f}ms")
    big.sort(key=lambda x: x[3])
    if big:
        print("  per-position deltas >=20%:")
        for id_, bt, nt, d in big: print(f"     {id_:<28} {bt:>8.0f} -> {nt:>8.0f}ms  ({100*d:+.0f}%)")
    print(f"  worst new: " + ", ".join(f"{p['id']}={p['time_ms']:.0f}ms" for p in n_over[:5]))
    return len(n_over) <= len(b_over) and n_total < b_total, len(n_over), n_total

def cmp_time(prefix, tag):
    base = load(os.path.join(BENCH, f"baseline-{tag}.json"))
    new = load(os.path.join(BENCH, "results", f"{prefix}-{tag}.json"))
    b, n = pos_map(base), pos_map(new)
    deeper = sum(1 for i in b if n[i]["depth"] > b[i]["depth"])
    shallower = sum(1 for i in b if n[i]["depth"] < b[i]["depth"])
    mb = sum(b[i]["depth"] for i in b)/len(b)
    mn = sum(n[i]["depth"] for i in b)/len(b)
    print(f"=== {tag} ===  mean depth {mb:.2f} -> {mn:.2f}   deeper {deeper} / shallower {shallower}")

if __name__ == "__main__":
    prefix = sys.argv[1]
    ok, over, total = cmp_depth6(prefix)
    for tag in ("time100ms", "time500ms", "time1000ms", "time3000ms"):
        try: cmp_time(prefix, tag)
        except FileNotFoundError: pass
    print()
    print(f"NET WIN (over-1s not up AND total down): {ok}")
