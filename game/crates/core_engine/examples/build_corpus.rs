//! Corpus builder for the search benchmark (v3).
//!
//! Plays N games with *realistic* skill loadouts and *search-driven* play
//! (depth-2 alpha-beta as the policy for both sides) and emits FEN snapshots
//! covering the 6 search-behaviour buckets we want to measure:
//!
//!   A. opening-with-skills      — round 1-3, Move phase, full board, drafted loadouts.
//!   B. midgame-move             — round 4-8, Move phase, some pieces engaged.
//!   C. skill-phase-full         — Skill phase with ≥3 money and pieces with skills.
//!   D. combo-loaded             — tracked_casters or tracked_enemies non-empty.
//!   F. endgame-with-skills      — round 12+, ≤10 pieces per side, real loadouts.
//!   G. king-in-danger           — enemy Champion within Chebyshev 2 of a King.
//!
//! Category E (mate-in-N) is hand-curated separately; the builder does not
//! attempt to synthesise tactical positions.
//!
//! Why search-driven play rather than random: uniform-random play produces
//! positions no human/AI would ever reach — a Champion casting Dash on itself
//! to hurl into enemy territory in round 1, for instance. Every downstream
//! position inherits that unrealism. Search-driven play (depths 2/3/4 cycled
//! across games) is deep enough to see immediate replies (won't burn a skill
//! for no gain) but cheap enough to run hundreds of games in minutes. The
//! depth cycling matters because a single fixed depth produces near-identical
//! opening lines game-to-game; varying depth picks different opening moves
//! and cascades into genuinely different downstream positions.
//!
//! Diversity: `MAX_PER_BUCKET_PER_GAME` caps how many positions one game can
//! contribute to any single bucket, so a long game can't monopolise a bucket
//! with 20 near-identical late-game positions. Two-layer dedup: zobrist for
//! fast exact-state and a "view-key" (board + STM + phase) that rejects
//! positions differing only in counter values (money / actions_remaining /
//! round) since those look identical in the inspector.
//!
//! Each snapshot is printed as one corpus row:
//!     <id>, <category>, -, -, <fen>
//!
//! Usage:
//!   cargo run -p core_engine --example build_corpus --release -- \
//!       --games 400 --seed 0xC0FFEE > game/bench/corpus/raw_corpus.txt
//! then hand-curate ~5 per bucket into corpus.txt.
//!
//! ## Loadout generation
//!
//! Per-side, we build a `SideLoadout` (12 skill slots) incrementally, weighted
//! random over skills that still fit under per-side caps:
//!   - Focus ≤ 2, Charge ≤ 2 (mystics are global; extras wasted)
//!   - Heal ≤ 3, Steal ≤ 3
//!   - Swap ≤ 2, Retreat ≤ 2
//!   - Everything else ≤ 4
//! and respecting the per-piece no-duplicate rule from `validate_loadout`.
//!
//! After the 12 slots are filled we check for category diversity (≥3 of the 4
//! categories present) and Strike presence; if either fails we force-swap
//! one or two slots to fix it. That guarantees a valid loadout in one pass.

use core_engine::game_logic::skills::{
    Skill, SkillCategory, SideLoadout, skill_category, skill_from_id, validate_loadout,
};
use core_engine::game_logic::{generator, make_unmake};
use core_engine::search::alpha_beta::find_best;
use core_engine::search::transposition::TranspositionTable;
use core_engine::state::Position;
use core_engine::state::fen::to_fen;
use core_engine::state::position::{Phase, Player};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::{HashMap, HashSet};

const MAX_PLIES: usize = 2_000;
const DEFAULT_GAMES: usize = 400;
const DEFAULT_SEED: u64 = 0xC0FF_EE12_3456_789A;

/// Emit up to this many candidates per bucket. User curates down to ~5.
const CANDIDATES_PER_BUCKET: usize = 20;

/// Diversity guard: cap candidates per (bucket, STM) per single game.
/// Prevents one long game from dominating a bucket with 20 near-identical
/// positions AND ensures both P1 and P2 turns get represented — a
/// per-bucket-only cap would fill up on P1's opening turns and never record
/// P2's, since search-driven play produces a deterministic P1-first sequence.
const MAX_PER_STM_PER_GAME: usize = 2;

/// Search depths cycled through as the play policy — game i uses depth
/// PLAY_DEPTHS[i % len]. Different depths pick different opening moves,
/// which produces genuinely different downstream games (not just different
/// loadouts on the same tactical line).
const PLAY_DEPTHS: [u8; 3] = [2, 3, 4];

// --- Skill loadout generation ----------------------------------------------

/// Per-side caps: max occurrences of any single skill across all 12 slots.
fn skill_cap(s: Skill) -> u8 {
    match s {
        Skill::Focus   => 2,
        Skill::Charge  => 2,
        Skill::Heal    => 3,
        Skill::Steal   => 3,
        Skill::Swap    => 2,
        Skill::Retreat => 2,
        _              => 4,
    }
}

const ALL_SKILLS: [Skill; 15] = [
    Skill::Lance, Skill::Hook, Skill::Break, Skill::Steal, Skill::Tempest,
    Skill::Shield, Skill::Heal, Skill::Plate,
    Skill::Dash, Skill::Blast, Skill::Shove, Skill::Swap, Skill::Retreat,
    Skill::Focus, Skill::Charge,
];

/// Build a plausible SideLoadout using weighted-incremental placement.
///
/// The 12 slots (6 pieces × 2 slots) are filled in a shuffled order. For each
/// slot we pick uniformly from the skills that:
///   (a) haven't hit their per-side cap yet, and
///   (b) don't already occupy the OTHER slot on the same piece.
///
/// After all slots are filled we check the category-diversity and Strike-
/// presence rules and force-swap slots if they fail. Loadout is then validated
/// via `validate_loadout` as a final sanity check (panics if we produced an
/// invalid one — indicates a bug in this function).
fn random_loadout(rng: &mut StdRng) -> SideLoadout {
    loop {
        let mut lo: SideLoadout = [(0, 0); 6];
        let mut counts: HashMap<u8, u8> = HashMap::new();

        // Slot order: (piece_idx, slot_idx), shuffled. This avoids biasing
        // slot-1 vs slot-2 fills.
        let mut order: Vec<(usize, usize)> = (0..6).flat_map(|p| [(p, 0), (p, 1)]).collect();
        order.shuffle(rng);

        for (piece, slot) in order {
            // Candidates: (a) below cap, (b) not equal to the *other* slot on this piece.
            let other = if slot == 0 { lo[piece].1 } else { lo[piece].0 };
            let candidates: Vec<Skill> = ALL_SKILLS
                .iter()
                .copied()
                .filter(|s| {
                    let id = *s as u8;
                    counts.get(&id).copied().unwrap_or(0) < skill_cap(*s)
                        && id != other
                })
                .collect();
            if candidates.is_empty() {
                // Extremely unlikely given the caps — restart from scratch.
                break;
            }
            let pick = *candidates.choose(rng).unwrap();
            let id = pick as u8;
            *counts.entry(id).or_insert(0) += 1;
            if slot == 0 { lo[piece].0 = id; } else { lo[piece].1 = id; }
        }

        // Any zero slot means the inner loop broke early — retry.
        if lo.iter().any(|(a, b)| *a == 0 || *b == 0) {
            continue;
        }

        // Category diversity + Strike presence checks.
        let mut cat_present = [false; 4];
        let mut strike_present = false;
        for (a, b) in lo.iter() {
            for &id in &[*a, *b] {
                if let Some(sk) = skill_from_id(id) {
                    let c = skill_category(sk);
                    cat_present[cat_idx(c)] = true;
                    if c == SkillCategory::Strike { strike_present = true; }
                }
            }
        }
        let n_cats = cat_present.iter().filter(|&&x| x).count();
        if n_cats < 3 || !strike_present {
            // Force-fix: swap one slot to introduce a missing category.
            // Simpler to just retry — retries are cheap.
            continue;
        }

        // Final validity check (per-piece dup, id range).
        if validate_loadout(&lo).is_ok() {
            return lo;
        }
    }
}

fn cat_idx(c: SkillCategory) -> usize {
    match c {
        SkillCategory::Strike => 0,
        SkillCategory::Shield => 1,
        SkillCategory::Move   => 2,
        SkillCategory::Mystic => 3,
    }
}

// --- Classification --------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cat {
    OpeningWithSkills,
    MidgameMove,
    SkillPhaseFull,
    ComboLoaded,
    EndgameWithSkills,
    KingInDanger,
}

fn cat_str(c: Cat) -> &'static str {
    match c {
        Cat::OpeningWithSkills => "opening-with-skills",
        Cat::MidgameMove       => "midgame-move",
        Cat::SkillPhaseFull    => "skill-phase-full",
        Cat::ComboLoaded       => "combo-loaded",
        Cat::EndgameWithSkills => "endgame-with-skills",
        Cat::KingInDanger      => "king-in-danger",
    }
}

fn piece_count(pos: &Position) -> u32 {
    (pos.p1_pieces | pos.p2_pieces).0.count_ones()
}

fn side_piece_count(pos: &Position, side: Player) -> u32 {
    match side {
        Player::P1 => pos.p1_pieces.0.count_ones(),
        Player::P2 => pos.p2_pieces.0.count_ones(),
    }
}

fn king_in_danger(pos: &Position) -> bool {
    // Any enemy Champion within Chebyshev-2 of a King. Chebyshev 2 = the
    // Champion could reach the King's square in one Move (speed=1) + one
    // Move-Attack via approach, OR a Guard could reach via speed-2. We use
    // Chebyshev 2 as the outer bound; that's the closest a real threat can be.
    let kings = pos.kings.0;
    if kings == 0 { return false; }
    let p1_champs = pos.p1_pieces.0 & pos.champions.0;
    let p2_champs = pos.p2_pieces.0 & pos.champions.0;
    let p1_kings  = pos.p1_pieces.0 & kings;
    let p2_kings  = pos.p2_pieces.0 & kings;

    fn any_within_2(kings_bb: u64, enemy_champs: u64) -> bool {
        let mut k = kings_bb;
        while k != 0 {
            let ks = k.trailing_zeros() as u8;
            k &= k - 1;
            let mut e = enemy_champs;
            while e != 0 {
                let es = e.trailing_zeros() as u8;
                e &= e - 1;
                let dx = ((ks % 8) as i32 - (es % 8) as i32).abs();
                let dy = ((ks / 8) as i32 - (es / 8) as i32).abs();
                if dx.max(dy) <= 2 { return true; }
            }
        }
        false
    }
    any_within_2(p1_kings, p2_champs) || any_within_2(p2_kings, p1_champs)
}

fn skill_money_available(pos: &Position) -> bool {
    // At least one side has ≥3 money (enough for most skills) and it's their
    // Skill phase turn.
    match pos.to_move {
        Player::P1 => pos.p1_money >= 3,
        Player::P2 => pos.p2_money >= 3,
    }
}

fn classify(pos: &Position) -> Option<Cat> {
    let r = pos.round_number;
    let phase = pos.current_phase;
    let pc = piece_count(pos);
    let combo_loaded = pos.tracked_casters_len > 0 || pos.tracked_enemies_len > 0;

    // King-in-danger overrides other buckets — it's the most search-stressing regime.
    if king_in_danger(pos) && pc >= 6 {
        return Some(Cat::KingInDanger);
    }

    // Combo-loaded — any Skill phase with tracked entity present.
    if combo_loaded && phase == Phase::Skill {
        return Some(Cat::ComboLoaded);
    }

    // Skill-phase-full — Skill phase, mover has money, at least a few pieces
    // on the board with skills.
    if phase == Phase::Skill && skill_money_available(pos) && pc >= 10 {
        return Some(Cat::SkillPhaseFull);
    }

    // Opening — early rounds, Move phase, both sides mostly intact.
    if r <= 3 && phase == Phase::Move
        && side_piece_count(pos, Player::P1) >= 6
        && side_piece_count(pos, Player::P2) >= 6
    {
        return Some(Cat::OpeningWithSkills);
    }

    // Midgame-move — rounds 4-8, Move phase, plenty of pieces.
    if (4..=8).contains(&r) && phase == Phase::Move && pc >= 14 {
        return Some(Cat::MidgameMove);
    }

    // Endgame — late round, board thinned.
    if r >= 12 && pc <= 20 && pc >= 4 {
        return Some(Cat::EndgameWithSkills);
    }

    None
}

// --- Main ------------------------------------------------------------------

fn parse_args() -> (usize, u64) {
    let mut games = DEFAULT_GAMES;
    let mut seed = DEFAULT_SEED;
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--games" => {
                games = argv[i + 1].parse().expect("--games number");
                i += 2;
            }
            "--seed" => {
                let s = &argv[i + 1];
                seed = if let Some(hex) = s.strip_prefix("0x") {
                    u64::from_str_radix(hex, 16).expect("hex seed")
                } else {
                    s.parse().expect("decimal seed")
                };
                i += 2;
            }
            other => {
                eprintln!("unknown arg: {}", other);
                std::process::exit(2);
            }
        }
    }
    (games, seed)
}

fn main() {
    let (n_games, seed) = parse_args();
    eprintln!("# build_corpus v3 (search play depths={:?}): games={} seed=0x{:016X}", PLAY_DEPTHS, n_games, seed);

    let mut buckets: HashMap<Cat, Vec<String>> = HashMap::new();
    let mut seen: HashSet<u64> = HashSet::new();       // zobrist dedup (fast path)
    let mut seen_view: HashSet<String> = HashSet::new(); // board+STM+phase dedup (visual)

    let mut diag_max_round = 0u16;
    let mut diag_skill_phase_plies = 0u64;
    let mut diag_skill_money_plies = 0u64;
    let mut diag_combo_loaded = 0u64;
    let mut diag_king_danger = 0u64;

    for g in 0..n_games {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(g as u64));
        let play_depth = PLAY_DEPTHS[g % PLAY_DEPTHS.len()];
        let p1_loadout = random_loadout(&mut rng);
        let p2_loadout = random_loadout(&mut rng);
        let mut pos = Position::setup_stack_m_with_loadouts(&p1_loadout, &p2_loadout);
        let mut plies = 0usize;
        // Fresh TT per game — avoids cross-game contamination and keeps memory bounded.
        let mut tt = TranspositionTable::with_capacity_mb(16);
        // Per-(bucket, STM) cap for THIS game. Keyed by (Cat, Player) so both
        // P1 and P2 turns can land, but neither monopolises.
        let mut per_stm_this_game: HashMap<(Cat, Player), usize> = HashMap::new();

        while plies < MAX_PLIES {
            if pos.game_result.is_some() { break; }
            let moves = generator::generate(&pos);
            if moves.is_empty() { break; }

            // Diagnostics.
            if pos.round_number > diag_max_round { diag_max_round = pos.round_number; }
            if pos.current_phase == Phase::Skill {
                diag_skill_phase_plies += 1;
                if skill_money_available(&pos) { diag_skill_money_plies += 1; }
            }
            if pos.tracked_casters_len > 0 || pos.tracked_enemies_len > 0 {
                diag_combo_loaded += 1;
            }
            if king_in_danger(&pos) { diag_king_danger += 1; }

            // Classify + record. Two-layer dedup:
            //  (a) zobrist — fast exact-state dedup.
            //  (b) view-key — board + STM + phase. Positions differing only in
            //      actions_remaining / money / round counters look identical
            //      in the inspector, so they're worthless as separate corpus
            //      rows.
            // Diversity guard: skip if this game already contributed
            // MAX_PER_STM_PER_GAME to this (bucket, STM) combo. Ensures both
            // players' turns are represented in every bucket.
            if let Some(cat) = classify(&pos) {
                let key = (cat, pos.to_move);
                let per_game_count = per_stm_this_game.get(&key).copied().unwrap_or(0);
                if per_game_count < MAX_PER_STM_PER_GAME {
                    let fen = to_fen(&pos);
                    let view_key: String = fen.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
                    if seen.insert(pos.zobrist) && seen_view.insert(view_key) {
                        let bucket = buckets.entry(cat).or_insert_with(Vec::new);
                        if bucket.len() < CANDIDATES_PER_BUCKET {
                            bucket.push(fen);
                            *per_stm_this_game.entry(key).or_insert(0) += 1;
                        }
                    }
                }
            }

            // Search-driven play policy: depth-N alpha-beta (N cycles through
            // PLAY_DEPTHS across games) picks the move. Falls back to a random
            // legal move if the search returns no best (shouldn't happen for
            // non-terminal positions, but defensive).
            let pick = {
                let res = find_best(&mut pos, &mut tt, 0, play_depth);
                res.best.unwrap_or_else(|| *moves.choose(&mut rng).unwrap())
            };

            let _undo = make_unmake::make(&mut pos, pick);
            plies += 1;
        }
    }

    eprintln!("# diag: max_round={} skill_phase_plies={} skill_money_plies={} combo_loaded_plies={} king_danger_plies={}",
        diag_max_round, diag_skill_phase_plies, diag_skill_money_plies, diag_combo_loaded, diag_king_danger);

    let order = [
        Cat::OpeningWithSkills,
        Cat::MidgameMove,
        Cat::SkillPhaseFull,
        Cat::ComboLoaded,
        Cat::EndgameWithSkills,
        Cat::KingInDanger,
    ];
    println!("# Auto-generated corpus rows (v2). Hand-curate before committing.");
    println!("# Format: id, category, expected_depth_n, expected_score_range, fen [; expected_best_move_raw,...]");
    println!("# Curation target: ~5 rows per bucket after review.");
    for cat in order {
        let bucket = buckets.get(&cat).cloned().unwrap_or_default();
        let label = cat_str(cat);
        eprintln!("# {}: {} candidates available", label, bucket.len());
        println!();
        println!("# --- {} ({} candidates) ---", label, bucket.len());
        for (i, fen) in bucket.iter().enumerate() {
            println!("{}-{:02}, {}, -, -, {}", label, i + 1, label, fen);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_loadout_passes_validation() {
        for seed in 0..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            let lo = random_loadout(&mut rng);
            validate_loadout(&lo).expect("random_loadout produced invalid loadout");
        }
    }

    #[test]
    fn random_loadout_respects_caps() {
        for seed in 0..20 {
            let mut rng = StdRng::seed_from_u64(seed);
            let lo = random_loadout(&mut rng);
            let mut counts: HashMap<u8, u8> = HashMap::new();
            for (a, b) in lo.iter() {
                *counts.entry(*a).or_insert(0) += 1;
                *counts.entry(*b).or_insert(0) += 1;
            }
            for (id, n) in counts.iter() {
                if *id == 0 { panic!("zero slot in loadout"); }
                let sk = skill_from_id(*id).unwrap();
                assert!(*n <= skill_cap(sk),
                    "skill {:?} appears {} times, cap {}", sk, n, skill_cap(sk));
            }
        }
    }

    #[test]
    fn random_loadout_has_strike_and_diverse_categories() {
        for seed in 0..20 {
            let mut rng = StdRng::seed_from_u64(seed);
            let lo = random_loadout(&mut rng);
            let mut cats = [false; 4];
            let mut has_strike = false;
            for (a, b) in lo.iter() {
                for &id in &[*a, *b] {
                    let sk = skill_from_id(id).unwrap();
                    let c = skill_category(sk);
                    cats[cat_idx(c)] = true;
                    if c == SkillCategory::Strike { has_strike = true; }
                }
            }
            assert!(has_strike, "no Strike skill in loadout");
            let n = cats.iter().filter(|&&x| x).count();
            assert!(n >= 3, "only {} categories present", n);
        }
    }
}
