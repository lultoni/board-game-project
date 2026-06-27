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

pub mod batch;
pub mod encoding;
pub mod gauntlet;
pub mod lineage;
pub mod loadout;
pub mod model;
pub mod persistence;
pub mod registry;
pub mod selfplay;
pub mod train;

pub use encoding::{encode_position, INPUT_DIM};
pub use gauntlet::{
    mirrored_bo3, play_match, tier1_fitness, tier2_acceptance,
    AcceptanceReport, Bracket, BracketResults, ChampionTracker,
    MatchOutcome, RaterId, SeriesTally, TrackUpdate,
};
pub use lineage::{perturb_model, train_lineages, Lineage, LineageConfig};
pub use model::{Mlp, MlpConfig};
pub use persistence::{
    load_rater, paths_from_stem, save_rater, BracketWinRate, PerturbationEvent,
    PersistenceError, RaterMetadata, TrainingConfigSnapshot, RATER_FORMAT_VERSION,
};
pub use registry::{IndexEntry, IndexError, RaterIndex, Track, INDEX_FORMAT_VERSION};
pub use selfplay::{play_game, GameRecord, LabelledPosition};
pub use batch::generate_corpus;
pub use loadout::{random_loadout, random_loadout_from_seed};
pub use train::{batch_to_tensors, into_inference, train, train_step, TrainingConfig};

#[cfg(test)]
mod integration_tests {
    //! Cross-module smoke test: encode setup_stack_m through the v1 model
    //! and assert the pipeline produces a finite scalar. Catches dim
    //! mismatches between the encoder and the topology default.

    use super::*;
    use burn::backend::NdArray;
    use burn::tensor::{Tensor, TensorData};
    use core_engine::state::Position;

    type B = NdArray<f32>;

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
