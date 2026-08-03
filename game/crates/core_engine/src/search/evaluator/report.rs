//! Breakdown types (ns-53) — the shared eval wire format.
//!
//! [`EvalReport`] is the dynamic breakdown an evaluator produces on demand: a
//! flat list of aggregate terms, the side-level terms, and (optionally) a
//! per-piece decomposition. It lives at the evaluator top level because any
//! evaluator may produce one — the heuristic builds its per-piece rows from the
//! SAME term pass that produces its scalar total, so for it consistency is
//! definitional; an evaluator with no term structure returns [`EvalReport::single`].
//!
//! The report is `Serialize`/`Deserialize` — it is the wire format sent to the
//! frontend eval panel. It is NOT persisted in match logs (per-ply breakdowns
//! were removed as redundant: a log already stores each position's FEN + scalar
//! eval, so any term decomposition is recomputable live).

/// One term's aggregate contribution across the whole board.
///
/// `p1`/`p2` are the raw per-side magnitudes (always positive); `signed` is the
/// term's contribution to `total` with its sign/weight already applied (so a
/// penalty term reports a negative `signed`, and a weighted term reports the
/// weighted value).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TermEntry {
    pub name:   String,
    pub p1:     i32,
    pub p2:     i32,
    pub signed: i32,
}

/// One piece's per-term contributions on one square. Only the per-piece terms
/// appear here; side-level terms live in [`EvalReport::side_terms`]. `signed` on
/// each entry is the owner-signed contribution (P1 positive, P2 negative) so a
/// consumer can sum `pieces[*].piece_total` + `side_terms[*].signed` == `total`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PieceTermBreakdown {
    pub sq:         u8,
    pub is_p1:      bool,
    /// 1 = guard, 2 = champion, 3 = king.
    pub piece_kind: u8,
    pub hp:         u8,
    pub armor:      u8,
    pub skill1_id:  u8,
    pub skill2_id:  u8,
    /// Per-piece term magnitudes for THIS piece. `signed` is owner-signed.
    pub terms:      Vec<TermEntry>,
    /// Owner-signed sum of this piece's per-piece terms (P1 positive, P2 negative).
    pub piece_total: i32,
}

/// The evaluator's dynamic breakdown. Aggregate always; per-piece optional.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvalReport {
    /// Aggregate per-piece terms (summed over the board), active terms only.
    pub terms:      Vec<TermEntry>,
    /// Side-level terms (money / tempo / offensive_range / …), active terms only.
    pub side_terms: Vec<TermEntry>,
    /// Per-piece decomposition — `Some` only when [`BreakdownDetail::PerPiece`]
    /// was requested. One entry per occupied square.
    pub pieces:     Option<Vec<PieceTermBreakdown>>,
    /// `evaluate()` for this position (P1-POV; `±MATE_SCORE` for terminals).
    pub total:      i32,
    /// True when the position is terminal (no terms run).
    pub terminal:   bool,
}

impl EvalReport {
    /// A terminal report — no terms, just the mate total.
    pub fn terminal(total: i32) -> Self {
        EvalReport { terms: Vec::new(), side_terms: Vec::new(), pieces: None, total, terminal: true }
    }

    /// A single-term report for evaluators with no term structure (NNUE/dense
    /// raters). The whole score is one synthetic `name` term. `pieces` is `None`
    /// (an NN has no per-piece decomposition).
    pub fn single(name: &str, total: i32) -> Self {
        EvalReport {
            terms: vec![TermEntry {
                name: name.to_string(),
                p1: total.max(0),
                p2: (-total).max(0),
                signed: total,
            }],
            side_terms: Vec::new(),
            pieces: None,
            total,
            terminal: false,
        }
    }
}

/// How much detail to compute when producing an [`EvalReport`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakdownDetail {
    /// Aggregate + side-level terms only (`pieces: None`). Cheaper.
    Aggregate,
    /// Also produce the per-piece decomposition (`pieces: Some`).
    PerPiece,
}
