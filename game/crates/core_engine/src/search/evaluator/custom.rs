//! Custom-evaluator scaffold (ns-55) — the file you edit to build your own eval.
//!
//! This is a complete, registered [`Evaluator`]. Pick it from the setup /
//! settings dropdowns (id `"custom-stub"`) and it runs immediately.
//!
//! ## The model: one contextual score PER PIECE, plus a few side terms
//!
//! This evaluator is **not** a sum of independent add/subtract terms. Its core is
//! [`score_piece`]: ONE function, called once per occupied piece, that returns a
//! single contextual value for that piece. Inside it you start from a base value
//! and let **factors interact** — multiply it up for activity, down for exposure,
//! bend it by conditionals — because the factors are just local variables in one
//! function and can freely read each other. That is the whole point: an exposed
//! champion that is ALSO cut off from its guards can be worth far less than either
//! penalty alone, which a sum of terms can never express.
//!
//! The position total is: **sum of every piece's score (owner-signed) + the side
//! terms**. You never write a `total()` or touch the driver — you write
//! `score_piece` and the side-term fns; the framework walks the board once, signs
//! each piece by its owner, and folds everything.
//!
//! ## Side terms (money, tempo, …) — still write-once
//!
//! Whole-side quantities that aren't about one piece live in [`SIDE_TERMS`]. Each
//! is written ONCE from one side's perspective — `fn(ctx, is_p1) -> i32` returning
//! that side's positive magnitude — and the driver runs it for P1 and P2 and diffs
//! them. Read "my" side's state via `ctx` accessors (e.g. `ctx.money(is_p1)`).
//!
//! ## The shared context — computed once, borrowed everywhere
//!
//! [`CustomCtx::new`] runs once per `evaluate()` before the board walk. Put
//! anything a factor would otherwise recompute per-square in here (occupancy is
//! seeded; add attacker tables / game stage when your activity or safety factors
//! need them). `score_piece` and every side term borrow `&CustomCtx`.
//!
//! ## Panel breakdown (for now: per-piece total only)
//!
//! The hover-card shows each piece's final score. Factor-level decomposition
//! (activity 1.3×, exposure 0.6×, …) is deliberately deferred until the scoring
//! math settles — see the note in [`score_piece`] for the one-line hook to expose
//! a factor when you want it.
//!
//! ## What you may borrow (opt-in — you are NOT forced through the heuristic)
//!
//!   - `crate::search::see::{see_capture, see_single_hit, build_attackers_table}`
//!     — static exchange eval (is this piece hanging?).
//!   - `crate::search::quiescence::is_king_threatened(pos, side)` — one-tempo-
//!     from-death check.
//!   - `super::EvalContext` / `super::EvalParams` — the heuristic's per-call state
//!     and tuned weights, if you ever want to reuse them wholesale.
//!
//! To ship a variant: copy this file, rename the struct, add another
//! `builtin::BUILTINS` line (one edit each).

use crate::state::{MailboxEntry, Position};
use crate::state::position::GameResult;
use super::{BreakdownDetail, EvalReport, Evaluator, MATE_SCORE, PieceTermBreakdown, TermEntry};

/// Your evaluator. Zero-size for now; add fields (tuned weights, a cached table,
/// a loaded model handle) as you flesh it out. Keep it `Send`.
#[derive(Clone, Debug, Default)]
pub struct CustomEvaluator;

/// What kind of piece occupies a square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Guard,
    Champion,
    King,
}

/// One piece on the board, resolved from the bitboards so `score_piece` doesn't
/// have to poke them. `sq` is the square index; `is_p1` its owner; `mb` its
/// mailbox entry (hp / armor / skill ids).
// `sq` / `is_p1` / `mb` are unused until your factors read them — drop this
// `allow` once `score_piece` uses more than `kind`.
#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Piece {
    sq:    u8,
    is_p1: bool,
    kind:  Kind,
    mb:    MailboxEntry,
}

// ============================================================================
// THE SCORER — this is what you edit. One function, called once per piece.
// ============================================================================

/// Score ONE piece in context, as a **positive magnitude for its owner** (the
/// driver applies the owner's sign — P1 positive, P2 negative — so you never
/// think about sides here). Return `0` for a piece you don't want to value.
///
/// This is where interaction lives: compute factors as locals and combine them
/// however you like. They read each other for free because they're in scope.
///
/// ```ignore
/// let base = ...;                       // piece's intrinsic worth
/// let activity = ...;                   // >1.0 mobile, <1.0 stuck
/// let safety   = ...;                   // <1.0 when exposed / hanging
/// // interaction: exposure hurts MORE when the piece is also low-activity
/// let combined = base as f32 * activity * safety;
/// combined.round() as i32
/// ```
///
/// To surface a factor in the hover-card later, we'll add a `ctx.note(...)` hook;
/// for now only the returned total is shown.
// `ctx` is unused until a factor reads precomputed state — drop this `allow` then.
#[allow(unused_variables)]
fn score_piece(ctx: &CustomCtx, p: Piece) -> i32 {
    // ---- base value ---------------------------------------------------------
    // The king isn't scored as material (its capture is the MATE branch, handled
    // before any piece runs); give it 0 base and value it via safety factors.
    let base: i32 = match p.kind {
        Kind::King     => 0,
        Kind::Champion => 1000,
        Kind::Guard    => 600,
    };

    // ---- factors (add your own; they interact freely as locals) -------------
    // Each factor is a multiplier around 1.0. Start neutral; introduce real
    // factors (activity, exposure, guard-link, threat) one at a time. Example
    // interaction to build toward:
    //
    //   let activity = ctx.activity_mult(&p);   // e.g. 0.8 ..= 1.3
    //   let safety   = ctx.safety_mult(&p);     // e.g. 0.5 ..= 1.0
    //   // an exposed AND inactive piece is punished more than either alone:
    //   let combined = if activity < 1.0 && safety < 1.0 { activity * safety * 0.9 }
    //                  else { activity * safety };

    // Neutral for now — pure base value until you add factors.
    let combined: f32 = 1.0;

    (base as f32 * combined).round() as i32
}

// ============================================================================
// SIDE TERMS — whole-side quantities, written once, driven for both players.
// ============================================================================

/// A side-level term: score ONE side as a positive magnitude. Written once from
/// one side's perspective; the driver calls it for P1 and P2 and diffs them.
struct SideTerm {
    name: &'static str,
    sign: i32,
    f:    fn(ctx: &CustomCtx, is_p1: bool) -> i32,
}

/// Side-level terms, in report order. ADD A LINE to register a term.
const SIDE_TERMS: &[SideTerm] = &[
    SideTerm { name: "money", sign: 1, f: term_money },
];

/// Money: this side's treasury, scaled. Written once for "my" side; the driver
/// runs it for both players and takes the difference.
fn term_money(ctx: &CustomCtx, is_p1: bool) -> i32 {
    const MONEY: i32 = 25;
    ctx.money(is_p1) * MONEY
}

// ============================================================================
// THE SHARED CONTEXT — computed once per eval, borrowed by the scorer + terms.
// ============================================================================

/// Everything the scorer / terms might want precomputed, built once in
/// [`CustomCtx::new`]. Seeded with the cheap occupancy bitboards; add attacker
/// tables, game stage, availability lookups, and factor helpers here as your
/// scoring grows to need them.
struct CustomCtx<'a> {
    pos:     &'a Position,
    /// All occupied squares (both players).
    all_occ: u64,
    // --- add precomputed state your factors need, e.g.: ---
    // atk:   AttackersTable,           // build_attackers_table(pos)
    // stage: GameStage,                // opening / mid / end
}

impl<'a> CustomCtx<'a> {
    fn new(pos: &'a Position) -> Self {
        CustomCtx {
            pos,
            all_occ: pos.p1_pieces.0 | pos.p2_pieces.0,
        }
    }

    /// Resolve the piece kind at an occupied square.
    #[inline]
    fn kind_at(&self, sq: u8) -> Kind {
        let mask = 1u64 << sq;
        if self.pos.kings.0 & mask != 0 {
            Kind::King
        } else if self.pos.champions.0 & mask != 0 {
            Kind::Champion
        } else {
            Kind::Guard
        }
    }

    // --- per-side accessors: read "my" side's state from a side term ---------
    // Each returns the value for the side `is_p1` selects, so a side term is
    // written once and the driver runs it for both players. Add one per per-side
    // field your terms need (they all follow this p1/p2 pattern).

    /// This side's treasury.
    #[inline]
    fn money(&self, is_p1: bool) -> i32 {
        if is_p1 { self.pos.p1_money as i32 } else { self.pos.p2_money as i32 }
    }

    // Factor helpers for `score_piece` go here as you build them, e.g.:
    // fn activity_mult(&self, p: &Piece) -> f32 { ... }
    // fn safety_mult(&self, p: &Piece)   -> f32 { ... }
}

// ============================================================================
// THE DRIVER — you should never need to touch below this line.
// Walks the board once, scores each piece + each side term, and produces BOTH
// the scalar total and the breakdown from the same pass.
// ============================================================================

/// A side term's accumulated `(p1_magnitude, p2_magnitude)`.
#[derive(Clone, Copy, Default)]
struct Sums {
    p1: i32,
    p2: i32,
}

/// Result of one board walk: the piece-score total (owner-signed), the side-term
/// sums, the grand total, and — when requested — the per-piece breakdown rows.
struct Scored {
    piece_total: i32,
    side_sums:   Vec<Sums>,
    total:       i32,
    rows:        Option<Vec<PieceTermBreakdown>>,
}

impl CustomEvaluator {
    /// The single scoring pass. `score_piece` runs on every occupied square;
    /// every side term runs once per side. `with_rows` also assembles the
    /// per-piece breakdown. Both `evaluate` and `evaluate_report` call this, so
    /// the score and the report are the same numbers by construction.
    fn score(&self, pos: &Position, with_rows: bool) -> Scored {
        let ctx = CustomCtx::new(pos);

        let mut piece_total = 0i32;
        let mut rows: Option<Vec<PieceTermBreakdown>> =
            with_rows.then(|| Vec::with_capacity(ctx.all_occ.count_ones() as usize));

        // One board pass: score each occupied piece in context, sign by owner.
        let mut bits = ctx.all_occ;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let mask = 1u64 << sq;
            let is_p1 = pos.p1_pieces.0 & mask != 0;
            let kind = ctx.kind_at(sq);
            let mb = pos.mailbox[sq as usize];
            let piece = Piece { sq, is_p1, kind, mb };

            let mag = score_piece(&ctx, piece);
            let owner_signed = if is_p1 { mag } else { -mag };
            piece_total += owner_signed;

            if let Some(rows) = rows.as_mut() {
                let piece_kind = match kind { Kind::King => 3, Kind::Champion => 2, Kind::Guard => 1 };
                rows.push(PieceTermBreakdown {
                    sq, is_p1, piece_kind,
                    hp: mb.hp(), armor: mb.armor(),
                    skill1_id: mb.skill1(), skill2_id: mb.skill2(),
                    // Per-piece total only for now — no factor decomposition yet.
                    terms: Vec::new(),
                    piece_total: owner_signed,
                });
            }
        }

        // Side terms: run each once per side (written once, driven for both).
        let side_sums: Vec<Sums> = SIDE_TERMS.iter()
            .map(|t| Sums { p1: (t.f)(&ctx, true), p2: (t.f)(&ctx, false) })
            .collect();

        let mut total = piece_total;
        for (i, term) in SIDE_TERMS.iter().enumerate() {
            total += term.sign * (side_sums[i].p1 - side_sums[i].p2);
        }

        Scored { piece_total, side_sums, total, rows }
    }
}

impl Evaluator for CustomEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 {
        match terminal_score(pos) {
            Some(s) => s,
            None => self.score(pos, false).total,
        }
    }

    fn evaluate_report(&self, pos: &Position, detail: BreakdownDetail) -> EvalReport {
        if let Some(s) = terminal_score(pos) {
            return EvalReport::terminal(s);
        }

        let want_rows = matches!(detail, BreakdownDetail::PerPiece);
        let scored = self.score(pos, want_rows);

        // The per-piece scoring is one aggregate "pieces" term (owner-signed sum);
        // its per-piece decomposition lives in `pieces`. Side terms are listed
        // individually. A term with zero magnitude on both sides is omitted.
        let mut terms = Vec::new();
        if scored.piece_total != 0 {
            terms.push(TermEntry {
                name: "pieces".to_string(),
                p1: scored.piece_total.max(0),
                p2: (-scored.piece_total).max(0),
                signed: scored.piece_total,
            });
        }

        let side_terms = SIDE_TERMS.iter().zip(&scored.side_sums)
            .filter(|(_, s)| s.p1 != 0 || s.p2 != 0)
            .map(|(t, s)| TermEntry {
                name: t.name.to_string(),
                p1: s.p1, p2: s.p2,
                signed: t.sign * (s.p1 - s.p2),
            })
            .collect();

        EvalReport {
            terms,
            side_terms,
            pieces: scored.rows,
            total: scored.total, // == evaluate(): same walk, same numbers
            terminal: false,
        }
    }
}

/// Terminal shortcut: `Some(±MATE_SCORE)` if the game is decided, else `None`.
/// Shared by both trait methods so they can never disagree on terminals.
#[inline]
fn terminal_score(pos: &Position) -> Option<i32> {
    match pos.game_result {
        Some(GameResult::P1Wins) => Some(MATE_SCORE),
        Some(GameResult::P2Wins) => Some(-MATE_SCORE),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_scores_and_reports_consistently() {
        let pos = Position::setup_stack_m();
        let ev = CustomEvaluator;
        let r = ev.evaluate_report(&pos, BreakdownDetail::Aggregate);
        assert_eq!(r.total, ev.evaluate(&pos), "report total must equal evaluate()");
        assert!(!r.terminal);
    }

    #[test]
    fn custom_per_piece_rows_sum_to_pieces_term() {
        // The per-piece rows must reconstruct the aggregate "pieces" term.
        let pos = Position::setup_stack_m();
        let ev = CustomEvaluator;
        let r = ev.evaluate_report(&pos, BreakdownDetail::PerPiece);
        let rows = r.pieces.expect("PerPiece requested");
        let row_sum: i32 = rows.iter().map(|row| row.piece_total).sum();
        let pieces = r.terms.iter().find(|t| t.name == "pieces").map(|t| t.signed).unwrap_or(0);
        assert_eq!(row_sum, pieces);
    }

    #[test]
    fn custom_terminal() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        let r = CustomEvaluator.evaluate_report(&pos, BreakdownDetail::PerPiece);
        assert!(r.terminal);
        assert_eq!(r.total, MATE_SCORE);
    }
}
