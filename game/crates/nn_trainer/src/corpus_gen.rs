//! In-process training-corpus generation for the NNUE Phase-0 bootstrap.
//!
//! The Phase-0 bootstrap (`bootstrap.rs`) regresses the NNUE net to reproduce
//! the hand-crafted `evaluate`. That needs *many* realistic positions - the
//! search benchmark corpus (`bench/corpus/raw_corpus.txt`, ~120 curated rows)
//! is far too small and is a different artifact entirely (a hand-curated
//! 6-bucket benchmark, not a training set). This module generates a large,
//! diverse, deduped set of realistic non-terminal positions for training.
//!
//! Same *search-driven self-play* idea as `core_engine/examples/build_corpus.rs`
//! (both sides play depth-N alpha-beta, depths cycled 2/3/4 across games so
//! opening lines diverge), but tuned for **volume, not curation**: every
//! deduped non-terminal position visited is kept - no 6-bucket classification,
//! no per-bucket cap. Realistic loadouts come from the shared
//! `loadout::random_loadout` (ns-50 §5.3). Reproducible from a seed.
//!
//! Terminal positions are NOT emitted - they bypass the NN in-search (the
//! bootstrap labeller skips them anyway), so there's no point storing them.

use crate::loadout::random_loadout;
use core_engine::game_logic::{generator, make_unmake};
use core_engine::search::alpha_beta::find_best;
use core_engine::search::transposition::TranspositionTable;
use core_engine::state::fen::to_fen;
use core_engine::state::Position;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

/// Ply cap per game (same as the benchmark builder - long games terminate).
const MAX_PLIES: usize = 2_000;

/// Search depths cycled as the play policy - game `i` uses `PLAY_DEPTHS[i %
/// len]`. Different depths pick different opening moves, so games diverge
/// beyond just their loadouts (mirrors the benchmark builder).
const PLAY_DEPTHS: [u8; 3] = [2, 3, 4];

/// Generate up to `target_positions` deduped, realistic, non-terminal positions
/// by search-driven self-play over up to `n_games` games seeded from `seed`.
///
/// Dedup is two-layer (same as the benchmark builder):
/// - `zobrist` - fast exact-state dedup.
/// - view-key (board + STM + phase) - rejects positions differing only in
///   counter values (money / actions_remaining / round), which are worthless
///   as separate training rows.
///
/// Stops early once `target_positions` unique positions are collected. Returns
/// however many it found (may be fewer than the target if `n_games` runs out).
pub fn generate_training_corpus(
    target_positions: usize,
    n_games: usize,
    seed: u64,
) -> Vec<Position> {
    let mut out: Vec<Position> = Vec::with_capacity(target_positions.min(1 << 20));
    let mut seen_zobrist: HashSet<u64> = HashSet::new();
    let mut seen_view: HashSet<String> = HashSet::new();

    'games: for g in 0..n_games {
        if out.len() >= target_positions {
            break;
        }
        let mut rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(g as u64));
        let play_depth = PLAY_DEPTHS[g % PLAY_DEPTHS.len()];
        let p1_loadout = random_loadout(&mut rng);
        let p2_loadout = random_loadout(&mut rng);
        let mut pos = Position::setup_stack_m_with_loadouts(&p1_loadout, &p2_loadout);
        // Fresh TT per game - avoids cross-game contamination, bounds memory.
        let mut tt = TranspositionTable::with_capacity_mb(16);

        let mut plies = 0usize;
        while plies < MAX_PLIES {
            if pos.game_result.is_some() {
                break;
            }
            let moves = generator::generate(&pos);
            if moves.is_empty() {
                break;
            }

            // Record this (non-terminal) position if new.
            let fen = to_fen(&pos);
            let view_key: String = fen.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
            if seen_zobrist.insert(pos.zobrist) && seen_view.insert(view_key) {
                out.push(pos.clone());
                if out.len() >= target_positions {
                    break 'games;
                }
            }

            // Search-driven play: depth-N alpha-beta picks the move; fall back
            // to a random legal move if search returns none (defensive).
            let pick = {
                let res = find_best(&mut pos, &mut tt, 0, play_depth);
                res.best.unwrap_or_else(|| *moves.choose(&mut rng).unwrap())
            };
            let _undo = make_unmake::make(&mut pos, pick);
            plies += 1;
        }
    }

    out
}

/// Write a training corpus to `path`, one FEN per line (plus a `#` header).
/// `bootstrap::label_corpus` reads this back (it treats a comma-free line as a
/// bare FEN). Creates parent directories.
pub fn write_training_corpus_file(
    path: &std::path::Path,
    positions: &[Position],
) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut buf = String::with_capacity(positions.len() * 96 + 128);
    buf.push_str("# NN training corpus - auto-generated (ns-50). One FEN per line.\n");
    buf.push_str("# Regenerated on demand by nn_trainer::corpus_gen; gitignored. NOT the search benchmark.\n");
    for pos in positions {
        buf.push_str(&to_fen(pos));
        buf.push('\n');
    }
    let mut f = std::fs::File::create(path)?;
    f.write_all(buf.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_deduped_nonterminal_positions() {
        // Small target so the test is fast. Assert the core contract:
        // non-empty, all non-terminal, no duplicate zobrist keys.
        let corpus = generate_training_corpus(300, 20, 0xC0FFEE);
        assert!(!corpus.is_empty(), "should produce positions");
        let mut seen = HashSet::new();
        for p in &corpus {
            assert!(p.game_result.is_none(), "terminals must not be emitted");
            assert!(seen.insert(p.zobrist), "positions must be deduped by zobrist");
        }
    }

    #[test]
    fn is_deterministic_from_seed() {
        let a = generate_training_corpus(150, 10, 42);
        let b = generate_training_corpus(150, 10, 42);
        assert_eq!(a.len(), b.len(), "same seed → same count");
        // FEN equality is the observable determinism signal.
        for (pa, pb) in a.iter().zip(b.iter()) {
            assert_eq!(to_fen(pa), to_fen(pb), "same seed → same positions in order");
        }
    }

    #[test]
    fn respects_target_cap() {
        // With many games available, the target caps the output.
        let corpus = generate_training_corpus(50, 100, 7);
        assert!(corpus.len() <= 50, "must not exceed target; got {}", corpus.len());
        assert!(!corpus.is_empty());
    }

    #[test]
    fn write_and_reparse_roundtrips() {
        use crate::bootstrap::label_corpus;
        let corpus = generate_training_corpus(80, 10, 123);
        let dir = std::env::temp_dir().join(format!("nn_corpus_gen_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("train.txt");
        write_training_corpus_file(&path, &corpus).expect("write");
        // label_corpus reads FEN-per-line + labels via evaluate.
        let text = std::fs::read_to_string(&path).unwrap();
        let labelled = label_corpus(&text);
        assert!(!labelled.is_empty(), "reparsed corpus must yield labelled positions");
        for ex in &labelled {
            assert!(ex.position.game_result.is_none());
            assert!(ex.label_cp.is_finite());
        }
    }
}
