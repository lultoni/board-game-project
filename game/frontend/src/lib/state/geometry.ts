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

const SQUARE_SIZE = 100; // viewBox 800 / 8

/** Engine direction indices: 0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW.
 *  SVG y-axis points down, so rank+1 → y decreases (dy = -1). */
const DIR_VECTORS: Array<{ dx: number; dy: number }> = [
  { dx:  0, dy: -1 }, // 0 N
  { dx:  1, dy: -1 }, // 1 NE
  { dx:  1, dy:  0 }, // 2 E
  { dx:  1, dy:  1 }, // 3 SE
  { dx:  0, dy:  1 }, // 4 S
  { dx: -1, dy:  1 }, // 5 SW
  { dx: -1, dy:  0 }, // 6 W
  { dx: -1, dy: -1 }, // 7 NW
];

/**
 * Given the SVG cursor position and the origin square (where the skill caster
 * stands), return the direction index (0–7) whose vector best aligns with the
 * angle from the origin to the cursor. Used to auto-resolve Shove direction
 * from mouse position so the arrow overlay is only needed when truly ambiguous.
 *
 * `flipped` must match the Board's current orientation so the coordinate
 * conversion is correct.
 */
export function pickDirectionByCursor(
  originSq: number,
  cursorX: number,
  cursorY: number,
  legalDirs: number[],
  flipped = false,
): number | null {
  if (legalDirs.length === 0) return null;
  if (legalDirs.length === 1) return legalDirs[0];
  const oFile = fileOf(originSq);
  const oRank = rankOf(originSq);
  const oCX = oFile * SQUARE_SIZE + SQUARE_SIZE / 2;
  const oCY = (flipped ? oRank : 7 - oRank) * SQUARE_SIZE + SQUARE_SIZE / 2;
  const offX = cursorX - oCX;
  const offY = cursorY - oCY;
  const len2 = offX * offX + offY * offY;
  if (len2 < 4) return legalDirs[0]; // cursor on origin — pick first legal
  let best = legalDirs[0];
  let bestScore = -Infinity;
  for (const dir of legalDirs) {
    const v = DIR_VECTORS[dir];
    const score = (offX * v.dx + offY * v.dy) / Math.sqrt(v.dx * v.dx + v.dy * v.dy);
    if (score > bestScore) { bestScore = score; best = dir; }
  }
  return best;
}
