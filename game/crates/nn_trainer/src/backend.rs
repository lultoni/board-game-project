//! Backend type aliases for the burn ML framework.
//!
//! Two always-available aliases pin the inference + (default) training backend
//! to CPU `ndarray`:
//! - `InferenceBackend` — `NdArray<f32>`. Used for forward-only paths
//!   (loading raters, running gauntlets, the NN evaluator inside search).
//!   Always CPU: a search call burns microseconds per position; GPU dispatch
//!   overhead at batch-size 1 would dominate, and enum-dispatching the
//!   evaluator through every search call site is a worse outcome than a
//!   cross-backend save→load at the acceptance boundary (which we already do
//!   via burn's `NamedMpkFileRecorder`, see `lineage_checkpoint`).
//! - `TrainingBackend` — `Autodiff<NdArray<f32>>`. The CPU training backend.
//!   Other training backends (wgpu / cuda) sit beside it as additional
//!   aliases gated on Cargo features.
//!
//! Burn pins the backend in its type system, so any given monomorphisation
//! of `run_training` / `train_lineages` / `train_step` is locked to one
//! backend. To support a runtime "CPU vs GPU" UI choice within a single
//! binary, the top-level `run_training` in `run.rs` matches `BackendChoice`
//! and dispatches into the right monomorphisation — see A2 of the plan.
//!
//! Persistence is backend-agnostic at the wire level: `save_rater` /
//! `load_rater` are generic over `B: Backend` and the `.mpk` recorder is
//! cross-backend. A model trained on Wgpu can be loaded back on NdArray for
//! inference. We rely on this in two places:
//! 1. `lineage_checkpoint` strips autograd via `.valid()` (autodiff backend
//!    → inner inference backend) and reloads into autodiff on resume.
//! 2. Accepted GPU-trained raters are written via the inference-backend
//!    side of whichever `Autodiff<...>` did the training and consumed by
//!    `NnEvaluator` on `InferenceBackend` (CPU).

use burn::backend::{Autodiff, NdArray};

/// The always-available CPU inference backend. All long-lived inference
/// paths (search-time `NnEvaluator`, gauntlet match play, the rater
/// inspector command) load through this.
pub type InferenceBackend = NdArray<f32>;

/// The always-available CPU training backend — autodiff layered on
/// `InferenceBackend`. `BackendChoice::Cpu` dispatches here.
pub type TrainingBackend = Autodiff<InferenceBackend>;

/// `wgpu` training backend — Metal on Mac, Vulkan on Linux, DX12 on
/// Windows. Wired into `run_training`'s dispatch via
/// `BackendChoice::Wgpu`.
#[cfg(feature = "backend-wgpu")]
pub type WgpuTrainingBackend = Autodiff<burn::backend::wgpu::Wgpu<f32, i32>>;

/// CUDA training backend — Linux + NVIDIA only; requires CUDA Toolkit
/// 12.x at build time. Wired into `run_training`'s dispatch via
/// `BackendChoice::Cuda`. The whole binary opts into the CUDA toolchain
/// when this feature is on; built as a separate release artefact.
#[cfg(feature = "backend-cuda")]
pub type CudaTrainingBackend = Autodiff<burn::backend::cuda::Cuda<f32, i32>>;

/// Default-device shim, kept for callers that don't care which backend
/// they're running on. Always returns the CPU device — the GPU backends
/// have their own `Default` device that the dispatcher in `run.rs`
/// constructs at the right monomorphisation.
pub fn default_device() -> burn::tensor::Device<InferenceBackend> {
    Default::default()
}

/// Runtime backend selector. Drives the dispatch in
/// `run::run_training`. The set of variants is fixed at compile time
/// (the enum is always whole), but `available()` returns only those
/// variants whose Cargo feature was enabled in this build — that's the
/// list the UI dropdown gets to choose from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendChoice {
    /// CPU `ndarray`. Always available.
    Cpu,
    /// `wgpu` (Metal / Vulkan / DX12). Available when the
    /// `backend-wgpu` Cargo feature is on; that's the default for the
    /// `default` release binary.
    Wgpu,
    /// NVIDIA CUDA. Available when the `backend-cuda` Cargo feature is
    /// on; only the dedicated CUDA release binary enables it.
    Cuda,
}

impl BackendChoice {
    /// The set of backends compiled into this binary, in
    /// preference-for-default order.
    pub fn available() -> Vec<BackendChoice> {
        let mut v = Vec::with_capacity(3);
        // GPU choices first so a binary that has them lists them above
        // the CPU fallback. UI is free to re-order.
        #[cfg(feature = "backend-cuda")]
        v.push(BackendChoice::Cuda);
        #[cfg(feature = "backend-wgpu")]
        v.push(BackendChoice::Wgpu);
        v.push(BackendChoice::Cpu);
        v
    }

    /// The recommended default for this binary — the first entry of
    /// `available()`. UIs persist the user's last choice and fall back
    /// to this when no previous selection exists.
    pub fn default_choice() -> BackendChoice {
        // Always at least one entry (Cpu) — unwrap is sound.
        Self::available().into_iter().next().unwrap()
    }

    /// Stable lowercase tag for serde / UI / logs. Matches the serde
    /// `rename_all = "lowercase"` output.
    pub fn as_str(self) -> &'static str {
        match self {
            BackendChoice::Cpu => "cpu",
            BackendChoice::Wgpu => "wgpu",
            BackendChoice::Cuda => "cuda",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_is_always_available() {
        assert!(BackendChoice::available().contains(&BackendChoice::Cpu));
    }

    #[test]
    fn default_choice_matches_first_available() {
        let avail = BackendChoice::available();
        assert_eq!(BackendChoice::default_choice(), avail[0]);
    }

    #[test]
    fn backend_choice_roundtrips_serde() {
        for c in [BackendChoice::Cpu, BackendChoice::Wgpu, BackendChoice::Cuda] {
            let s = serde_json::to_string(&c).unwrap();
            let back: BackendChoice = serde_json::from_str(&s).unwrap();
            assert_eq!(c, back);
            // Tag matches as_str().
            assert_eq!(s, format!("\"{}\"", c.as_str()));
        }
    }
}
