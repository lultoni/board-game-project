//! Quiescence Search — catalogue §3, minus SEE.
//!
//! Hooked at `depth <= 0` in `alpha_beta::search`. Resolves the horizon
//! effect: at the depth-0 boundary the static eval is blind to a hanging
//! Champion, a Lance/Hook combo, or a Move-Attack-into-King that resolves
//! one ply past the leaf. QS continues the search through "loud" actions
//! (HP-changing moves + King-threats) until the position is quiet, then
//! returns the static eval.
//!
//! # Frame
//!
//! **Absolute P1-POV**, matching the rest of the engine — not negamax.
//! P1 maximises, P2 minimises. Stand-pat logic is asymmetric.
//!
//! # First-cut scope (v1, this module)
//!
//! - Stand-pat with check-evasion gate (no stand-pat when King is threatened).
//! - Loud-action loop (Move-Attacks + Strike/Blast skills). Quiet moves
//!   (Shield/Heal/Plate/Focus/Charge, Dash/Retreat/Shove/Swap, plain Moves,
//!   EndPhase, EndTurn) are skipped.
//! - When in check, search ALL legal moves (check-evasion).
//! - Hard ply cap at MAX_QS_PLY to prevent infinite recursion in
//!   pathological positions.
//! - **No TT, no killers/history bumps** — keeps the diff small and
//!   avoids polluting the main-search ordering tables.
//!
//! Deferred to v2 (each tracked in the catalogue):
//! - Delta pruning (skip clearly-losing tactical lines).
//! - King-threat-changing-Move detection (a plain Move that puts an
//!   attacker into next-ply Strike range of opponent King).
//! - SEE-style skill-exchange evaluation.
//! - QS-level TT entries.

use crate::time::now_ms;

use super::alpha_beta::{SearchCtx, INF, TIME_CHECK_MASK};
use super::evaluator::{evaluate, MATE_SCORE};
use crate::game_logic::action::{Action, ActionKind};
use crate::game_logic::skills::{
    skill_category, skill_cost, skill_default_range, skill_from_id, Skill, SkillCategory,
};
use crate::game_logic::{generator, make_unmake};
use crate::state::Position;
use crate::state::position::{GameResult, Phase, Player};

/// Cap on quiescence recursion depth. Prevents stack overflow in pathological
/// positions (e.g. two pieces shuffling Strike skills with regenerating money).
/// 8 plies is far past any realistic tactical line in Stack M.
const MAX_QS_PLY: i32 = 8;

/// Mirror `alpha_beta::MAX_PLY` — the absolute ply cap that bounds killer/
/// history arrays. Quiescence must not exceed this in the cumulative `ply`
/// counter or it would index out-of-range tables if/when we add QS hooks
/// into history later.
const MAX_PLY: i32 = 128;

/// True iff `a` is a "loud" action that should be searched inside quiescence.
///
/// Loud = "changes HP" or "redirects an in-flight damaging move":
/// - Move-Attack (`has_approach() == true`).
/// - Strike-category skills (Lance/Hook/Break/Steal/Tempest).
/// - Blast (Move-category but deals combo-tick damage in `apply_blast`).
/// - BodyguardChoice (mid-resolution of an already-loud Move-Attack).
///
/// Not loud:
/// - Plain Moves (no approach).
/// - Shield/Heal/Plate/Focus/Charge skills (no HP swing to opponent).
/// - Dash/Retreat/Shove/Swap (positional Move-category, no damage).
/// - EndPhase / EndTurn.
/// - DraftTurn (no QS in draft phase — there's no HP).
///
/// `pos` is unused in v1 — reserved for the King-threat-changing-Move
/// extension (catalogue §3, "loud actions to search in QS").
#[inline]
pub(super) fn is_loud(a: Action, _pos: &Position) -> bool {
    if a.is_draft_turn() { return false; }
    if a.is_bodyguard_choice() { return true; }
    match a.kind() {
        ActionKind::Move => a.has_approach(),
        ActionKind::Skill => match skill_from_id(a.skill_id()) {
            Some(s) => skill_category(s) == SkillCategory::Strike || s == Skill::Blast,
            None    => false,
        },
        ActionKind::EndPhase | ActionKind::EndTurn => false,
    }
}

/// True iff `side`'s King could be damaged by any of the *opponent's* loud
/// actions at the next ply (Strike/Blast skills landing on the King square,
/// or a Move-Attack capturing the King).
///
/// Fast bitboard scan — does NOT call `generator::generate`. Iterates opponent
/// piece squares and tests reachability against the King via Chebyshev
/// distance (Move-Attacks use piece speed; Strike/Blast skills use the
/// skill's default range). Over-approximates by ignoring secondary
/// constraints (Hook landing space, Focus +1 buff, blocking pieces) — being
/// "too cautious about check" only forces QS to search all moves instead of
/// pruning, which is still correct.
///
/// Returns false when:
/// - `side`'s King is already gone (`game_result` handles the terminal).
/// - `pending_bodyguard` is set (mid-stack, parent frame will resolve).
/// - `actions_remaining == 0` (opponent can only EndPhase, no loud actions).
/// - Phase is Draft (no HP, no threats).
pub(super) fn is_king_threatened(pos: &mut Position, side: Player) -> bool {
    if pos.pending_bodyguard.is_some() { return false; }
    if pos.actions_remaining == 0 { return false; }

    let (side_bb, opp_bb, opp_money) = match side {
        Player::P1 => (pos.p1_pieces, pos.p2_pieces, pos.p2_money),
        Player::P2 => (pos.p2_pieces, pos.p1_pieces, pos.p1_money),
    };
    let king_sq = match (pos.kings & side_bb).lsb() {
        Some(sq) => sq,
        None     => return false,
    };

    match pos.current_phase {
        Phase::Draft => false,
        Phase::Move => {
            let mut bits = opp_bb.0;
            while bits != 0 {
                let sq = bits.trailing_zeros() as u8;
                bits &= bits - 1;
                let speed: u8 = if pos.guards.contains(sq) { 2 } else { 1 };
                if chebyshev(sq, king_sq) <= speed { return true; }
            }
            false
        }
        Phase::Skill => {
            let mut bits = opp_bb.0;
            while bits != 0 {
                let sq = bits.trailing_zeros() as u8;
                bits &= bits - 1;
                let mb = pos.mailbox[sq as usize];
                for sid in [mb.skill1(), mb.skill2()] {
                    let s = match skill_from_id(sid) { Some(s) => s, None => continue };
                    let damaging = skill_category(s) == SkillCategory::Strike || s == Skill::Blast;
                    if !damaging { continue; }
                    if (opp_money as u8) < skill_cost(s) { continue; }
                    if chebyshev(sq, king_sq) <= skill_default_range(s) { return true; }
                }
            }
            false
        }
    }
}

#[inline]
fn chebyshev(a: u8, b: u8) -> u8 {
    let ar = (a / 8) as i8; let af = (a % 8) as i8;
    let br = (b / 8) as i8; let bf = (b % 8) as i8;
    let dr = (ar - br).unsigned_abs();
    let df = (af - bf).unsigned_abs();
    dr.max(df)
}

/// Quiescence search at depth-0 boundary of the main alpha-beta search.
///
/// `ply`    — cumulative ply (used for mate-distance encoding, shared with main search).
/// `qs_ply` — plies *within* QS (used for the MAX_QS_PLY cap). Caller passes 0.
pub(super) fn quiesce(
    pos: &mut Position,
    mut alpha: i32,
    mut beta:  i32,
    ply: i32,
    qs_ply: i32,
    ctx: &mut SearchCtx,
) -> i32 {
    ctx.nodes += 1;

    if ctx.nodes & TIME_CHECK_MASK == 0 {
        if let Some(d) = ctx.deadline {
            if now_ms() >= d { ctx.aborted = true; return 0; }
        }
    }

    if let Some(r) = pos.game_result {
        return match r {
            GameResult::P1Wins =>  MATE_SCORE - ply,
            GameResult::P2Wins => -MATE_SCORE + ply,
        };
    }

    if qs_ply >= MAX_QS_PLY || ply >= MAX_PLY {
        return evaluate(pos);
    }

    let in_check = is_king_threatened(pos, pos.to_move);
    let static_eval = evaluate(pos);
    let maximising = pos.to_move == Player::P1;

    // Stand-pat — skip when in check (otherwise side-to-move can "stand still"
    // while the King is captured next ply).
    if !in_check {
        if maximising {
            if static_eval >= beta { return beta; }
            if static_eval > alpha { alpha = static_eval; }
        } else {
            if static_eval <= alpha { return alpha; }
            if static_eval < beta   { beta  = static_eval; }
        }
    }

    let moves = generator::generate(pos);
    if moves.is_empty() {
        // Non-terminal position with no legal actions shouldn't happen
        // (generator always emits EndPhase as a fallback) — but if it does,
        // returning static_eval is the safe choice.
        return static_eval;
    }

    let mut best = if in_check {
        if maximising { -INF } else { INF }
    } else {
        static_eval
    };

    for a in moves {
        if !in_check && !is_loud(a, pos) { continue; }

        let undo = make_unmake::make(pos, a);
        let s = quiesce(pos, alpha, beta, ply + 1, qs_ply + 1, ctx);
        make_unmake::unmake(pos, &undo);
        if ctx.aborted { return 0; }

        if maximising {
            if s > best { best = s; }
            if best > alpha { alpha = best; }
        } else {
            if s < best { best = s; }
            if best < beta  { beta = best; }
        }
        if alpha >= beta { break; }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::action::ActionKind;
    use crate::game_logic::skills::Skill;
    use crate::state::{Bitboard, MailboxEntry, Position};
    use crate::state::position::{Phase, Player};

    fn place(p: &mut Position, sq: u8, player: Player, kind: u8, entry: MailboxEntry) {
        let bit = Bitboard::from_square(sq);
        match player {
            Player::P1 => p.p1_pieces = p.p1_pieces | bit,
            Player::P2 => p.p2_pieces = p.p2_pieces | bit,
        }
        match kind {
            0 => p.kings     = p.kings     | bit,
            1 => p.champions = p.champions | bit,
            _ => p.guards    = p.guards    | bit,
        }
        p.mailbox[sq as usize] = entry;
    }

    // --- is_loud ---------------------------------------------------------

    #[test]
    fn is_loud_move_attack_true() {
        let pos = Position::empty();
        let a = Action::encode_move_attack(/*src*/10, /*target*/20, /*choice*/0, /*approach*/12);
        assert!(is_loud(a, &pos));
    }

    #[test]
    fn is_loud_plain_move_false() {
        let pos = Position::empty();
        let a = Action::encode(10, 20, ActionKind::Move, 0, 0);
        assert!(!is_loud(a, &pos));
    }

    #[test]
    fn is_loud_strike_skills_true() {
        let pos = Position::empty();
        for s in [Skill::Lance, Skill::Hook, Skill::Break, Skill::Steal, Skill::Tempest, Skill::Blast] {
            let a = Action::encode(10, 20, ActionKind::Skill, s as u8, 0);
            assert!(is_loud(a, &pos), "skill {:?} should be loud", s);
        }
    }

    #[test]
    fn is_loud_support_skills_false() {
        let pos = Position::empty();
        for s in [Skill::Shield, Skill::Heal, Skill::Plate, Skill::Focus, Skill::Charge,
                  Skill::Dash, Skill::Retreat, Skill::Shove, Skill::Swap] {
            let a = Action::encode(10, 20, ActionKind::Skill, s as u8, 0);
            assert!(!is_loud(a, &pos), "skill {:?} should NOT be loud", s);
        }
    }

    #[test]
    fn is_loud_endphase_endturn_false() {
        let pos = Position::empty();
        let ep = Action::encode(0, 0, ActionKind::EndPhase, 0, 0);
        let et = Action::encode(0, 0, ActionKind::EndTurn,  0, 0);
        assert!(!is_loud(ep, &pos));
        assert!(!is_loud(et, &pos));
    }

    #[test]
    fn is_loud_bodyguard_choice_true() {
        let pos = Position::empty();
        let bg = Action::encode_bodyguard_choice(2);
        assert!(is_loud(bg, &pos));
    }

    #[test]
    fn is_loud_draft_turn_false() {
        let pos = Position::empty();
        let dr = Action::encode_draft_turn(1, 0, 0, 2, 1, 1);
        assert!(!is_loud(dr, &pos));
    }

    // --- is_king_threatened ----------------------------------------------

    /// P1 King at 36, P2 Champion adjacent at 35. P2's Move-Attack (35→36)
    /// damages the King → P1 King is threatened.
    #[test]
    fn king_threatened_by_adjacent_enemy_move_attack() {
        let mut pos = Position::empty();
        place(&mut pos, 36, Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 35, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        assert!(is_king_threatened(&mut pos, Player::P1));
    }

    /// P1 King at 0, P2 Champion at 63. Far apart, no Move-Attack can reach.
    #[test]
    fn king_not_threatened_by_distant_piece() {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        assert!(!is_king_threatened(&mut pos, Player::P1));
    }

    /// P1 King at 36, P2 Champion at 35 with Lance skill, P2 has money for it.
    /// Skill-Phase: Lance from 35 to 36 damages the King.
    #[test]
    fn king_threatened_by_in_range_lance() {
        let mut pos = Position::empty();
        place(&mut pos, 36, Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 35, Player::P2, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Skill;
        pos.actions_remaining = 2;
        pos.p2_money = 6;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        assert!(is_king_threatened(&mut pos, Player::P1));
    }

    #[test]
    fn king_threatened_returns_false_when_king_missing() {
        let mut pos = Position::empty();
        // No P1 King placed.
        place(&mut pos, 35, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.to_move = Player::P1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        assert!(!is_king_threatened(&mut pos, Player::P1));
    }

    // --- quiesce ---------------------------------------------------------

    fn fresh_tt() -> crate::search::transposition::TranspositionTable {
        crate::search::transposition::TranspositionTable::with_capacity_pow2(12)
    }

    /// Mate-in-1 must still be found when search depth is high enough that
    /// the capture line resolves inside quiescence at the leaf.
    #[test]
    fn mate_in_1_propagates_through_quiescence() {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
        place(&mut pos, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 36, Player::P2, 0, MailboxEntry::default().with_hp(1));
        pos.to_move = Player::P1;
        pos.current_phase = Phase::Move;
        pos.actions_remaining = 2;
        pos.round_number = 1;
        pos.zobrist = crate::state::zobrist::full_recompute(&pos);
        let mut tt = fresh_tt();
        let r = crate::search::alpha_beta::find_best(&mut pos, &mut tt, 0, 4);
        assert!(r.score.abs() > MATE_SCORE - 128, "expected mate score, got {}", r.score);
        assert!(r.score > 0, "P1 mate must be positive, got {}", r.score);
    }

    /// Mate-distance must be stable across max_depth — QS must not inflate
    /// the mate-distance count by treating capture moves as non-terminal.
    #[test]
    fn mate_score_distance_invariant_with_qs() {
        fn build() -> Position {
            let mut pos = Position::empty();
            place(&mut pos, 0,  Player::P1, 0, MailboxEntry::default().with_hp(2));
            place(&mut pos, 28, Player::P1, 1, MailboxEntry::default().with_hp(2));
            place(&mut pos, 36, Player::P2, 0, MailboxEntry::default().with_hp(1));
            pos.to_move = Player::P1;
            pos.current_phase = Phase::Move;
            pos.actions_remaining = 2;
            pos.round_number = 1;
            pos.zobrist = crate::state::zobrist::full_recompute(&pos);
            pos
        }
        let mut p1 = build(); let mut tt1 = fresh_tt();
        let r1 = crate::search::alpha_beta::find_best(&mut p1, &mut tt1, 0, 1);
        let mut p2 = build(); let mut tt2 = fresh_tt();
        let r2 = crate::search::alpha_beta::find_best(&mut p2, &mut tt2, 0, 2);
        let mut p4 = build(); let mut tt4 = fresh_tt();
        let r4 = crate::search::alpha_beta::find_best(&mut p4, &mut tt4, 0, 4);
        assert_eq!(r1.score, r2.score,
            "mate score drifted between depth 1 and 2: {} vs {}", r1.score, r2.score);
        assert_eq!(r2.score, r4.score,
            "mate score drifted between depth 2 and 4: {} vs {}", r2.score, r4.score);
    }

    /// QS must not break `make` / `unmake` symmetry.
    #[test]
    fn unmake_perfectly_restores_position_through_qs() {
        let mut pos = Position::setup_stack_m();
        let zobrist_before   = pos.zobrist;
        let p1_pieces_before = pos.p1_pieces.0;
        let p2_pieces_before = pos.p2_pieces.0;
        let to_move_before   = pos.to_move;
        let mut tt = fresh_tt();
        let _ = crate::search::alpha_beta::find_best(&mut pos, &mut tt, 0, 2);
        assert_eq!(pos.zobrist, zobrist_before, "zobrist drifted across search+QS");
        assert_eq!(pos.p1_pieces.0, p1_pieces_before);
        assert_eq!(pos.p2_pieces.0, p2_pieces_before);
        assert_eq!(pos.to_move, to_move_before);
    }
}
