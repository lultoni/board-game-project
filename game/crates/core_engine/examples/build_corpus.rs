//! Corpus builder for the search benchmark.
//!
//! Plays N random Stack M games and emits FEN snapshots at well-chosen
//! plies covering the categories called out in
//! `design/inbox/digital/search-speed-benchmark-plan.md`:
//!
//!   opening               — round 0–2, both phases.
//!   midgame-low-skill     — round 6–12, Move phase, low piece interaction.
//!   midgame-high-skill    — round 6–14, Skill phase, modifier loaded.
//!   endgame-sparse        — late round, few pieces left.
//!   endgame-attrition     — late round, armour stacked.
//!   combo-loaded          — Skill phase with non-empty tracked_casters/enemies.
//!   phase-boundary        — first ply of Skill phase (boundary just crossed).
//!
//! Each snapshot is printed as one corpus row:
//!     <id>, <category>, -, -, <fen>
//!
//! Tactical/known-result positions are hand-added separately.
//!
//! Usage:
//!   cargo run -p core_engine --example build_corpus --release -- \
//!       --games 200 --seed 0xC0FFEE > /tmp/raw_corpus.txt
//! then hand-curate the rows to 20–50 representative entries.

use core_engine::game_logic::action::ActionKind;
use core_engine::game_logic::{generator, make_unmake};
use core_engine::state::Position;
use core_engine::state::fen::to_fen;
use core_engine::state::position::Phase;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

const MAX_PLIES: usize = 5_000;
const DEFAULT_GAMES: usize = 200;
const DEFAULT_SEED: u64 = 0xC0FF_EE12_3456_789A;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cat {
    Opening,
    MidgameLowSkill,
    MidgameHighSkill,
    EndgameSparse,
    EndgameAttrition,
    ComboLoaded,
    PhaseBoundary,
}

fn cat_str(c: Cat) -> &'static str {
    match c {
        Cat::Opening => "opening",
        Cat::MidgameLowSkill => "midgame-low-skill",
        Cat::MidgameHighSkill => "midgame-high-skill",
        Cat::EndgameSparse => "endgame-sparse",
        Cat::EndgameAttrition => "endgame-attrition",
        Cat::ComboLoaded => "combo-loaded",
        Cat::PhaseBoundary => "phase-boundary",
    }
}

fn piece_count(pos: &Position) -> u32 {
    (pos.p1_pieces | pos.p2_pieces).0.count_ones()
}

fn total_armour(pos: &Position) -> u32 {
    let mut sum = 0u32;
    for sq in 0..64u8 {
        let e = pos.mailbox[sq as usize];
        sum += e.armor() as u32;
    }
    sum
}

fn classify(pos: &Position, prev_phase: Phase) -> Option<Cat> {
    let r = pos.round_number;
    let phase = pos.current_phase;
    let pc = piece_count(pos);
    let combo_loaded = pos.tracked_casters_len > 0 || pos.tracked_enemies_len > 0;
    let armour = total_armour(pos);

    // Phase boundary — Move→Skill or Skill→Move transition.
    if phase != prev_phase {
        return Some(Cat::PhaseBoundary);
    }

    // Combo-loaded — any Skill phase with tracked entity present (rare).
    if combo_loaded && phase == Phase::Skill {
        return Some(Cat::ComboLoaded);
    }

    // Midgame-high-skill — any Skill phase with many pieces still on board.
    // (Modifiers and tracked entities are rare in random play; using high-pc
    // Skill plies as the proxy for "high branching density".)
    if phase == Phase::Skill && pc >= 16 {
        return Some(Cat::MidgameHighSkill);
    }

    if r <= 2 {
        return Some(Cat::Opening);
    }

    if r >= 6 && r <= 12 && phase == Phase::Move && pc >= 18 {
        return Some(Cat::MidgameLowSkill);
    }

    // Endgame-attrition — armour stacked, board still mostly full.
    if r >= 8 && armour >= 5 && pc >= 10 {
        return Some(Cat::EndgameAttrition);
    }

    // Endgame-sparse — late round, board thinned.
    if r >= 10 && pc <= 10 {
        return Some(Cat::EndgameSparse);
    }

    None
}

const SAMPLES_PER_CAT: usize = 6;

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
    eprintln!("# build_corpus: games={} seed=0x{:016X}", n_games, seed);

    let mut buckets: std::collections::HashMap<Cat, Vec<String>> =
        std::collections::HashMap::new();

    let mut diag_max_round = 0u16;
    let mut diag_skill_phase_plies = 0u64;
    let mut diag_skill_phase_pc16 = 0u64;
    let mut diag_combo_loaded = 0u64;
    let mut diag_armour_5 = 0u64;

    for g in 0..n_games {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(g as u64));
        let mut pos = Position::setup_stack_m();
        let mut prev_phase = pos.current_phase;
        let mut plies = 0usize;
        while plies < MAX_PLIES {
            if pos.game_result.is_some() {
                break;
            }
            let moves = generator::generate(&pos);
            if moves.is_empty() {
                break;
            }

            // Diagnostics — what does the random play distribution look like?
            if pos.round_number > diag_max_round { diag_max_round = pos.round_number; }
            if pos.current_phase == Phase::Skill {
                diag_skill_phase_plies += 1;
                let pc = piece_count(&pos);
                if pc >= 16 { diag_skill_phase_pc16 += 1; }
            }
            if pos.tracked_casters_len > 0 || pos.tracked_enemies_len > 0 {
                diag_combo_loaded += 1;
            }
            if total_armour(&pos) >= 5 { diag_armour_5 += 1; }

            if let Some(cat) = classify(&pos, prev_phase) {
                let bucket = buckets.entry(cat).or_insert_with(Vec::new);
                if bucket.len() < SAMPLES_PER_CAT * 4 {
                    // keep a few extras per cat; we'll subsample at print time
                    bucket.push(to_fen(&pos));
                }
            }
            prev_phase = pos.current_phase;
            // Bias random play AWAY from EndPhase/EndTurn when there is at least
            // one substantive alternative — otherwise the random walker rushes
            // through skill phases in a single ply and we never get mid-Skill
            // positions. Real engines won't pick EndPhase uniformly either.
            let active: Vec<_> = moves
                .iter()
                .copied()
                .filter(|a| a.kind() != ActionKind::EndPhase && a.kind() != ActionKind::EndTurn)
                .collect();
            let pool: &[_] = if active.is_empty() { &moves[..] } else { &active[..] };
            let pick = pool.choose(&mut rng).copied().unwrap();
            let _undo = make_unmake::make(&mut pos, pick);
            plies += 1;
        }
    }

    eprintln!("# diag: max_round={} skill_phase_plies={} skill_pc16={} combo_loaded_plies={} armour_5_plies={}",
        diag_max_round, diag_skill_phase_plies, diag_skill_phase_pc16, diag_combo_loaded, diag_armour_5);

    // Print: SAMPLES_PER_CAT rows per category, in stable order.
    let order = [
        Cat::Opening,
        Cat::MidgameLowSkill,
        Cat::MidgameHighSkill,
        Cat::PhaseBoundary,
        Cat::ComboLoaded,
        Cat::EndgameSparse,
        Cat::EndgameAttrition,
    ];
    println!("# Auto-generated corpus rows. Hand-curate before committing.");
    println!("# Format: id, category, expected_depth_n, expected_score_range, fen [; expected_best_move_raw,...]");
    for cat in order {
        let bucket = buckets.get(&cat).cloned().unwrap_or_default();
        let label = cat_str(cat);
        eprintln!("# {}: {} samples available", label, bucket.len());
        for (i, fen) in bucket.iter().take(SAMPLES_PER_CAT).enumerate() {
            println!("{}-{:02}, {}, -, -, {}", label, i + 1, label, fen);
        }
    }
}
