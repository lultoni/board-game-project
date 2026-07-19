// u16 mailbox entry (mirrors `core_engine/src/state/mailbox.rs`).
//
//   bits  0..2   HP        (0..=2)
//   bits  2..4   Armor     (0..=2)
//   bits  4..7   Combo     (0..=7)
//   bits  7..11  Skill 1   (4 bits)
//   bits 11..15  Skill 2   (4 bits)
//   bit  15      reserved
//
// Empty square = 0x0000.

export interface MailboxEntry {
  empty: boolean;
  hp: number;
  armor: number;
  combo: number;
  skill1: number;
  skill2: number;
}

export function decodeMailbox(u16: number): MailboxEntry {
  const v = u16 & 0xffff;
  return {
    empty: v === 0,
    hp: v & 0x3,
    armor: (v >> 2) & 0x3,
    combo: (v >> 4) & 0x7,
    skill1: (v >> 7) & 0xf,
    skill2: (v >> 11) & 0xf,
  };
}

/**
 * Bitboard layout: [p1, p2, kings, champions, guards].
 * Returns a per-square description derived from bitboards + mailbox.
 */
export type PieceKind = "king" | "champion" | "guard";
export type Owner = "p1" | "p2";

export interface BoardPiece extends MailboxEntry {
  square: number;
  owner: Owner;
  kind: PieceKind;
}

export function squareToFileRank(sq: number): { file: number; rank: number } {
  return { file: sq & 7, rank: (sq >> 3) & 7 };
}

export function formatSquare(sq: number): string {
  const file = String.fromCharCode("a".charCodeAt(0) + (sq % 8));
  const rank = Math.floor(sq / 8) + 1;
  return `${file}${rank}`;
}

export function bitboardHas(bb: bigint, sq: number): boolean {
  return ((bb >> BigInt(sq)) & 1n) === 1n;
}

/** Iterate set bits of a bitboard. */
export function* bitsOf(bb: bigint): IterableIterator<number> {
  let b = bb;
  while (b !== 0n) {
    const lsb = b & -b;
    // Math.log2 is sufficient up to 53 bits but a board is 64.
    // Use trailingZeros via bit twiddle.
    let n = 0n;
    let v = lsb;
    while ((v & 0xffffffffn) === 0n) {
      n += 32n;
      v >>= 32n;
    }
    let v32 = Number(v & 0xffffffffn);
    while ((v32 & 1) === 0) {
      n += 1n;
      v32 >>= 1;
    }
    yield Number(n);
    b ^= lsb;
  }
}

/**
 * Build a typed list of pieces from bitboards + mailbox.
 * Returns one BoardPiece per non-empty square.
 */
export function readPieces(
  bitboards: BigUint64Array | bigint[],
  mailbox: Uint16Array | number[],
): BoardPiece[] {
  const bb = (i: number): bigint =>
    typeof bitboards[i] === "bigint"
      ? (bitboards[i] as bigint)
      : BigInt(bitboards[i] as unknown as number);
  const p1 = bb(0);
  const p2 = bb(1);
  const kings = bb(2);
  const champions = bb(3);
  const guards = bb(4);

  const pieces: BoardPiece[] = [];
  for (let sq = 0; sq < 64; sq++) {
    const inP1 = bitboardHas(p1, sq);
    const inP2 = bitboardHas(p2, sq);
    if (!inP1 && !inP2) continue;
    const owner: Owner = inP1 ? "p1" : "p2";
    const kind: PieceKind = bitboardHas(kings, sq)
      ? "king"
      : bitboardHas(champions, sq)
        ? "champion"
        : bitboardHas(guards, sq)
          ? "guard"
          : "guard";
    const cell = decodeMailbox(mailbox[sq]);
    pieces.push({ square: sq, owner, kind, ...cell });
  }
  return pieces;
}
