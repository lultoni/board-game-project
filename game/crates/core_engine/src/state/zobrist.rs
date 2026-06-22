//! Zobrist hashing table and incremental update helpers.
//!
//! A static random table is generated once at engine init. The Position's
//! `zobrist` field is XOR-updated on every state change so equality checks
//! and transposition-table lookups stay O(1).
//!
//! # Required key categories (per ADR-005 + audit + session-30 fixup)
//!
//! - Piece-on-square: 64 squares × {P1,P2} × {King,Champion,Guard}
//!                    × HP {1,2} × Armor {0,1,2} × Skill1 {0..15} × Skill2 {0..15}
//!                    (folded by feature where independent — see Stockfish's
//!                    piece+square+colour keying for the pattern.)
//! - Side-to-move: 1 key, XOR'd on every turn flip.
//! - Phase: 1 key (Move vs Skill).
//! - Pending modifier bits: 1 key per bit in `modifier_bits` (Focus, Charge,
//!   plus reserved bits for future modifiers).
//! - Combo counter state: per (square, counter-value) — folded as needed.
//! - moved_this_phase: 64 keys, one per square. Folded into the hash so that
//!   two positions with identical spatial state but different "already moved"
//!   sets don't collide.
//! - round_number: hash modulo the income-scaling cadence (every 5 rounds the
//!   income jumps; identical round-number-mod-5 buckets *can* share keys
//!   safely, but simplest is one key per concrete round_number value, lazy-
//!   computed). Slice 6 decides.
//! - Champion-credit + tracked-enemies: NOT hashed individually. These are
//!   transient turn-state and cleared at end-of-turn. Including them in the
//!   hash would prevent transposition between move orderings that arrive at
//!   the same end-of-turn state; excluding them is safe because they cannot
//!   survive a turn boundary.
//! - Money: design decision — money is a scalar resource that affects
//!   future legality. Hash money explicitly so positions with identical
//!   spatial state but different money don't collide in the TT.

// TODO: PRNG-seeded static table (deterministic seed for reproducibility).
// TODO: Incremental update helpers, one per key category above.
