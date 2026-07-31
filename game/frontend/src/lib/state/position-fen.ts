// Pure FEN-board manipulation for the Position Builder. Kept out of the
// Svelte component so the string logic is unit-testable without the engine
// (which is Tauri-only). See fen.rs for the canonical FEN grammar.
//
// Field layout (space-separated):
//   0 board  1 to_move  2 phase  3 actions  4 p1_money  5 p2_money
//   6 modifiers  7 round  8 moved_this_phase  [9.. turn-scoped trailers]

/** Parse the board section of a FEN into a 64-element array of piece tokens.
 *  Each token is a string like "K", "c", "C[1/2/0/7/15]", or "" for empty. */
/** Split a board section into its 8 rank strings on the `/` separator, but
 *  NOT on `/` characters that appear inside a mailbox bracket. Mailbox tokens
 *  like `G[1/0/0/0/0]` contain slashes, so a naive `boardStr.split("/")`
 *  shatters bracketed pieces and misaligns every rank after them — which
 *  silently dropped pieces (e.g. a king) on any edited position. */
function splitRanks(boardStr: string): string[] {
  const ranks: string[] = [];
  let cur = "";
  let depth = 0;
  for (const ch of boardStr) {
    if (ch === "[") depth++;
    else if (ch === "]") depth = Math.max(0, depth - 1);
    if (ch === "/" && depth === 0) {
      ranks.push(cur);
      cur = "";
    } else {
      cur += ch;
    }
  }
  ranks.push(cur);
  return ranks;
}

export function parseBoardSection(boardStr: string): string[] {
  const squares: string[] = new Array(64).fill("");
  const ranks = splitRanks(boardStr);
  for (let r = 0; r < 8; r++) {
    const rank = ranks[r] ?? "";
    // rank 0 of FEN = rank 8 = board row index 7 (top)
    const rankIdx = 7 - r;
    let file = 0;
    let i = 0;
    while (i < rank.length && file < 8) {
      const ch = rank[i];
      if (ch >= "1" && ch <= "8") {
        file += parseInt(ch, 10);
        i++;
      } else if ("KCGkcg".includes(ch)) {
        let token = ch;
        if (rank[i + 1] === "[") {
          const end = rank.indexOf("]", i + 2);
          if (end !== -1) {
            token = rank.slice(i, end + 1);
            i = end + 1;
          } else {
            i++;
          }
        } else {
          i++;
        }
        squares[rankIdx * 8 + file] = token;
        file++;
      } else {
        i++;
      }
    }
  }
  return squares;
}

/** Encode a 64-element squares array back to the board FEN section. */
export function encodeBoardSection(squares: string[]): string {
  const ranks: string[] = [];
  for (let r = 7; r >= 0; r--) {
    let rank = "";
    let empty = 0;
    for (let f = 0; f < 8; f++) {
      const token = squares[r * 8 + f];
      if (token === "") {
        empty++;
      } else {
        if (empty > 0) { rank += empty; empty = 0; }
        rank += token;
      }
    }
    if (empty > 0) rank += empty;
    ranks.push(rank);
  }
  return ranks.join("/");
}

/** Extract piece info from a token string. */
export function parseToken(token: string): { char: string; hp: number; armor: number; s1: number; s2: number } {
  const char = token[0];
  const bracketMatch = token.match(/\[(\d+)\/(\d+)\/(\d+)\/(\d+)\/(\d+)\]/);
  if (bracketMatch) {
    return {
      char,
      hp: parseInt(bracketMatch[1], 10),
      armor: parseInt(bracketMatch[2], 10),
      s1: parseInt(bracketMatch[4], 10),
      s2: parseInt(bracketMatch[5], 10),
    };
  }
  return { char, hp: 2, armor: 0, s1: 0, s2: 0 };
}

/** Build a token string from piece info. Omit brackets if all defaults. */
export function buildToken(char: string, hp: number, armor: number, s1: number, s2: number): string {
  if (hp === 2 && armor === 0 && s1 === 0 && s2 === 0) return char;
  return `${char}[${hp}/${armor}/0/${s1}/${s2}]`;
}

/**
 * Apply a mutation to the board squares of `fen` and return a *static*
 * between-turns FEN: the board is re-encoded, `moved_this_phase` (field 8) is
 * forced to `0x0`, and any turn-scoped trailer fields (9+) are dropped.
 *
 * This is the fix for the Position Builder round-trip: moving or removing a
 * piece that was flagged in `moved_this_phase` would otherwise leave a bit that
 * no longer overlaps the side-to-move bitboard, which the engine rejects with
 * BadDecimal{field:"moved_this_phase"}. Because a builder edit represents a
 * fresh static position, zeroing the tracker is always correct.
 */
export function mutateBoardToStaticFen(fen: string, mutateFn: (squares: string[]) => void): string {
  const parts = fen.split(/\s+/);
  const squares = parseBoardSection(parts[0]);
  mutateFn(squares);
  parts[0] = encodeBoardSection(squares);
  if (parts.length > 8) parts[8] = "0x0";
  return parts.slice(0, 9).join(" ");
}
