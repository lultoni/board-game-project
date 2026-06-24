// Regex helpers for the multiplayer resume handshake. The engine's MatchLog
// JSON contains u64 Zobrist hashes that exceed JS's safe-integer range, so we
// cannot round-trip the log through JSON.parse without lossy bigint coercion.
// Both helpers therefore extract the raw decimal digit-string directly from
// the JSON text and leave string comparison to the caller.
//
// Pinned to MatchLog JSON v1 (see core_engine/src/session.rs). If the
// `match-log` serde representation changes — renamed fields, restructured
// nesting, base64 zobrist encoding — update these regexes AND the pinning
// tests in `multiplayer-resume.test.ts`.

// MatchLog ply_no is 1-indexed (core_engine/src/session.rs:360); plyCount=0 → start_zobrist.
export function extractPostZobristForPly(
  matchLogJson: string,
  n: number,
): string | null {
  const plyRe = new RegExp(`"ply_no"\\s*:\\s*${n}\\b`);
  const m = plyRe.exec(matchLogJson);
  if (!m) return null;
  const tail = matchLogJson.slice(m.index);
  const zRe = /"post_zobrist"\s*:\s*(\d+)/;
  const z = zRe.exec(tail);
  return z ? z[1] : null;
}

export function extractStartZobrist(matchLogJson: string): string | null {
  const m = /"start_zobrist"\s*:\s*(\d+)/.exec(matchLogJson);
  return m ? m[1] : null;
}

/** Compute the (plyCount, zobrist) pair to send in a resume-request from a
 *  persisted MatchLog. Returns `{plyCount: 0, zobrist: "0"}` when the log
 *  shape is unexpected so the host can still respond (with start_zobrist
 *  comparison) or reject cleanly. */
export function extractResumeStateFromLog(
  logJson: string,
): { plyCount: number; zobrist: string } {
  const tpMatch = /"total_plies"\s*:\s*(\d+)/.exec(logJson);
  const plyCount = tpMatch ? parseInt(tpMatch[1], 10) : 0;
  if (plyCount === 0) {
    const sz = extractStartZobrist(logJson);
    return { plyCount: 0, zobrist: sz ?? "0" };
  }
  const pz = extractPostZobristForPly(logJson, plyCount);
  return { plyCount, zobrist: pz ?? "0" };
}
