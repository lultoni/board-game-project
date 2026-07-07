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
use crate::state::position::{GameResult, Phase, Player};
use crate::state::magic;
use crate::game_logic::skills::{
    Skill, SkillCategory, TargetOwner, skill_from_id, skill_cost, skill_category,
    skill_default_range, skill_target_owner,
};

pub const MATE_SCORE: i32 = 1_000_000;

// === Slice 9: material weights ===========================================
//
// Order of magnitude: one Champion >> one HP swing >> one armor swing >>
// one coin. MATE_SCORE (1_000_000) is three orders above any plausible
// material sum (~24 pieces × ~1500 each ≈ 36k), so terminals never compete.

/// King material weight = 0. The king's presence/absence is *already*
/// encoded by the MATE_SCORE branch above; counting it again would only
/// "reward" malformed positions (king missing, `game_result == None`).
const KING_MATERIAL:   i32 = 0;
const CHAMPION_VALUE:  i32 = 1000;
const GUARD_VALUE:     i32 = 600;
const HP_PER_POINT:    i32 = 150;
const ARMOR_PER_POINT: i32 = 120;
const MONEY_PER_UNIT:  i32 = 25;

// Mobility scoring: reward pieces for having reachable squares.
// Guards use BFS-2 (speed=2) discounting occupied squares; Champions/Kings
// use the 8-adjacent mask discounting own pieces.  Weights are small relative
// to material so positional advantage doesn't overshadow piece count.
const GUARD_MOB_PER_SQ:   i32 = 8;   // centre Guard (20 reachable) ≈ 160 pts
const CHAMP_MOB_PER_SQ:   i32 = 12;  // centre Champ (8 free) ≈ 96 pts

// Skill-activity weights. Kept small relative to mobility (which is 8–12/sq)
// so they nudge play toward useful casts without swamping material.
const STRIKE_PER_TARGET:  i32 = 6;   // per enemy in Strike range
const MOVE_PER_DEST:      i32 = 3;   // per legal destination (Dash/Retreat) or per pushable target (Shove/Blast) or per swap partner
const SHIELD_PER_TARGET:  i32 = 5;   // per Heal/Plate ally that would actually benefit
const SHIELD_SELF:        i32 = 5;   // Shield if own armor < cap
const MYSTIC_FLAG_BONUS:  i32 = 20;  // per Focus/Charge that has a real follow-up this turn

// Stack M caps.
const ARMOR_CAP:          u8 = 2;
const HP_CAP:             u8 = 2;

// PLACEHOLDER. A balance-slice will replace this with a tuned table once we
// have playtest data. The current scheme — cost × 40 + range bonus + category
// bonus — keeps each skill in a sensible 50..=220 range (well under
// CHAMPION_VALUE) and orders skills roughly by their resource cost. It is
// *consistent* (deterministic), so alpha-beta will still prefer the
// objectively better of two material-equivalent positions; it is just not
// strictly correct in absolute terms.
#[inline]
fn skill_value(s: Skill) -> i32 {
    let base = skill_cost(s) as i32 * 40;
    let range_bonus = match skill_default_range(s) {
        0 => 0, 1 => 10, 2 => 20, _ => 30,
    };
    let category_bonus = match skill_category(s) {
        SkillCategory::Strike => 30,
        SkillCategory::Move   => 20,
        SkillCategory::Shield => 15,
        SkillCategory::Mystic => 10,
    };
    base + range_bonus + category_bonus
}

/// Per-component decomposition of the static eval. `total` is exactly what
/// `evaluate()` returns (so L3 sees zero behaviour change). The per-bucket
/// fields are sign-corrected: P1 contributions go to `*_p1`, P2 to `*_p2`,
/// both as positive magnitudes. `total = sum(*_p1) - sum(*_p2)` (terminal
/// short-circuit aside).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvalBreakdown {
    pub material_p1:  i32,
    pub material_p2:  i32,
    pub hp_p1:        i32,
    pub hp_p2:        i32,
    pub armor_p1:     i32,
    pub armor_p2:     i32,
    pub skills_p1:    i32,
    pub skills_p2:    i32,
    pub money_p1:     i32,
    pub money_p2:     i32,
    pub mobility_p1:  i32,
    pub mobility_p2:  i32,
    /// Pre-priced "hanging piece" credit: 0.25× the value of enemy pieces
    /// this side could move-attack this turn. Symmetric across sides so the
    /// leaf eval doesn't flip sign with the side-to-move — the attacker's
    /// pending gain is already reflected before the capture ply resolves.
    pub threat_p1:    i32,
    pub threat_p2:    i32,
    /// Active-skill activity credit: per-target for Strike/Move/Shield (money-
    /// and legality-gated), single flag for Mystic (Focus/Charge) gated on an
    /// affordable follow-on active skill actually having ≥1 legal action.
    pub skill_act_p1: i32,
    pub skill_act_p2: i32,
    pub total:        i32,
}

pub fn evaluate(pos: &Position) -> i32 {
    evaluate_breakdown(pos).total
}

/// Position-rater interface. The search calls `evaluate` once per leaf; an
/// `Evaluator` impl returns a P1-POV score in the same units as the free
/// `evaluate()` function above (positive = P1, ±MATE_SCORE for terminals).
///
/// Two impls are planned: `HeuristicEvaluator` wraps today's hand-coded eval
/// (zero-behaviour-change default); a future `NnEvaluator` will host the
/// trained position rater (`design/inbox/digital/nn-rater-plan.md`).
///
/// **Send-only** bound: the search itself is single-threaded but evaluators
/// are owned by `Match` (one per AI seat), which lives on a worker thread
/// and gets moved between thread-pool tasks via `tauri::async_runtime`.
/// Code that needs to share an evaluator across threads (e.g. the tier-2
/// gauntlet's predecessor list) re-asserts `+ Sync` locally.
pub trait Evaluator: Send {
    fn evaluate(&self, pos: &Position) -> i32;
    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown;
}

/// Zero-size wrapper around the free `evaluate()` / `evaluate_breakdown()`
/// functions. The default evaluator everywhere — preserves S36 behaviour.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicEvaluator;

impl Evaluator for HeuristicEvaluator {
    #[inline]
    fn evaluate(&self, pos: &Position) -> i32 { evaluate(pos) }
    #[inline]
    fn evaluate_breakdown(&self, pos: &Position) -> EvalBreakdown { evaluate_breakdown(pos) }
}

pub fn evaluate_breakdown(pos: &Position) -> EvalBreakdown {
    // (a) Terminal — overrules everything. Per-bucket fields stay zero;
    //     only `total` carries the ±MATE_SCORE.
    match pos.game_result {
        Some(GameResult::P1Wins) => return EvalBreakdown { total:  MATE_SCORE, ..Default::default() },
        Some(GameResult::P2Wins) => return EvalBreakdown { total: -MATE_SCORE, ..Default::default() },
        None => {}
    }

    let mut b = EvalBreakdown::default();

    let all_occ = (pos.p1_pieces | pos.p2_pieces).0;

    // (b) Single pass over occupied bits.
    let mut bits = all_occ;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let mask = 1u64 << sq;
        let m = pos.mailbox[sq as usize];
        let is_p1 = pos.p1_pieces.0 & mask != 0;

        let is_guard    = pos.guards.0    & mask != 0;
        let _is_champion = pos.champions.0 & mask != 0;

        let material =
            if      pos.kings.0     & mask != 0 { KING_MATERIAL }
            else if pos.champions.0 & mask != 0 { CHAMPION_VALUE }
            else                                { GUARD_VALUE };
        let hp_term    = HP_PER_POINT    * m.hp()    as i32;
        let armor_term = ARMOR_PER_POINT * m.armor() as i32;
        let mut skill_term = 0;
        if let Some(sk) = skill_from_id(m.skill1()) { skill_term += skill_value(sk); }
        if let Some(sk) = skill_from_id(m.skill2()) { skill_term += skill_value(sk); }

        // Mobility: count squares the piece can actually reach given board state.
        // Guards: BFS-2 discounting all occupied squares.
        // Champions/Kings: 8-adjacent discounting own pieces.
        let own_bb = if is_p1 { pos.p1_pieces.0 } else { pos.p2_pieces.0 };
        let mob_score = if is_guard {
            magic::movement_targets_speed2(sq, all_occ).0.count_ones() as i32
                * GUARD_MOB_PER_SQ
        } else {
            // Champion and King: 8-adjacent minus own pieces
            (magic::movement_targets_speed1(sq).0 & !own_bb).count_ones() as i32
                * CHAMP_MOB_PER_SQ
        };

        if is_p1 {
            b.material_p1  += material;
            b.hp_p1        += hp_term;
            b.armor_p1     += armor_term;
            b.skills_p1    += skill_term;
            b.mobility_p1  += mob_score;
        } else {
            b.material_p2  += material;
            b.hp_p2        += hp_term;
            b.armor_p2     += armor_term;
            b.skills_p2    += skill_term;
            b.mobility_p2  += mob_score;
        }
    }

    // (c) Money is global, not per-square.
    b.money_p1 = MONEY_PER_UNIT * pos.p1_money as i32;
    b.money_p2 = MONEY_PER_UNIT * pos.p2_money as i32;

    // (d) Threat-symmetric term: pre-price hanging pieces so the eval doesn't
    // sign-flip between P1's turn and P2's turn just because one side happens
    // to be side-to-move at the leaf. For each capturable enemy piece, credit
    // the attacker 0.25× the target's value (material + hp + armor). Kings
    // are excluded (their capture goes through the MATE_SCORE branch and would
    // otherwise be double-counted at MATE_SCORE weight). Skipped in Draft phase
    // where pieces aren't on the board yet.
    if pos.current_phase != Phase::Draft {
        let non_kings = all_occ & !pos.kings.0;
        let p1_threats = magic::threat_bb(
            pos.p1_pieces.0,
            pos.p1_pieces.0 & pos.guards.0,
            pos.p2_pieces.0 & non_kings,
            all_occ,
        ).0;
        let p2_threats = magic::threat_bb(
            pos.p2_pieces.0,
            pos.p2_pieces.0 & pos.guards.0,
            pos.p1_pieces.0 & non_kings,
            all_occ,
        ).0;
        b.threat_p1 = threat_value(pos, p1_threats);
        b.threat_p2 = threat_value(pos, p2_threats);

        // (e) Skill-activity: credit only skills that could actually do
        // something right now (money available, legal targets exist). See
        // `skill_activity` for the per-category rules.
        b.skill_act_p1 = skill_activity(pos, Player::P1);
        b.skill_act_p2 = skill_activity(pos, Player::P2);
    }

    b.total =
        (b.material_p1 - b.material_p2) +
        (b.hp_p1       - b.hp_p2)       +
        (b.armor_p1    - b.armor_p2)    +
        (b.skills_p1   - b.skills_p2)   +
        (b.money_p1    - b.money_p2)    +
        (b.mobility_p1 - b.mobility_p2) +
        (b.threat_p1   - b.threat_p2)   +
        (b.skill_act_p1 - b.skill_act_p2);
    b
}

/// Sum 0.25× (material + hp_value + armor_value) over all squares in `mask`.
/// Values match the main eval's weights so a hanging Champion pre-prices at
/// exactly 25% of what capturing it would net.
#[inline]
fn threat_value(pos: &Position, mask: u64) -> i32 {
    let mut acc = 0i32;
    let mut bits = mask;
    while bits != 0 {
        let sq = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let bit = 1u64 << sq;
        let m = pos.mailbox[sq as usize];
        let material = if pos.champions.0 & bit != 0 { CHAMPION_VALUE }
                       else if pos.guards.0 & bit != 0 { GUARD_VALUE }
                       else { 0 };
        let raw = material + HP_PER_POINT * m.hp() as i32 + ARMOR_PER_POINT * m.armor() as i32;
        acc += raw / 4;
    }
    acc
}

/// Skill-activity term for one side. Only credits skills that could actually
/// be used this turn: caster affords the cost AND ≥1 legal target/destination
/// exists. Mystic (Focus/Charge) get a single flag bonus gated on the caster
/// having an affordable, legally-usable follow-up active skill this turn.
///
/// Cost budget: for each equipped skill on each of `side`'s pieces, we call
/// `magic::skill_attacks` once (O(1)) and count set bits in the result. That's
/// ~24 pieces × 2 slots × O(1) = ~48 lookups per leaf. Cheap.
fn skill_activity(pos: &Position, side: Player) -> i32 {
    let (own_bb, opp_bb, own_money) = match side {
        Player::P1 => (pos.p1_pieces.0, pos.p2_pieces.0, pos.p1_money),
        Player::P2 => (pos.p2_pieces.0, pos.p1_pieces.0, pos.p2_money),
    };
    let all_occ = pos.p1_pieces.0 | pos.p2_pieces.0;
    let mut acc = 0i32;

    let mut bits = own_bb;
    while bits != 0 {
        let src = bits.trailing_zeros() as u8;
        bits &= bits - 1;
        let m = pos.mailbox[src as usize];

        // Detect Focus/Charge modifiers already in play for this piece to
        // avoid crediting the mystic flag AND the buffed follow-up range.
        // Cheap approximation: we use `skill_default_range` throughout and
        // don't apply the +1 for a pending Focus. Not perfectly accurate but
        // conservative — it always undercounts, never over.

        // Iterate this piece's two skill slots.
        for slot in 0u8..2 {
            let sid = if slot == 0 { m.skill1() } else { m.skill2() };
            let Some(sk) = skill_from_id(sid) else { continue };

            let cost = skill_cost(sk) as u16;
            if own_money < cost { continue; }

            acc += skill_slot_credit(pos, side, sk, src, own_bb, opp_bb, all_occ, own_money, m);
        }
    }
    acc
}

/// Per-skill target/destination counting, returning the eval credit for one
/// slot. Split out so the outer loop stays readable.
#[inline]
fn skill_slot_credit(
    pos: &Position,
    side: Player,
    sk: Skill,
    src: u8,
    own_bb: u64,
    opp_bb: u64,
    all_occ: u64,
    own_money: u16,
    m: crate::state::MailboxEntry,
) -> i32 {
    let range = skill_default_range(sk);
    let owner = skill_target_owner(sk);
    let cat = skill_category(sk);

    // Mystic (Focus/Charge): flag bonus gated on an affordable, castable
    // follow-up active skill on the SAME piece this turn.
    if matches!(cat, SkillCategory::Mystic) {
        let mystic_cost = skill_cost(sk) as u16;
        // Look at the OTHER slot on this piece for a follow-up.
        let other_sid = if m.skill1() == sk as u8 { m.skill2() } else { m.skill1() };
        let Some(follow) = skill_from_id(other_sid) else { return 0 };
        // Follow-up must be an active category (not another mystic modifier).
        if matches!(skill_category(follow), SkillCategory::Mystic) { return 0 };
        let follow_cost = skill_cost(follow) as u16;
        // Both must be affordable together.
        if own_money < mystic_cost + follow_cost { return 0 };
        // Follow-up must have ≥1 legal target from `src`.
        let follow_range = skill_default_range(follow);
        let follow_owner = skill_target_owner(follow);
        if slot_target_count(follow, follow_owner, follow_range, src, own_bb, opp_bb, all_occ, pos, side) == 0 {
            return 0;
        }
        return MYSTIC_FLAG_BONUS;
    }

    let n = slot_target_count(sk, owner, range, src, own_bb, opp_bb, all_occ, pos, side);
    if n == 0 { return 0; }

    match cat {
        SkillCategory::Strike => n as i32 * STRIKE_PER_TARGET,
        SkillCategory::Move   => n as i32 * MOVE_PER_DEST,
        SkillCategory::Shield => {
            // Shield (SelfOnly) contributes a fixed bonus if it would stick.
            if matches!(owner, TargetOwner::SelfOnly) { SHIELD_SELF }
            else { n as i32 * SHIELD_PER_TARGET }
        }
        SkillCategory::Mystic => 0, // handled above
    }
}

/// Count legal targets for one skill from square `src`. Cheap proxy —
/// approximates the generator's logic without duplicating it. Used only for
/// the eval's activity term.
#[inline]
fn slot_target_count(
    sk: Skill,
    owner: TargetOwner,
    range: u8,
    src: u8,
    own_bb: u64,
    opp_bb: u64,
    all_occ: u64,
    pos: &Position,
    _side: Player,
) -> u32 {
    match owner {
        TargetOwner::Enemy => {
            let ray = magic::skill_attacks(src, all_occ, range).0;
            (ray & opp_bb).count_ones()
        }
        TargetOwner::Ally => {
            let ray = magic::skill_attacks(src, all_occ, range).0;
            let candidates = ray & own_bb & !(1u64 << src);
            // Filter Heal/Plate: target must actually need it, else no credit.
            match sk {
                Skill::Heal => {
                    let mut n = 0u32;
                    let mut bits = candidates;
                    while bits != 0 {
                        let t = bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        if pos.mailbox[t].hp() < HP_CAP { n += 1; }
                    }
                    n
                }
                Skill::Plate => {
                    let mut n = 0u32;
                    let mut bits = candidates;
                    while bits != 0 {
                        let t = bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        if pos.mailbox[t].armor() < ARMOR_CAP { n += 1; }
                    }
                    n
                }
                _ => candidates.count_ones(), // Swap: any ally partner is valid
            }
        }
        TargetOwner::Either => {
            // Shove: any target square in range on a ray.
            let ray = magic::skill_attacks(src, all_occ, range).0;
            (ray & (own_bb | opp_bb) & !(1u64 << src)).count_ones()
        }
        TargetOwner::Empty => {
            // Dash/Retreat: empty squares within range on a queen-ray. Use
            // `skill_attacks(src, 0, range)` (unblocked) then subtract occupied.
            let all_ray = magic::skill_attacks(src, 0, range).0;
            (all_ray & !all_occ).count_ones()
        }
        TargetOwner::SelfOnly => {
            // Shield: 1 credit only if own armor < cap.
            let a = pos.mailbox[src as usize].armor();
            if a < ARMOR_CAP { 1 } else { 0 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Bitboard, MailboxEntry, Position};
    use crate::state::position::{GameResult, Player};

    /// Place a piece on `sq` for `player` of `kind` (0=King, 1=Champion, 2=Guard)
    /// with mailbox `entry`. Mirrors the structure of `make_unmake::tests::place`
    /// (which is pub(super)-scoped and not reachable from here).
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

    #[test]
    fn empty_board_is_zero() {
        let pos = Position::empty();
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn terminal_p1_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn terminal_p2_wins() {
        let mut pos = Position::empty();
        pos.game_result = Some(GameResult::P2Wins);
        assert_eq!(evaluate(&pos), -MATE_SCORE);
    }

    #[test]
    fn terminal_overrules_material() {
        // Place a P2 Champion (which would give P1 a negative material score)
        // but set game_result = P1Wins. Terminal must short-circuit the loop
        // and return exactly +MATE_SCORE.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
        pos.game_result = Some(GameResult::P1Wins);
        assert_eq!(evaluate(&pos), MATE_SCORE);
    }

    #[test]
    fn mirrored_single_champion_is_zero() {
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn hp_differential() {
        // P1 Champion HP=2 vs P2 Champion HP=1, no armor, no skills.
        // Differential is exactly HP_PER_POINT.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));
        assert_eq!(evaluate(&pos), HP_PER_POINT);
    }

    #[test]
    fn armor_differential() {
        // P1 Champion armor=1 vs P2 Champion armor=0, identical otherwise.
        let mut pos = Position::empty();
        place(&mut pos, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut pos, 63, Player::P2, 1, MailboxEntry::default().with_hp(2).with_armor(0));
        assert_eq!(evaluate(&pos), ARMOR_PER_POINT);
    }

    #[test]
    fn money_differential() {
        let mut pos = Position::empty();
        pos.p1_money = 10;
        pos.p2_money = 4;
        assert_eq!(evaluate(&pos), 6 * MONEY_PER_UNIT);
    }

    #[test]
    fn skill_equipped_beats_unequipped() {
        // P1 Champion with Lance equipped vs P2 Champion bare.
        // Both HP=2, no armor → differential is exactly skill_value(Lance).
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1,
            MailboxEntry::default().with_hp(2).with_skill1(Skill::Lance as u8));
        place(&mut pos, 63, Player::P2, 1,
            MailboxEntry::default().with_hp(2));
        assert_eq!(evaluate(&pos), skill_value(Skill::Lance));
    }

    #[test]
    fn stack_m_setup_is_zero() {
        // Canonical start: identical material on both sides, 6 money each.
        let pos = Position::setup_stack_m();
        assert_eq!(evaluate(&pos), 0);
    }

    #[test]
    fn sign_convention_p1_positive_p2_negative() {
        // A lone P1 Champion → positive score.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P1, 1, MailboxEntry::default().with_hp(2));
        assert!(evaluate(&pos) > 0);

        // Symmetric: a lone P2 Champion → negative.
        let mut pos = Position::empty();
        place(&mut pos, 0, Player::P2, 1, MailboxEntry::default().with_hp(2));
        assert!(evaluate(&pos) < 0);
    }

    #[test]
    fn additivity() {
        // Build three positions:
        //   A: P1 +1 HP advantage (P1 HP=2, P2 HP=1, no armor)
        //   B: P1 +1 armor advantage (HP=2 both, P1 armor=1, P2 armor=0)
        //   AB: both effects combined
        // Assert evaluate(AB) == evaluate(A) + evaluate(B).
        let mut a = Position::empty();
        place(&mut a, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2));
        place(&mut a, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));

        let mut b = Position::empty();
        place(&mut b, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut b, 63, Player::P2, 1, MailboxEntry::default().with_hp(2));

        let mut ab = Position::empty();
        place(&mut ab, 0,  Player::P1, 1, MailboxEntry::default().with_hp(2).with_armor(1));
        place(&mut ab, 63, Player::P2, 1, MailboxEntry::default().with_hp(1));

        assert_eq!(evaluate(&ab), evaluate(&a) + evaluate(&b));
    }

    #[test]
    fn maxed_piece_formula() {
        // Pin the math: single P1 Champion HP=2 armor=2 skill1=Tempest skill2=Charge,
        // empty money. Score must equal the explicit sum.
        let mut pos = Position::empty();
        place(&mut pos, 28, Player::P1, 1,
            MailboxEntry::default()
                .with_hp(2)
                .with_armor(2)
                .with_skill1(Skill::Tempest as u8)
                .with_skill2(Skill::Charge as u8));
        // Mobility: Champion at sq 28 (rank 3), 8 neighbours all free.
        // CHAMP_MOB_PER_SQ=12, 8 squares = 96.
        let mob = 8 * CHAMP_MOB_PER_SQ;
        let expected = CHAMPION_VALUE
            + 2 * HP_PER_POINT
            + 2 * ARMOR_PER_POINT
            + skill_value(Skill::Tempest)
            + skill_value(Skill::Charge)
            + mob;
        assert_eq!(evaluate(&pos), expected);
    }

    #[test]
    fn asymmetric_kings_no_panic() {
        // Malformed: P2 has a king, P1 doesn't, but game_result is None.
        // Eval must return a finite i32 without panicking.
        let mut pos = Position::empty();
        place(&mut pos, 4, Player::P2, 0, MailboxEntry::default().with_hp(2));
        // game_result stays None.
        let s = evaluate(&pos);
        // We don't assert a specific value — just that it computed.
        // KING_MATERIAL is 0, so the king contributes only its HP. P1 has nothing.
        // The point is: no panic, no overflow.
        assert!(s > i32::MIN && s < i32::MAX);
    }
}
