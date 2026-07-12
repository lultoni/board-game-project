//! NN position-rater trainer.
//!
//! Native-only (no WASM). Owns self-play data generation, gradient training,
//! the perturbation-injection layer, and the two-tier gauntlet. The trained
//! weights blob is consumed by a future `core_engine::search::evaluator::
//! NnEvaluator` impl (the trait seam shipped in session-37).
//!
//! Architecture: see `design/inbox/digital/nn-rater-plan.md`.
//! Autograd choice: see ADR-007 (burn with the ndarray backend).
//!
//! Current sub-slice 4a: input encoding + MLP topology + forward pass.
//! Training loop, self-play, gauntlet are not yet implemented.

pub mod backend;
pub mod batch;
pub mod accumulator;
pub mod bootstrap;
pub mod calibration;
pub mod corpus_gen;
pub mod encoding;
pub mod gauntlet;
pub mod lineage;
pub mod lineage_checkpoint;
pub mod live;
pub mod loadout;
pub mod matrix;
pub mod model;
pub mod nn_evaluator;
pub mod nnue_evaluator;
pub mod persistence;
pub mod quantized;
pub mod registry;
pub mod run;
pub mod selfplay;
pub mod snapshot;
pub mod sparse;
pub mod train;

pub use encoding::{encode_position, INPUT_DIM};
pub use sparse::{encode_sparse, encode_sparse_vec, ACCUM_WIDTH, NUM_FEATURES};
pub use accumulator::{Accumulator, FeatureTransform};
pub use quantized::{QuantScales, QuantizedNet};
pub use bootstrap::{bootstrap, label_corpus, mean_abs_error_cp, train_scalar, LABEL_DIVISOR};
pub use nnue_evaluator::NnueEvaluator;
pub use backend::BackendChoice;
pub use gauntlet::{
    accept_vs, mirrored_bo3, play_match, play_match_with_callback,
    Acceptance, ChampionTracker, MatchOutcome, RaterId, SeriesTally,
};
pub use lineage::{perturb_model, train_lineages, Lineage, LineageConfig};
pub use live::{
    is_subscribed, read_live, subscribe, unsubscribe, write_if_subscribed,
    EvalBars, LiveError, LivePosition, LIVE_POSITION_VERSION,
    LIVE_STATE_FILENAME, LIVE_SUBSCRIBE_FILENAME,
};
pub use model::{LayerStats, Mlp, MlpConfig};
pub use nn_evaluator::{InferenceBackend, NnEvaluator, DEFAULT_EVAL_SCALE, MAX_NN_SCORE};
pub use persistence::{
    load_rater, paths_from_stem, save_rater, BracketWinRate, PerturbationEvent,
    PersistenceError, RaterMetadata, TrainingConfigSnapshot, RATER_FORMAT_VERSION,
};
pub use registry::{IndexEntry, IndexError, RaterIndex, Track, INDEX_FORMAT_VERSION};
pub use run::{run_training, RunConfig, RunError, RunSummary};
pub use selfplay::{play_game, GameRecord, LabelledPosition};
pub use snapshot::{
    read_snapshot, write_snapshot, ActiveMatch, PopulationMember, SnapshotError,
    StatusSnapshot, TrainingPhase, STATUS_FILENAME, STATUS_SNAPSHOT_VERSION,
};
pub use batch::generate_corpus;
pub use loadout::{random_loadout, random_loadout_from_seed};
pub use corpus_gen::{generate_training_corpus, write_training_corpus_file};
pub use matrix::{
    load_matrix, save_matrix, GauntletMatrix, MatrixEntry, MatrixError,
    MATRIX_FILENAME, MATRIX_FORMAT_VERSION,
};
pub use train::{batch_to_tensors, into_inference, train, train_step, TrainingConfig};

#[cfg(test)]
mod integration_tests {
    //! Cross-module smoke test: encode setup_stack_m through the v1 model
    //! and assert the pipeline produces a finite scalar. Catches dim
    //! mismatches between the encoder and the topology default.

    use super::*;
    use crate::backend::InferenceBackend as B;
    use burn::tensor::{Tensor, TensorData};
    use core_engine::state::Position;

    #[test]
    fn end_to_end_forward_on_stack_m_start_position() {
        let device = Default::default();
        let cfg = MlpConfig::new();
        let model: Mlp<B> = cfg.init(&device);

        let pos = Position::setup_stack_m();
        let features = encode_position(&pos);
        assert_eq!(features.len(), INPUT_DIM);

        let data = TensorData::new(features, [1, INPUT_DIM]);
        let input: Tensor<B, 2> = Tensor::from_data(data, &device);
        let out = model.forward(input);
        let v: Vec<f32> = out.into_data().to_vec().unwrap();
        assert_eq!(v.len(), 1);
        assert!(v[0].is_finite(), "rater output was non-finite: {}", v[0]);
    }
}
