//! Phase-0 supervised bootstrap: regress the sparse NNUE net to reproduce the
//! hand-crafted `evaluate` (centipawns), then quantize and grade.
//!
//! See `design/inbox/nnue-rework-plan.md` §2, §4. This is the de-risking slice:
//! it proves the full pipeline (sparse encode → dense-scatter train → quantize
//! → accumulator refresh → integer forward → centipawns) end-to-end on a target
//! with a KNOWN correct answer, before any self-play. Labels are cheap
//! (`evaluate` is ~260 ns), so this needs no self-play games.
//!
//! Bootstrap uses **gradient descent** (the right tool for a known target);
//! self-play refinement (Phase 1) switches to mutation.

use burn::optim::AdamConfig;

use crate::backend::{InferenceBackend, TrainingBackend};
use crate::model::{Mlp, MlpConfig};
use crate::quantized::{QuantScales, QuantizedNet, QA, QW};
use crate::sparse::NUM_FEATURES;
use crate::train::{into_inference, sparse_batch_to_tensors, train_step, ScalarLabelled, TrainingConfig};

use core_engine::search::evaluator::evaluate;
use core_engine::state::fen::from_fen;

/// Divisor mapping centipawn labels into the ~[-1,1] regime the optimizer likes.
/// Folded into the quantized net's output scale so `forward_int` returns
/// centipawns directly. Typical non-terminal `evaluate` magnitudes are a few
/// thousand cp; /1000 keeps targets O(1) without saturating.
pub const LABEL_DIVISOR: f32 = 1000.0;

/// Parse a corpus text file (FEN is the last comma-separated field; `#` comment
/// lines and blanks are skipped) into positions labelled with the hand-crafted
/// `evaluate`. Terminal positions are skipped - they bypass the NN in-search.
pub fn label_corpus(corpus_text: &str) -> Vec<ScalarLabelled> {
    let mut out = Vec::new();
    for line in corpus_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fen = line.split(',').next_back().map(str::trim).unwrap_or("");
        let Ok(pos) = from_fen(fen) else { continue };
        if pos.game_result.is_some() {
            continue; // terminals bypass the NN
        }
        let label_cp = evaluate(&pos) as f32;
        out.push(ScalarLabelled { position: pos, label_cp });
    }
    out
}

/// Train the sparse-input net (input_dim == NUM_FEATURES) to regress the
/// centipawn labels via MSE. Returns the trained inference-backend model + the
/// per-epoch mean loss (in normalized-label units²).
pub fn train_scalar(
    corpus: &[ScalarLabelled],
    config: &TrainingConfig,
) -> (Mlp<InferenceBackend>, Vec<f32>) {
    let device = Default::default();
    let cfg = MlpConfig::new().with_input_dim(NUM_FEATURES);
    let mut model: Mlp<TrainingBackend> = cfg.init(&device);
    let mut optimizer = AdamConfig::new().init();
    let mut epoch_losses = Vec::with_capacity(config.epochs);

    for _epoch in 0..config.epochs {
        let mut sum = 0.0f32;
        let mut n_batches = 0usize;
        for chunk in corpus.chunks(config.batch_size) {
            if chunk.is_empty() {
                continue;
            }
            let (inputs, labels) =
                sparse_batch_to_tensors::<TrainingBackend>(chunk, LABEL_DIVISOR, &device);
            let (next, loss) =
                train_step(model, &mut optimizer, inputs, labels, config.learning_rate);
            model = next;
            sum += loss;
            n_batches += 1;
        }
        epoch_losses.push(if n_batches > 0 { sum / n_batches as f32 } else { f32::NAN });
    }

    (into_inference(model), epoch_losses)
}

/// Full bootstrap: train on `corpus`, quantize, return the quantized net. The
/// output scale folds in `LABEL_DIVISOR` so `forward_int` yields centipawns.
pub fn bootstrap(corpus: &[ScalarLabelled], config: &TrainingConfig) -> QuantizedNet {
    let (model, _losses) = train_scalar(corpus, config);
    let scales = QuantScales { qa: QA, qw: QW, out: LABEL_DIVISOR };
    QuantizedNet::from_mlp(&model, scales)
}

/// Mean absolute error (centipawns) between the quantized net and the
/// hand-crafted `evaluate` across `corpus`.
pub fn mean_abs_error_cp(net: &QuantizedNet, corpus: &[ScalarLabelled]) -> f64 {
    use crate::accumulator::Accumulator;
    if corpus.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f64;
    for ex in corpus {
        let pred = net.forward_int(&Accumulator::refresh(&ex.position, net.ft())) as f64;
        sum += (pred - ex.label_cp as f64).abs();
    }
    sum / corpus.len() as f64
}

/// Convenience: label the repo's raw corpus (search-driven realistic positions).
pub fn label_repo_raw_corpus() -> Vec<ScalarLabelled> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/corpus/raw_corpus.txt");
    match std::fs::read_to_string(path) {
        Ok(text) => label_corpus(&text),
        Err(_) => Vec::new(),
    }
}

/// Path to the large NN **training** corpus - DISTINCT from the search
/// benchmark corpora (`raw_corpus.txt` / `corpus.txt`). Generated on demand,
/// gitignored, one FEN per line. ~100k positions at the default target.
pub const TRAINING_CORPUS_FILENAME: &str = "nn_training_corpus.txt";

/// Default training-corpus target: enough labelled positions for the Phase-0
/// bootstrap to regress `evaluate` into a genuinely heuristic-like net (the
/// ~120-row benchmark is far too small - see ns-50 Phase-1 findings).
pub const TRAINING_CORPUS_TARGET: usize = 100_000;

/// Ceiling on games played to reach the target. Generation stops as soon as
/// the target unique positions are collected; this only bounds the worst case.
pub const TRAINING_CORPUS_MAX_GAMES: usize = 6_000;

/// Deterministic seed for on-demand training-corpus generation.
pub const TRAINING_CORPUS_SEED: u64 = 0x5EED_C0DE_1234_5678;

/// Absolute path to the training corpus file (sibling of the benchmark files).
pub fn training_corpus_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/corpus")
        .join(TRAINING_CORPUS_FILENAME)
}

/// Label the large NN training corpus, generating it on demand if the file is
/// missing. If present, parse + label the existing file (reproducible across
/// runs); if absent, generate `target` positions via `corpus_gen`, write the
/// file (gitignored), then label. Labels are the hand-crafted `evaluate`
/// (cheap); terminals are skipped.
///
/// `path` / `target` / `max_games` / `seed` parameterize the file location and
/// on-demand generation so tests can point at a temp path and request a tiny
/// corpus. Production callers use [`label_training_corpus`] (default path +
/// `TRAINING_CORPUS_TARGET`).
pub fn label_training_corpus_with(
    path: &std::path::Path,
    target: usize,
    max_games: usize,
    seed: u64,
) -> Vec<ScalarLabelled> {
    if let Ok(text) = std::fs::read_to_string(path) {
        let labelled = label_corpus(&text);
        if !labelled.is_empty() {
            return labelled;
        }
        // Present-but-empty (or all-terminal) file → fall through to regenerate.
    }
    eprintln!(
        "[bootstrap] training corpus missing at {} - generating {} positions (up to {} games)…",
        path.display(), target, max_games,
    );
    let positions = crate::corpus_gen::generate_training_corpus(target, max_games, seed);
    eprintln!("[bootstrap] generated {} positions; writing corpus file", positions.len());
    if let Err(e) = crate::corpus_gen::write_training_corpus_file(path, &positions) {
        eprintln!("[bootstrap] WARNING: failed to write training corpus ({e}); labelling in memory");
    }
    // Label the in-memory positions directly (avoids a re-read; identical result).
    positions
        .into_iter()
        .filter(|p| p.game_result.is_none())
        .map(|position| {
            let label_cp = evaluate(&position) as f32;
            ScalarLabelled { position, label_cp }
        })
        .collect()
}

/// Production entry point: label the training corpus at the default path +
/// target (`TRAINING_CORPUS_TARGET` positions), generating on demand if missing.
pub fn label_training_corpus() -> Vec<ScalarLabelled> {
    label_training_corpus_with(
        &training_corpus_path(),
        TRAINING_CORPUS_TARGET,
        TRAINING_CORPUS_MAX_GAMES,
        TRAINING_CORPUS_SEED,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_corpus_skips_comments_and_terminals() {
        let corpus = label_repo_raw_corpus();
        assert!(!corpus.is_empty(), "raw corpus should yield labelled positions");
        for ex in &corpus {
            assert!(ex.position.game_result.is_none(), "terminals must be skipped");
            assert!(ex.label_cp.is_finite());
        }
    }

    /// Milestone accuracy test: after a short gradient bootstrap, the quantized
    /// net must reproduce `evaluate` within a small band on held-out positions,
    /// and materially beat a constant (mean) predictor baseline.
    ///
    /// `#[ignore]` - the 300-epoch train on the raw corpus takes several
    /// minutes (dense-scatter of NUM_FEATURES through burn per example). This is
    /// the Phase-0 milestone gate, run explicitly:
    /// `cargo test -p nn_trainer --release quantized_net_reproduces -- --ignored --nocapture`.
    #[test]
    #[ignore = "slow (minutes): Phase-0 milestone gate, run explicitly"]
    fn quantized_net_reproduces_evaluate_scalar_within_band() {
        let mut all = label_repo_raw_corpus();
        assert!(all.len() >= 20, "need a corpus; got {}", all.len());

        // Deterministic split: every 4th position held out.
        let mut train = Vec::new();
        let mut held = Vec::new();
        for (i, ex) in all.drain(..).enumerate() {
            if i % 4 == 0 {
                held.push(ex);
            } else {
                train.push(ex);
            }
        }

        let config = TrainingConfig { learning_rate: 1e-3, batch_size: 16, epochs: 300 };
        let net = bootstrap(&train, &config);

        let mae = mean_abs_error_cp(&net, &held);

        // Constant-predictor baseline: predict the train-set mean label.
        let mean_label: f64 =
            train.iter().map(|e| e.label_cp as f64).sum::<f64>() / train.len() as f64;
        let baseline: f64 = held
            .iter()
            .map(|e| (mean_label - e.label_cp as f64).abs())
            .sum::<f64>()
            / held.len() as f64;

        eprintln!("held-out MAE = {mae:.1} cp; constant-predictor baseline = {baseline:.1} cp");

        // The net must beat the trivial baseline. (A tight absolute cp band is
        // set once real training volume is available; on the ~130-position raw
        // corpus, "materially better than constant" is the honest gate.)
        assert!(
            mae < baseline * 0.75,
            "quantized net MAE {mae:.1} cp did not beat 0.75x baseline {baseline:.1} cp"
        );
        assert!(mae.is_finite());
    }

    /// Bigger-corpus milestone (ns-50 Phase-1 follow-up): the whole point of the
    /// large in-process training corpus is that the bootstrap net gets much
    /// closer to `evaluate` than the ~116-position benchmark allowed. Generate a
    /// moderate training corpus, train, and print held-out MAE vs the constant
    /// baseline. `#[ignore]` - generation + train take minutes. Run explicitly:
    /// `cargo test -p nn_trainer --release bigger_corpus_bootstrap -- --ignored --nocapture`.
    #[test]
    #[ignore = "slow (minutes): bigger-corpus bootstrap accuracy diagnostic"]
    fn bigger_corpus_bootstrap_beats_baseline() {
        // Generate ~15k positions in-process (deterministic), label with evaluate.
        let positions = crate::corpus_gen::generate_training_corpus(15_000, 2_000, 0x1234_5678);
        eprintln!("generated {} training positions", positions.len());
        let mut all: Vec<ScalarLabelled> = positions
            .into_iter()
            .filter(|p| p.game_result.is_none())
            .map(|position| {
                let label_cp = evaluate(&position) as f32;
                ScalarLabelled { position, label_cp }
            })
            .collect();
        assert!(all.len() >= 1_000, "need a sizable corpus; got {}", all.len());

        // Deterministic split: every 5th held out.
        let mut train = Vec::new();
        let mut held = Vec::new();
        for (i, ex) in all.drain(..).enumerate() {
            if i % 5 == 0 { held.push(ex); } else { train.push(ex); }
        }
        eprintln!("train={} held={}", train.len(), held.len());

        let config = TrainingConfig { learning_rate: 1e-3, batch_size: 64, epochs: 200 };
        let net = bootstrap(&train, &config);
        let mae = mean_abs_error_cp(&net, &held);

        let mean_label: f64 =
            train.iter().map(|e| e.label_cp as f64).sum::<f64>() / train.len() as f64;
        let baseline: f64 = held
            .iter()
            .map(|e| (mean_label - e.label_cp as f64).abs())
            .sum::<f64>()
            / held.len() as f64;

        eprintln!("BIGGER CORPUS: held-out MAE = {mae:.1} cp; constant-predictor baseline = {baseline:.1} cp (ratio {:.2})", mae / baseline);
        assert!(mae.is_finite());
        assert!(mae < baseline, "net must at least beat the constant baseline");
    }
}
