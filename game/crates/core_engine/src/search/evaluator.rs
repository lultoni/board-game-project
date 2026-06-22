//! Heuristic evaluation for terminal / time-out search nodes.
//!
//! Score convention: positive = P1 advantage, negative = P2 advantage.
//! Win/loss are represented as ±(MATE_SCORE - depth_to_mate) so shorter wins
//! score higher and the search prefers fast mates.
//!
//! ============================================================
//! Design philosophy (load-bearing — read before changing eval)
//! ============================================================
//!
//! Source: designer's eval-function notes (Session 28 inbox, Perplexity
//! transcript). Distilled here because the original file was deleted once
//! its content landed in code. These principles outlive the stub.
//!
//! 1. WIN/LOSS OVERRULES EVERYTHING.
//!    Captured-King = ±MATE_SCORE. Checked before any other term.
//!    Encoded as ±(MATE_SCORE - depth) so a mate-in-2 scores higher than
//!    a mate-in-5. Standard chess-engine convention — get this wrong and
//!    the engine ignores forced mates in favour of positional fluff.
//!
//! 2. "FASTEST PATH" LIVES IN THE SEARCH, NOT IN EVAL.
//!    Do NOT bake depth/tempo-to-resolution into the static evaluation.
//!    Tiebreaks between equal-eval positions are search's job (via the
//!    MATE_SCORE-depth encoding above and via move ordering). Keep eval
//!    pure: it scores the position as-is, ignoring how we got here.
//!
//! 3. AFTER WIN/LOSS: COUNT REAL THINGS.
//!    Material first (pieces, HP, armor, money, equipped skills + their
//!    follow-on possibilities). This is the baseline and MUST beat random
//!    play before anything fancier is added.
//!
//! 4. TWO ANGLES ON EVERY ADVANTAGE — TEMPO AND MONEY.
//!    For each material/positional gain, measure it both ways:
//!      - TEMPO  = how many opponent actions are required to reverse it,
//!                 assuming their best counter-line.
//!      - MONEY  = how much it costs the affected player (given their
//!                 skill flags) to undo it or to compensate for it.
//!    These two angles disagree usefully. A cheap-to-undo gain is worth
//!    less than an expensive-to-undo gain of the same material weight.
//!    Project both forward to an assumed game-end horizon — the longer
//!    the effect persists, the bigger the term.
//!
//! 5. EVAL COST IS A FIRST-CLASS BUDGET.
//!    A 10 ms eval at depth 1 loses to a 0.01 ms eval at depth 6. If the
//!    full tempo+money projection turns out too expensive, fall back to
//!    a simpler eval AND keep the complex one around; diff them on a
//!    suite of random positions to see where they disagree. That diff
//!    is what tells you which terms actually matter.
//!
//! 6. START STUPID.
//!    Material-only first. It will trounce random play and gives every
//!    later term a baseline to prove itself against. Resist the urge to
//!    ship the full tempo/money model on day one — Stockfish's eval grew
//!    over 15+ years, not in one design pass.
//!
//! Implementation order (matches slice plan, slices 7–8 and beyond):
//!   a) terminal: ±MATE_SCORE for captured King.
//!   b) material: pieces + HP + armor + money, weighted.
//!   c) skill-loadout value: equipped skills × follow-on action space.
//!   d) tempo term: opponent-actions-to-revert recent gains.
//!   e) money term: cost-to-undo recent gains.
//!   f) positional hooks (central squares, Champion–Guard adjacency for
//!      Bodyguard) — small bonuses, added last.

use crate::state::Position;

pub const MATE_SCORE: i32 = 1_000_000;

pub fn evaluate(_pos: &Position) -> i32 {
    // TODO(slice-7+): material → resources → tempo/money → positional hooks.
    // See module docs above for the full design philosophy and the order
    // in which terms should be added (start stupid, diff against complex).
    0
}
