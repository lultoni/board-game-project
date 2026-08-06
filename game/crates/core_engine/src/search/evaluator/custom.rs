//! Custom evaluator (ns-55) — the designer's hand-rolled per-piece eval, and the
//! shipped default. This is the file you edit to build the evaluation.
//!
//! This is a complete, registered [`Evaluator`], selected by default. It is also
//! pickable from the setup / settings dropdowns (id `"custom"`).
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
use crate::state::magic::{cheby_dist, king_expand, skill_attacks, between, on_ray, within_range};
use crate::game_logic::skills::{skill_from_id, skill_category, skill_default_range, skill_cost, Skill, SkillCategory};
use super::{BreakdownDetail, EvalReport, Evaluator, MATE_SCORE, PieceTermBreakdown, TermEntry};
use crate::search::evaluator::heuristic::context::{actions_per_round, max_owned_skill_cost};

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
#[derive(Clone, Copy)]
struct Piece {
    sq:    u8,
    is_p1: bool,
    kind:  Kind,
    /// Unused until a factor reads hp/armor/skills — drop this `allow` then.
    #[allow(dead_code)]
    mb:    MailboxEntry,
}

// ============================================================================
// THE SCORER — this is what you edit. One function, called once per piece.
// ============================================================================

fn score_piece(ctx: &CustomCtx, p: Piece) -> i32 {
    match p.kind {
        Kind::King     => score_king(ctx, p),
        Kind::Guard    => score_guard(ctx, p),
        Kind::Champion => score_champion(ctx, p),
    }
}

fn score_king(ctx: &CustomCtx, p: Piece) -> i32 {
    // The king has no material value (its capture is the MATE branch). Its only
    // score is the danger penalty — negative magnitude lowers the owner's total.
    -king_danger_malus(ctx, p)
}

fn score_guard(ctx: &CustomCtx, p: Piece) -> i32 {
    const BASE: f32 = 100.0;
    let rate = overextension_rate(ctx, p); // 0.0 (safe) ..= 1.0 (fully exposed)
    (BASE * (1.0 - rate)).round() as i32
}

fn score_champion(ctx: &CustomCtx, p: Piece) -> i32 {
    const BASE: f32 = 100.0;
    let rate = overextension_rate(ctx, p); // 0.0 (safe) ..= 1.0 (fully exposed)
    let base = (BASE * (1.0 - rate)).round() as i32;
    base + combo_overlap_bonus(ctx, p)
}

// ----- scoring helpers ------------------------------------------------------

/// Combo-setup bonus: reward a champion for sharing skill targets with a
/// *different* friendly combo-ticking champion — the geometric precondition for
/// a multi-champion combo. A square both can hit is worth more the closer it is
/// to an enemy (a real target), and only counts if this champion has a clear
/// path to it.
///
///   - Reach of a champion = `skill_attacks(sq, occ, max_range)` where `max_range`
///     is the larger of its two skills' true ranges (queen-rays, blocked by any
///     piece). Only combo-ticking skills (Strike or Move) matter for reach.
///   - Overlap = squares in BOTH this champion's reach and another friendly
///     combo-champion's reach, restricted to R4 of this champion (further out has
///     no combinatoric relevance).
///   - Per overlap square `f` with a clear path from `sq`: enemy-occupied → +50,
///     enemy at cheby 1 → +25, at cheby 2 → +10, else 0.
fn combo_overlap_bonus(ctx: &CustomCtx, p: Piece) -> i32 {
    // This champion's own reach; empty if it carries no combo-ticking skill.
    let my_reach = ctx.combo_reach(p.is_p1, p.sq);
    if my_reach == 0 {
        return 0;
    }

    // R4 disc around this champion (Chebyshev ≤ 4), from the precomputed table.
    // Excludes the centre, which is harmless here — the reach never contains sq.
    let r4 = within_range(p.sq, 4).0;

    let enemy = ctx.enemy_bb(p.is_p1);

    // Union of overlap squares this champion shares with any OTHER friendly
    // combo-champion, clipped to R4. Reaches are read from the per-side cache
    // (computed once in SideInfo::build), not recomputed here. OR-ing dedupes: a
    // square shared with two partners is still scored once (same target square).
    let my_clipped = my_reach & r4;
    let mut overlap = 0u64;
    for &(other_sq, other_reach) in &ctx.side(p.is_p1).reach {
        if other_sq == p.sq {
            continue; // must be a DIFFERENT champion
        }
        overlap |= my_clipped & other_reach;
    }

    // Score each overlap square by proximity to the nearest enemy, but only if
    // this champion has an unobstructed path to it.
    let mut bonus = 0i32;
    let mut sqs = overlap;
    while sqs != 0 {
        let f = sqs.trailing_zeros() as u8;
        sqs &= sqs - 1;

        // Path-clear: on a shared ray with nothing strictly between (checked first
        // to skip the proximity work for unreachable squares).
        if !(on_ray(p.sq, f) && (between(p.sq, f).0 & ctx.all_occ) == 0) {
            continue;
        }

        // Proximity tiers (occupied first, then exact rings 1 and 2).
        bonus += if enemy & (1u64 << f) != 0 {
            25
        } else if ctx.has_piece_in_ring(enemy, f, 1) {
            10
        } else if ctx.has_piece_in_ring(enemy, f, 2) {
            5
        } else {
            0
        };
    }
    bonus
}

/// How over-extended `p` is: `0.0` (well supported / no threat) up to `1.0`
/// (isolated in every ring while threatened). It measures **missing support**:
///
///   - For each Chebyshev ring n = 1, 2, 3 around the piece, if NO friendly
///     piece sits in that ring, add that ring's weight (0.50 / 0.33 / 0.17).
///     A single friend in a ring is enough to "cover" it — count doesn't matter.
///   - The whole thing is gated on enemy pressure: full weight if an enemy is
///     within R3, half weight if the nearest enemy is at R4, and `0.0` if no
///     enemy is within R4 (an isolated piece nobody can punish isn't a problem).
fn overextension_rate(ctx: &CustomCtx, p: Piece) -> f32 {
    // Ring weights for a ring with NO friendly support, indexed by ring-1.
    const RING_WEIGHT: [f32; 3] = [0.50, 0.33, 0.17];

    // Enemy-pressure gate via table masks (we only need the ≤3 / ==4 / >4 bucket,
    // not the exact distance): enemy within R3 → full, only at R4 → half, else 0.
    let enemy = ctx.enemy_bb(p.is_p1);
    let within3 = within_range(p.sq, 3).0;
    let within4 = within_range(p.sq, 4).0;
    let gate = if enemy & within3 != 0 {
        1.0
    } else if enemy & within4 != 0 {
        0.5
    } else {
        return 0.0; // no enemy within R4 → not over-extended
    };

    let own = ctx.own_bb(p.is_p1);
    let mut rate = 0.0;
    for (i, &weight) in RING_WEIGHT.iter().enumerate() {
        let ring = (i + 1) as u8;
        if !ctx.has_piece_in_ring(own, p.sq, ring) {
            rate += weight; // ring is unsupported → adds to over-extension
        }
    }

    rate * gate
}

/// King-danger penalty (positive magnitude; `score_king` negates it). Fires only
/// when the king is in REAL danger, then estimates a damage race: can the enemy
/// deal more than the king's effective health before we can shore it up?
///
/// **Gate** — return 0 unless an enemy can actually reach the king:
///   - any enemy within R2 (Move-Attack via free pathing), OR
///   - a Strike-carrying enemy within R4 that has a clear skill path to the king.
///
/// **Incoming** (raw max damage the enemy could deal next turn):
///   - +1 for the single Move-Attack per turn, iff an enemy sits within R2.
///   - affordable Strike casts over a 5-round window (actions ∩ money), only if a
///     Strike-carrying enemy is in skill range (R4 + path) — no cross-map strikes.
///
/// **Defense** (what we can add over the same window): affordable Shield/Heal/
/// Plate casts — Shield is self so always usable on the king; Heal/Plate need an
/// adjacent friendly caster, so they only count if we own one and a friendly
/// piece sits adjacent to the king.
///
/// **Balance:** `incoming − (king hp + armor + defense)`; penalty scales with the
/// positive remainder (0 if the defense holds).
fn king_danger_malus(ctx: &CustomCtx, p: Piece) -> i32 {
    /// Score per unit of unanswered incoming damage.
    const PER_DAMAGE: i32 = 400;
    /// Lookahead window (rounds) — matches the skill-capacity horizon.
    const LOOKAHEAD: u16 = 5;

    let king_sq = p.sq;
    let enemy_bb = ctx.enemy_bb(p.is_p1);

    // --- gate: is an enemy within Move-Attack (R2) or Strike (R4 + path) range? -
    let within2 = within_range(king_sq, 2).0;
    let within4 = within_range(king_sq, 4).0;
    let enemy_in_r2 = enemy_bb & within2 != 0;

    // A strike is only threatening if a Strike-carrying enemy can trace a clear
    // path to the king. `enemy_strike_reaches_king` checks R4 + path + strike skill.
    let atk = ctx.side(!p.is_p1); // the attacking side's precomputed info
    let strike_in_range =
        atk.min_strike_cost > 0 && ctx.enemy_strike_reaches_king(p.is_p1, king_sq, within4);

    if !enemy_in_r2 && !strike_in_range {
        return 0; // king not in real danger
    }

    // --- incoming max damage ------------------------------------------------
    let mut incoming = 0i32;
    if enemy_in_r2 {
        incoming += 1; // the one Move-Attack per turn
    }
    if strike_in_range {
        incoming += ctx.affordable_casts(!p.is_p1, atk.min_strike_cost, LOOKAHEAD);
    }

    // --- our defensive response over the same window ------------------------
    let me = ctx.side(p.is_p1);
    // Cheapest defensive skill we could spam. Shield (self) always applies to the
    // king; Heal/Plate only if we have an adjacent friendly caster+target.
    let mut defense = 0i32;
    if me.has_shield || (me.has_heal_or_plate && ctx.friendly_adjacent_to(p.is_p1, king_sq)) {
        // Both defensive families cost 2–3; use the side's max_skill_cost as the
        // conservative per-cast price (we don't track a separate min-defense cost,
        // and over-pricing defense keeps the penalty from being optimistic).
        let unit = me.max_skill_cost.max(1);
        defense = ctx.affordable_casts(p.is_p1, unit, LOOKAHEAD);
    }

    // --- balance against the king's effective health ------------------------
    let mb = ctx.pos.mailbox[king_sq as usize];
    let king_life = mb.hp() as i32 + mb.armor() as i32 + defense;
    let netto = incoming - king_life;
    if netto > 0 { netto * PER_DAMAGE } else { 0 }
}

// TODO piece activity term

// TODO piece's skill combo potential term (measuring if champs are set up well together so they can act together)

// TODO skill produktivitäts term (measuring if the skills currently hold value in the context of all other own pieces and the opponents pieces)
// (maybe a side term - but it depends on how i will design this)

// ============================================================================
// TERRITORY — Go-style board control, computed once for BOTH sides together.
// ============================================================================

/// Which side controls each square, from a simultaneous flood-fill (BFS) out of
/// both sides' pieces. The square a side reaches in fewer 8-adjacent steps is
/// theirs; pieces are walls — the flood goes around occupied squares, not through
/// them (so a square walled off behind your pieces is clearly yours, and the
/// enemy simply can't reach it).
///
/// **Contested squares** count HALF. A square is contested when either:
///   - both sides reach it in the SAME wave (equal distance — a genuine tie), or
///   - it is clearly one side's but sits ADJACENT to a square the other side
///     clearly controls (the a4/a5 front: both border squares are contested).
///
/// A tie square is handed to BOTH sides (each at half), so it nets out in the
/// diff; a border square stays its owner's but at half value.
///
/// This is symmetric in the two sides, so it is computed ONCE (in
/// [`CustomCtx::new`]) and both `score(is_p1)` values are read out of it — the
/// `territory` side-term stays write-once in shape but never double-computes the
/// expensive flood.
struct Territory {
    /// Fixed-point control value per side (scaled by [`Territory::SCALE`] so the
    /// ×0.5 contested weight stays integral). Divide by `SCALE` for "squares".
    p1: i32,
    p2: i32,
}

impl Territory {
    /// Fixed-point scale: values are held ×2 so a contested square's ½ weight is
    /// an integer. `score()` returns the scaled value; the diff is scaled too,
    /// which only changes the term's overall weight (tune it at the call site).
    const SCALE: i32 = 2;

    /// Run the full pipeline and return the per-side control totals.
    fn compute(pos: &Position) -> Self {
        let occ = pos.p1_pieces.0 | pos.p2_pieces.0;
        let empty = !occ;

        // ── Phase 1: simultaneous BFS flood, pieces block. ──────────────────
        // Each side's front grows one 8-adjacent ring per wave, into EMPTY
        // squares only (walls block). A square goes to whoever reaches it first;
        // a square reached by BOTH in the same wave is an equal-distance tie —
        // it goes to both (`ties`), and counts half for each.
        let mut p1_reached = pos.p1_pieces.0; // sources: own pieces (distance 0)
        let mut p2_reached = pos.p2_pieces.0;
        let mut p1_ctrl = 0u64; // empty squares controlled by P1 (incl. ties)
        let mut p2_ctrl = 0u64;
        let mut ties = 0u64;    // equal-distance squares (belong to both, halved)
        let mut claimed = occ;  // squares already decided (start: all pieces)

        // The flood saturates in ≤8 King-steps across an 8×8 grid, but loop until
        // nothing new is claimed to be safe against odd wall shapes.
        loop {
            // One wave out of each side's current front, into still-empty squares.
            let p1_front = king_expand(p1_reached) & empty & !claimed;
            let p2_front = king_expand(p2_reached) & empty & !claimed;
            if p1_front == 0 && p2_front == 0 {
                break; // nothing left reachable
            }

            // Ties (both reach this wave) go to BOTH sides; the rest split cleanly.
            let both = p1_front & p2_front;
            p1_ctrl |= p1_front; // includes ties — a tie square is P1's AND P2's
            p2_ctrl |= p2_front;
            ties    |= both;

            // Advance both fronts and mark this wave's squares decided.
            p1_reached |= p1_front;
            p2_reached |= p2_front;
            claimed    |= p1_front | p2_front;
        }

        // ── Phase 2: contested = ties ∪ border squares. ─────────────────────
        // Border: a square I clearly control that is adjacent to one the enemy
        // clearly controls (the a4/a5 front). Ties are contested by definition.
        // "Clearly" excludes ties from the border test so a tie doesn't drag its
        // neighbours in twice — ties are already contested.
        let p1_only = p1_ctrl & !ties;
        let p2_only = p2_ctrl & !ties;
        let p1_contested = ties | (p1_only & king_expand(p2_only));
        let p2_contested = ties | (p2_only & king_expand(p1_only));

        // ── Phase 3+4: enemy-king bonus on top, applied AFTER halving. ──────
        // Squares near the ENEMY king are worth more (R1 → 3, R2 → 2, else 1),
        // never around your own king (so a side isn't lured to march its king
        // forward to farm points). `>= 64` means that king is off the board.
        let p1_king_sq = (pos.kings.0 & pos.p1_pieces.0).trailing_zeros();
        let p2_king_sq = (pos.kings.0 & pos.p2_pieces.0).trailing_zeros();

        // For P1's score the relevant enemy king is P2's, and vice-versa.
        let p1 = Self::sum_side(p1_ctrl, p1_contested, p2_king_sq);
        let p2 = Self::sum_side(p2_ctrl, p2_contested, p1_king_sq);

        Territory { p1, p2 }
    }

    /// Sum one side's control value (fixed-point ×SCALE): base 1 per square,
    /// halved on contested squares FIRST, then raised toward the enemy king
    /// (R1 → ×3, R2 → ×2). `enemy_king_sq >= 64` skips the bonus (king gone).
    fn sum_side(ctrl: u64, contested: u64, enemy_king_sq: u32) -> i32 {
        let has_enemy_king = enemy_king_sq < 64;
        let mut total = 0i32;
        let mut bits = ctrl;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            let mask = 1u64 << sq;

            // Base 1, held in fixed point (×SCALE). Contested → half.
            let mut value = if contested & mask != 0 { Self::SCALE / 2 } else { Self::SCALE };

            // King bonus on top, AFTER the halving (a contested R1 square is
            // 3 × ½ = 1.5 → SCALE*3/2). Multiply the already-halved value.
            if has_enemy_king {
                let mult = match cheby_dist(sq, enemy_king_sq as u8) {
                    1 => 3,
                    2 => 2,
                    _ => 1,
                };
                value *= mult;
            }
            total += value;
        }
        total
    }

    /// This side's control total (fixed-point). The side-term reads this.
    #[inline]
    fn score(&self, is_p1: bool) -> i32 {
        if is_p1 { self.p1 } else { self.p2 }
    }
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
    SideTerm { name: "skill_capacity",   sign: 1, f: term_skill_capacity },
    SideTerm { name: "offense_capable",  sign: 1, f: term_offense_capable },
    SideTerm { name: "territory",        sign: 1, f: term_territory },
];

/// Offensive capability: a raw, symmetric count of how much *realisable* offense
/// this side's champions carry. It captures "don't fritter away all your
/// attackers — you still need the means to actually win".
///
///   - A champion with a **Strike** skill (Lance/Hook/Break/Steal/Tempest) is
///     offense on its own — it deals damage and ticks the combo counter.
///   - A champion with a **Move** skill (Dash/Blast/Shove/Swap/Retreat) deals no
///     damage by itself; it only contributes once the combo counter is already
///     built, i.e. only if the side has at least one Strike attacker to start it.
///
/// Each champion ticks the counter at most once (Strike takes precedence), so we
/// count Strike carriers and Move carriers separately, then only credit the Move
/// carriers when a Strike carrier exists to prime the combo.
fn term_offense_capable(ctx: &CustomCtx, is_p1: bool) -> i32 {
    /// Score per capable champion.
    const PER_CHAMP: i32 = 10;

    // Counts are precomputed once per side in SideInfo::build.
    let s = ctx.side(is_p1);
    // Move carriers only count if a Strike carrier exists to prime the combo.
    let realisable = s.strike_champs + if s.strike_champs > 0 { s.move_champs } else { 0 };
    realisable * PER_CHAMP
}


/// Skill capacity: how much max-cost skill use this side can actually PAY FOR
/// over the next few rounds — saturated at what it could ever spend.
///
/// The idea: money only matters as *potential skill throughput*. A side that can
/// fire its most expensive skill in every action, this round and the next few,
/// is ahead — but only up to the point where extra money is dead ("runaway": you
/// cannot spend faster than your actions × cost cap allows). So we compute:
///
///   - `spend_cap`  = Σ over the lookahead window of `actions(r) × max_cost` —
///     the most this side could possibly spend in the window.
///   - `available`  = current treasury + Σ income over the window.
///   - `score`      = min(available, spend_cap) — capacity, capped at the runaway
///     point. Hoarding past `spend_cap` adds nothing (flat, not falling — no
///     incentive to waste money, just none to hoard).
///
/// `max_cost` is the single most expensive owned skill: we score the POTENTIAL to
/// fire it every action, not the expected use. Productivity (does the skill do
/// anything?) is a separate term, deliberately not folded in here.
///
/// Money uses [`CustomCtx::effective_money`], which credits the not-on-move side
/// its pending start-of-turn income — so a side isn't over-rated just for having
/// banked this round's income a half-turn before its opponent does.
fn term_skill_capacity(ctx: &CustomCtx, is_p1: bool) -> i32 {
    use crate::game_logic::turn_manager::income_per_turn;

    /// How many rounds ahead to look. Beyond this, plans are too speculative.
    const LOOKAHEAD: u16 = 3;
    /// Score weight per unit of payable capacity (tune the term's magnitude here).
    const PER_UNIT: i32 = 10;

    // Effective treasury already folds in this side's current-round income (real
    // for the mover, pending-credited for the other), so the lookahead below adds
    // only genuinely FUTURE rounds' income to avoid double-counting.
    let money = ctx.effective_money(is_p1);
    if money <= 0 {
        return 0; // broke → no capacity, and avoids the degenerate case
    }

    // `max_cost` is fixed over the window (equipped skills don't change) and is
    // precomputed once per side in SideInfo::build. Zero → no capacity.
    let max_cost = ctx.side(is_p1).max_skill_cost;
    if max_cost == 0 {
        return 0;
    }

    // spend_cap spans the whole window (you can spend THIS round too); future
    // income is counted only from the next round on (this round's is in `money`).
    let mut spend_cap = 0i32;
    let mut income = 0i32;
    let start = ctx.pos.round_number;
    for r in start..start.saturating_add(LOOKAHEAD) {
        let actions = actions_per_round(ctx.pos.current_phase, r) as i32;
        spend_cap += actions * max_cost;
        if r > start {
            income += income_per_turn(r) as i32;
        }
    }

    let available = money + income;
    // Capacity, saturated at the runaway point: money you can't spend is dead.
    available.min(spend_cap) * PER_UNIT
}

/// Territory: this side's Go-style board control, read from the shared
/// [`Territory`] computed once per eval. Written once; the driver diffs P1/P2.
fn term_territory(ctx: &CustomCtx, is_p1: bool) -> i32 {
    ctx.territory.score(is_p1)
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
    /// Go-style board control, computed once for both sides (see [`Territory`]).
    territory: Territory,
    /// Per-side precomputed state, indexed `[0] = P1, [1] = P2`. Built once in
    /// `new` so the per-piece walk and the side-terms never re-scan the board.
    sides: [SideInfo; 2],
}

/// Per-side state precomputed once per eval (mirrors the heuristic's `EvalContext`
/// pattern of paying board scans once). Holds the champion combo-reach cache and
/// the skill-inventory counts the side-terms need.
#[derive(Default)]
struct SideInfo {
    /// Per-champion combo-reach bitboards, one entry per champion that carries a
    /// combo-ticking skill (ascending square order). Computed once so
    /// `combo_overlap_bonus` never recomputes `skill_attacks` O(champs²) times.
    reach: Vec<(u8, u64)>,
    /// Count of champions carrying a Strike skill.
    strike_champs: i32,
    /// Count of champions carrying a Move (but no Strike) skill.
    move_champs: i32,
    /// Most expensive owned skill cost on this side (0 if none).
    max_skill_cost: i32,
    /// Cheapest Strike-skill cost owned on this side (0 if the side has none) —
    /// the per-strike price used when estimating incoming king damage.
    min_strike_cost: i32,
    /// This side owns a Shield skill (self-cast +Armor on the caster).
    has_shield: bool,
    /// This side owns a Heal or Plate skill (adjacent-ally defensive cast).
    has_heal_or_plate: bool,
}

impl SideInfo {
    /// One walk over this side's champions, collecting everything the per-piece
    /// scorer and side-terms need: the combo-reach cache and the skill counts.
    /// Skill inventory (min strike cost, defensive flags) also covers the King's
    /// two slots, since the King carries skills too.
    fn build(pos: &Position, side_bb: u64, all_occ: u64) -> Self {
        let champs = side_bb & pos.champions.0;
        let mut reach = Vec::with_capacity(champs.count_ones() as usize);
        let mut strike_champs = 0i32;
        let mut move_champs = 0i32;
        let mut min_strike_cost = 0i32;
        let mut has_shield = false;
        let mut has_heal_or_plate = false;

        // Champions: reach cache + per-champion strike/move tick classification.
        let mut bits = champs;
        while bits != 0 {
            let sq = bits.trailing_zeros() as u8;
            bits &= bits - 1;

            // Combo reach (only nonzero if it carries a combo-ticking skill).
            let r = CustomCtx::combo_reach_of(pos, all_occ, sq);
            if r != 0 {
                reach.push((sq, r));
            }

            // Skill inventory: a champion ticks at most once, Strike over Move.
            let mb = pos.mailbox[sq as usize];
            let mut has_strike = false;
            let mut has_move = false;
            for id in [mb.skill1(), mb.skill2()] {
                if let Some(s) = skill_from_id(id) {
                    match skill_category(s) {
                        SkillCategory::Strike => has_strike = true,
                        SkillCategory::Move   => has_move = true,
                        _ => {}
                    }
                }
            }
            if has_strike { strike_champs += 1; }
            else if has_move { move_champs += 1; }
        }

        // Side-wide skill inventory over Champions AND the King (both carry
        // skills): cheapest strike cost + which defensive skills exist.
        let mut skilled = side_bb & (pos.champions.0 | pos.kings.0);
        while skilled != 0 {
            let sq = skilled.trailing_zeros() as usize;
            skilled &= skilled - 1;
            let mb = pos.mailbox[sq];
            for id in [mb.skill1(), mb.skill2()] {
                if let Some(s) = skill_from_id(id) {
                    match s {
                        Skill::Shield => has_shield = true,
                        Skill::Heal | Skill::Plate => has_heal_or_plate = true,
                        _ => {}
                    }
                    if skill_category(s) == SkillCategory::Strike {
                        let c = skill_cost(s) as i32;
                        if min_strike_cost == 0 || c < min_strike_cost {
                            min_strike_cost = c;
                        }
                    }
                }
            }
        }

        SideInfo {
            reach,
            strike_champs,
            move_champs,
            max_skill_cost: max_owned_skill_cost(pos, side_bb) as i32,
            min_strike_cost,
            has_shield,
            has_heal_or_plate,
        }
    }
}

impl<'a> CustomCtx<'a> {
    fn new(pos: &'a Position) -> Self {
        let all_occ = pos.p1_pieces.0 | pos.p2_pieces.0;
        CustomCtx {
            pos,
            all_occ,
            territory: Territory::compute(pos),
            sides: [
                SideInfo::build(pos, pos.p1_pieces.0, all_occ),
                SideInfo::build(pos, pos.p2_pieces.0, all_occ),
            ],
        }
    }

    /// Precomputed state for the side `is_p1` selects.
    #[inline]
    fn side(&self, is_p1: bool) -> &SideInfo {
        &self.sides[if is_p1 { 0 } else { 1 }]
    }

    /// Cached combo-reach for the champion at `sq` on the side `is_p1`, or `0` if
    /// there's no champion there or it carries no combo-ticking skill. Looks up
    /// the reach computed once in [`SideInfo::build`] instead of recomputing it.
    #[inline]
    fn combo_reach(&self, is_p1: bool, sq: u8) -> u64 {
        self.side(is_p1).reach.iter()
            .find(|&&(s, _)| s == sq)
            .map_or(0, |&(_, r)| r)
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

    /// This side's **effective** treasury: real money, plus the guaranteed
    /// start-of-turn income it hasn't collected yet if it's NOT on the move.
    ///
    /// Income is disbursed at each player's turn-start, so between the two turns
    /// of a round the side that already moved looks richer purely by timing. This
    /// credits the not-on-move side its pending income so a static eval doesn't
    /// over-rate whoever happens to have just banked. Which round's income is
    /// pending depends on who moves next (`round_number` bumps on P2→P1):
    ///   - P1 on move → P2 collects THIS round → credit `income(round_number)`.
    ///   - P2 on move → P1 collects NEXT round → credit `income(round_number+1)`.
    ///
    /// R1 pays no income (rule), so `income_per_turn`'s callers gate at ≥ 2; we
    /// mirror that here by only crediting when the pending round is ≥ 2.
    fn effective_money(&self, is_p1: bool) -> i32 {
        use crate::game_logic::turn_manager::income_per_turn;
        let money = self.money(is_p1);
        let on_move = (self.pos.to_move == crate::state::position::Player::P1) == is_p1;
        if on_move {
            return money; // already banked this round's income
        }
        // Not on move: which round's income is still pending for this side?
        let pending_round = if is_p1 {
            self.pos.round_number + 1 // P2 on move; P1 collects next round
        } else {
            self.pos.round_number // P1 on move; P2 collects this round
        };
        if pending_round < 2 {
            return money; // R1 pays nothing
        }
        money + income_per_turn(pending_round) as i32
    }

    /// How many skill activations of unit cost `unit_cost` this side could afford
    /// over a `lookahead`-round window, limited BOTH by skill-phase actions and by
    /// money (treasury + future income). This is the shared money/action model the
    /// king-danger term uses for incoming strikes AND our defensive casts, so the
    /// two are compared on the same footing.
    ///
    /// `unit_cost == 0` (side owns no such skill) → 0. Money uses
    /// [`effective_money`] so the income-timing artefact never tilts the estimate.
    fn affordable_casts(&self, is_p1: bool, unit_cost: i32, lookahead: u16) -> i32 {
        use crate::game_logic::turn_manager::income_per_turn;
        if unit_cost <= 0 {
            return 0;
        }
        let mut money = self.effective_money(is_p1);
        let start = self.pos.round_number;
        let mut casts = 0i32;
        for r in start..start.saturating_add(lookahead) {
            // This round's income is already in `effective_money`; future rounds add.
            if r > start {
                money += income_per_turn(r) as i32;
            }
            let actions = actions_per_round(self.pos.current_phase, r) as i32;
            let affordable_by_money = money / unit_cost;
            let this_round = actions.min(affordable_by_money);
            if this_round <= 0 {
                continue;
            }
            casts += this_round;
            money -= this_round * unit_cost; // spend it so later rounds can't reuse it
        }
        casts
    }

    /// Does the side OPPOSING `is_p1` have a Strike-carrying piece within R4 of
    /// `king_sq` that can trace a clear skill path to it? `within4` is the
    /// precomputed R4 mask of `king_sq`. Skills are queen-rays blocked by any
    /// piece, so "reaches" = on a shared ray with nothing strictly between.
    fn enemy_strike_reaches_king(&self, is_p1: bool, king_sq: u8, within4: u64) -> bool {
        // Enemy pieces that carry skills (champions + king) and sit within R4.
        let enemy = self.enemy_bb(is_p1);
        let mut cand = enemy & (self.pos.champions.0 | self.pos.kings.0) & within4;
        while cand != 0 {
            let sq = cand.trailing_zeros() as u8;
            cand &= cand - 1;
            // Must actually carry a Strike skill whose true range reaches king_sq…
            let mb = self.pos.mailbox[sq as usize];
            let mut reach = 0u8;
            for id in [mb.skill1(), mb.skill2()] {
                if let Some(s) = skill_from_id(id) {
                    if skill_category(s) == SkillCategory::Strike {
                        reach = reach.max(skill_default_range(s));
                    }
                }
            }
            if reach == 0 {
                continue;
            }
            // …and have a clear queen-ray path within that range to the king.
            if cheby_dist(sq, king_sq) <= reach
                && on_ray(sq, king_sq)
                && (between(sq, king_sq).0 & self.all_occ) == 0
            {
                return true;
            }
        }
        false
    }

    /// Is a friendly piece (of the side `is_p1`) on a square adjacent (R1) to
    /// `sq`? Used to check a Heal/Plate caster could reach the king.
    #[inline]
    fn friendly_adjacent_to(&self, is_p1: bool, sq: u8) -> bool {
        let ring1 = within_range(sq, 1).0;
        self.own_bb(is_p1) & ring1 != 0
    }

    // --- factor helpers for `score_piece` ------------------------------------

    /// Occupancy bitboard of the side `is_p1` selects (the piece's own side).
    #[inline]
    fn own_bb(&self, is_p1: bool) -> u64 {
        if is_p1 { self.pos.p1_pieces.0 } else { self.pos.p2_pieces.0 }
    }

    /// Occupancy bitboard of the side opposing `is_p1`.
    #[inline]
    fn enemy_bb(&self, is_p1: bool) -> u64 {
        if is_p1 { self.pos.p2_pieces.0 } else { self.pos.p1_pieces.0 }
    }

    /// Skill-target reach of the piece at `sq` for combo purposes: the squares it
    /// could hit with its longest-range **combo-ticking** skill (Strike or Move).
    /// Returns `0` if it carries no such skill. Queen-rays via `skill_attacks`,
    /// blocked by any piece (`all_occ`). When two combo skills are equipped we use
    /// the MAX of their true ranges (the wider reach dominates the overlap set).
    ///
    /// Free function (not a `ctx` method) because it's called during
    /// [`SideInfo::build`], before the `CustomCtx` exists.
    fn combo_reach_of(pos: &Position, all_occ: u64, sq: u8) -> u64 {
        let mb = pos.mailbox[sq as usize];
        let mut max_range = 0u8;
        for id in [mb.skill1(), mb.skill2()] {
            if let Some(s) = skill_from_id(id) {
                // Only Strike / Move skills tick the combo counter (per RULES.md).
                if matches!(skill_category(s), SkillCategory::Strike | SkillCategory::Move) {
                    max_range = max_range.max(skill_default_range(s));
                }
            }
        }
        if max_range == 0 {
            return 0;
        }
        skill_attacks(sq, all_occ, max_range).0
    }

    /// Is there at least one piece of `side_bb` at exactly Chebyshev distance
    /// `ring` from `sq`? Uses the table-backed [`within_range`] annulus (the disc
    /// at `ring` minus the disc at `ring-1`) — one mask AND, no per-piece loop.
    #[inline]
    fn has_piece_in_ring(&self, side_bb: u64, sq: u8, ring: u8) -> bool {
        let inner = if ring == 0 { 0 } else { within_range(sq, ring - 1).0 };
        let shell = within_range(sq, ring).0 & !inner;
        side_bb & shell != 0
    }
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
    /// Fixed-size (no heap alloc on the hot path): one `Sums` per `SIDE_TERMS`.
    side_sums:   [Sums; SIDE_TERMS.len()],
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
        // Fixed-size array — no heap allocation on the hot `evaluate` path.
        let mut side_sums = [Sums::default(); SIDE_TERMS.len()];
        let mut total = piece_total;
        for (i, t) in SIDE_TERMS.iter().enumerate() {
            let s = Sums { p1: (t.f)(&ctx, true), p2: (t.f)(&ctx, false) };
            total += t.sign * (s.p1 - s.p2);
            side_sums[i] = s;
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

    /// Opt OUT of quiescence search. QS exists to cover a horizon-blind eval on
    /// mid-exchange positions; this per-piece eval weighs a piece by its exposure
    /// (hp/armor vs threat) so it reads those positions directly. On this game's
    /// King-danger endgames QS is an undisciplined full-width search that starves
    /// the main tree of depth — measured +1.5–2 plies deeper with QS off, same or
    /// better moves. See `game/plans/custom-eval-search-cliff.md`.
    #[inline]
    fn wants_qs(&self) -> bool { false }

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

    /// A vertically-mirrored board (P1 rank r ↔ P2 rank 7-r, kings on the same
    /// file) must give both sides identical territory — the diff is exactly 0.
    /// This pins the flood, the contested split, and the enemy-king bonus all
    /// being symmetric.
    #[test]
    fn territory_is_symmetric_on_mirrored_board() {
        let mut pos = Position::empty();
        // P1: king d1 (sq 3), a champion at c1 (sq 2). P2: mirror across ranks
        // (sq ^ 56): king d8 (sq 59), champion c8 (sq 58). Same files → symmetric.
        for (sq, is_p1) in [(3u8, true), (2u8, true), (59u8, false), (58u8, false)] {
            let bit = 1u64 << sq;
            if is_p1 { pos.p1_pieces.0 |= bit; } else { pos.p2_pieces.0 |= bit; }
        }
        pos.kings.0     = (1 << 3) | (1 << 59);
        pos.champions.0 = (1 << 2) | (1 << 58);

        let t = Territory::compute(&pos);
        assert_eq!(t.p1, t.p2, "mirrored board must be territorially even");
    }

    /// Contested squares (ties + borders) must actually lower a side's raw count:
    /// a board where the two sides' territories touch should score less than the
    /// same square set with no opposing pressure. Sanity check that halving fires.
    #[test]
    fn territory_contested_halves_the_frontier() {
        // Two lone kings facing off in the centre: every empty square is a tie
        // or a border, so both sides' control is heavily halved but still equal.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = 1 << 27; // d4
        pos.p2_pieces.0 = 1 << 35; // d5
        pos.kings.0 = (1 << 27) | (1 << 35);

        let t = Territory::compute(&pos);
        assert_eq!(t.p1, t.p2, "symmetric face-off is even");
        assert!(t.p1 > 0, "each side still controls something");
    }

    /// The income-timing fix: on a mirrored board with equal money and skills,
    /// the side NOT on the move is credited its pending income, so the
    /// `skill_capacity` term reads equal for both — no artefact from the mover
    /// having banked this round's income a half-turn early.
    #[test]
    fn skill_capacity_equalises_income_timing() {
        let mut pos = Position::empty();
        pos.round_number = 3; // ≥ 2 so income is live
        pos.to_move = crate::state::position::Player::P1;
        pos.current_phase = crate::state::position::Phase::Skill;

        // One champion each, mirrored (sq ^ 56), both carrying Steal (id 4, cost 4).
        let mb = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(4);
        pos.p1_pieces.0 = 1 << 10; // c2
        pos.p2_pieces.0 = 1 << 50; // c7 (10 ^ 56)
        pos.champions.0 = (1 << 10) | (1 << 50);
        pos.mailbox[10] = mb;
        pos.mailbox[50] = mb;

        // Realistic timing: P1 is on the move and has ALREADY banked round 3's
        // income; P2 has not yet (it collects at the start of its own turn). So a
        // genuinely even position has P2's raw money lower by exactly income(3).
        use crate::game_logic::turn_manager::income_per_turn;
        let r3 = income_per_turn(3) as u16;
        pos.p1_money = 8;
        pos.p2_money = 8 - r3;

        let ctx = CustomCtx::new(&pos);
        // effective_money credits P2 its pending income, lifting it back to P1's
        // level, so the term must read equal for both sides.
        assert_eq!(
            term_skill_capacity(&ctx, true),
            term_skill_capacity(&ctx, false),
            "income timing must not tilt an evenly-timed position"
        );

        // The credit is exactly this round's income; the mover keeps its banked total.
        assert_eq!(
            ctx.effective_money(false),
            ctx.money(false) + r3 as i32,
        );
        assert_eq!(ctx.effective_money(true), ctx.money(true), "mover already banked");
    }

    /// Offense capability: a Move-only champion contributes NOTHING on its own
    /// (it can't prime a combo), but starts counting once a Strike carrier exists
    /// to build the counter. Strike carriers always count.
    #[test]
    fn offense_capable_gates_move_on_strike() {
        // Two champions for P1: one on c2, one on e2. P2 empty (isolate P1's count).
        let strike = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // Lance = Strike
        let mv     = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(9); // Dash  = Move

        // Case A: both champions carry only a Move skill → no Strike to prime →
        // realisable offense is 0.
        let mut pos = Position::empty();
        pos.p1_pieces.0 = (1 << 10) | (1 << 12);
        pos.champions.0 = (1 << 10) | (1 << 12);
        pos.mailbox[10] = mv;
        pos.mailbox[12] = mv;
        let ctx = CustomCtx::new(&pos);
        assert_eq!(term_offense_capable(&ctx, true), 0, "move-only can't realise offense");

        // Case B: swap one to a Strike carrier → now BOTH count (strike primes the
        // move carrier) → 2 × PER_CHAMP.
        pos.mailbox[10] = strike;
        let ctx = CustomCtx::new(&pos);
        assert_eq!(term_offense_capable(&ctx, true), 20, "strike primes the move carrier");
    }

    /// Combo overlap: two champions that both threaten enemy-near squares earn
    /// the proximity bonus, summed over every shared square; a lone champion with
    /// no partner earns nothing.
    #[test]
    fn combo_overlap_rewards_shared_target_near_enemy() {
        // P1 champs on c3(18) and e3(20), both carrying Lance (Strike, range 1).
        // Their range-1 reaches overlap on the column between them: d2(11), d3(19),
        // d4(27). With an enemy on d5(35): d4 is cheby-1 (+25), d3 cheby-2 (+10),
        // d2 cheby-3 (+0). All paths are adjacent (nothing between). Total = 35.
        let lance = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1);
        let mut pos = Position::empty();
        pos.p1_pieces.0 = (1 << 18) | (1 << 20);
        pos.champions.0 = (1 << 18) | (1 << 20) | (1 << 35);
        pos.p2_pieces.0 = 1 << 35;
        pos.mailbox[18] = lance;
        pos.mailbox[20] = lance;

        let ctx = CustomCtx::new(&pos);
        let c3 = Piece { sq: 18, is_p1: true, kind: Kind::Champion, mb: lance };
        assert_eq!(combo_overlap_bonus(&ctx, c3), 35, "d4(+25) + d3(+10), d2 too far");

        // Remove the partner on e3 → no other champion shares any square → no bonus.
        let mut solo = pos.clone();
        solo.p1_pieces.0 = 1 << 18;
        solo.champions.0 = (1 << 18) | (1 << 35);
        solo.mailbox[20] = crate::state::EMPTY_MAILBOX_ENTRY;
        let ctx = CustomCtx::new(&solo);
        assert_eq!(combo_overlap_bonus(&ctx, c3), 0, "no partner → no combo setup");
    }

    /// King danger: no malus when no enemy can reach the king; a real threat that
    /// outpaces the defense produces a positive penalty (negative king score).
    #[test]
    fn king_danger_gates_and_penalises() {
        let king = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2); // hp 2, no armor
        let lance = crate::state::EMPTY_MAILBOX_ENTRY.with_hp(2).with_skill1(1); // Strike, cost 2

        // Case A: P1 king on d1(3), lone enemy king far away on h8(63) → nobody in
        // R2 and no strike in range → no danger.
        let mut pos = Position::empty();
        pos.round_number = 6; // income live, actions available
        pos.to_move = crate::state::position::Player::P2; // enemy to move
        pos.current_phase = crate::state::position::Phase::Skill;
        pos.p1_pieces.0 = 1 << 3;
        pos.p2_pieces.0 = 1 << 63;
        pos.kings.0 = (1 << 3) | (1 << 63);
        pos.mailbox[3] = king;
        pos.mailbox[63] = king;
        let ctx = CustomCtx::new(&pos);
        let kp = Piece { sq: 3, is_p1: true, kind: Kind::King, mb: king };
        assert_eq!(king_danger_malus(&ctx, kp), 0, "no enemy in reach → no danger");

        // Case B: put P2 strike champions adjacent to the king (c2, e2) so their
        // range-1 Lance actually reaches it, with plenty of money and no P1
        // defensive skill → affordable strikes outrun the king's 2 health.
        pos.p2_money = 20;
        pos.p2_pieces.0 = (1 << 63) | (1 << 10) | (1 << 12); // c2, e2 — both king-adjacent
        pos.champions.0 = (1 << 10) | (1 << 12);
        pos.mailbox[10] = lance;
        pos.mailbox[12] = lance;
        let ctx = CustomCtx::new(&pos);
        let malus = king_danger_malus(&ctx, kp);
        assert!(malus > 0, "king outnumbered with no defense → penalty, got {malus}");

        // And score_king mirrors it as a negative owner magnitude.
        assert_eq!(score_king(&ctx, kp), -malus);
    }
}
