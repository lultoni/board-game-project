// Board geometry helpers. Squares are 0..63 with file = sq & 7,
// rank = (sq >> 3) & 7. Mirrors the conventions in core_engine.

export function fileOf(sq: number): number {
  return sq & 7;
}

export function rankOf(sq: number): number {
  return (sq >> 3) & 7;
}

export function chebyshev(a: number, b: number): number {
  const df = Math.abs(fileOf(a) - fileOf(b));
  const dr = Math.abs(rankOf(a) - rankOf(b));
  return Math.max(df, dr);
}

/** All squares within Chebyshev distance `r` of `centre`, excluding `centre`. */
export function ringWithin(centre: number, r: number): number[] {
  if (r <= 0) return [];
  const out: number[] = [];
  const cf = fileOf(centre);
  const cr = rankOf(centre);
  for (let df = -r; df <= r; df++) {
    for (let dr = -r; dr <= r; dr++) {
      if (df === 0 && dr === 0) continue;
      const f = cf + df;
      const rr = cr + dr;
      if (f < 0 || f > 7 || rr < 0 || rr > 7) continue;
      out.push((rr << 3) | f);
    }
  }
  return out;
}
