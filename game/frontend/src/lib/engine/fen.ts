// Client-side FEN rewrite for the makeshift draft.
//
// The engine's FEN format encodes per-square mailbox state as a bracketed
// tail on each piece token: `C[hp/armor/combo/s1/s2]`. An "empty" bracket
// (default 2/0/0/0/0) is omitted. To inject loadouts we walk the board half
// of a fresh starting FEN, find King/Champion tokens at the target squares,
// and rewrite their bracket.
//
// See `core_engine/src/state/fen.rs` for the canonical grammar.

import type { LoadoutMap } from "$lib/state/draft";

// Piece chars: K/k = King, C/c = Champion, G/g = Guard (upper = P1).
const PIECE_CHARS = new Set(["K", "k", "C", "c", "G", "g"]);
const SKILLABLE = new Set(["K", "k", "C", "c"]);

/**
 * Rewrite the board half of a FEN, replacing the mailbox bracket on each
 * King/Champion at a square present in `loadouts`. Other tokens pass through
 * verbatim. The non-board fields (to_move, phase, money, …) are preserved.
 *
 * Assumes the FEN comes from a fresh starting position (HP=2, armor=0,
 * combo=0). For makeshift use only — does not handle mid-game state.
 */
export function rewriteFenWithLoadouts(
  fen: string,
  loadouts: LoadoutMap,
): string {
  const firstSpace = fen.indexOf(" ");
  if (firstSpace < 0) throw new Error("malformed FEN: no fields");
  const board = fen.slice(0, firstSpace);
  const tail = fen.slice(firstSpace); // includes leading space

  const ranks = board.split("/");
  if (ranks.length !== 8) throw new Error(`malformed FEN: ${ranks.length} ranks`);

  // Top-of-FEN is rank 8 → bitboard rank 7. Walk rank-by-rank, file-by-file.
  const out: string[] = [];
  for (let rankTop = 0; rankTop < 8; rankTop++) {
    const bbRank = 7 - rankTop;
    out.push(rewriteRank(ranks[rankTop], bbRank, loadouts));
  }
  return out.join("/") + tail;
}

function rewriteRank(
  rank: string,
  bbRank: number,
  loadouts: LoadoutMap,
): string {
  let i = 0;
  let file = 0;
  let out = "";

  while (i < rank.length) {
    const ch = rank[i];

    // Run-length empties.
    if (ch >= "1" && ch <= "8") {
      out += ch;
      file += Number(ch);
      i++;
      continue;
    }

    if (!PIECE_CHARS.has(ch)) {
      throw new Error(`unexpected char in FEN rank: '${ch}'`);
    }

    // Piece token. Look ahead for an optional bracket.
    out += ch;
    i++;
    let bracket = "";
    if (rank[i] === "[") {
      const close = rank.indexOf("]", i);
      if (close < 0) throw new Error("unterminated bracket in FEN");
      bracket = rank.slice(i, close + 1);
      i = close + 1;
    }

    const sq = bbRank * 8 + file;
    const overrideLoadout = SKILLABLE.has(ch) ? loadouts.get(sq) : undefined;

    if (overrideLoadout) {
      const [s1, s2] = overrideLoadout;
      // Replace (or insert) the bracket. The makeshift only touches skill
      // slots, so HP=2 armor=0 combo=0 is fine.
      out += `[2/0/0/${s1}/${s2}]`;
    } else {
      out += bracket;
    }

    file += 1;
  }

  if (file !== 8) throw new Error(`rank covered ${file} squares, expected 8`);
  return out;
}
