//! Backend type aliases for the burn ML framework.
//!
//! Two aliases:
//! - `InferenceBackend` — the concrete backend used for forward-only paths
//!   (loading raters, running gauntlets, the NN evaluator inside search).
//! - `TrainingBackend` — `Autodiff<InferenceBackend>`, used by anything that
//!   takes gradient steps (`train`, `train_lineages`, the orchestrator's
//!   per-generation lineages).
//!
//! Backend selection is a Cargo feature, not a runtime decision: burn pins
//! the backend in its type system, so a single binary can only host one
//! backend. The three features (`backend-ndarray`, `backend-wgpu`,
//! `backend-cuda`) are mutually exclusive — enabling two violates Cargo's
//! "features are additive" guideline in spirit, but a single `pub type`
//! cannot resolve to two concrete types. The `compile_error!` guards below
//! make the misuse loud rather than producing baffling type errors deeper
//! in the build.
//!
//! Persistence is backend-agnostic: `save_rater` / `load_rater` are generic
//! over `B: Backend`, so a model trained on Wgpu can be loaded back on
//! NdArray for inference inside the WASM build (which never sees the GPU
//! features).

#[cfg(all(feature = "backend-ndarray", feature = "backend-wgpu"))]
compile_error!(
    "nn_trainer: features `backend-ndarray` and `backend-wgpu` are mutually exclusive"
);
#[cfg(all(feature = "backend-ndarray", feature = "backend-cuda"))]
compile_error!(
    "nn_trainer: features `backend-ndarray` and `backend-cuda` are mutually exclusive"
);
#[cfg(all(feature = "backend-wgpu", feature = "backend-cuda"))]
compile_error!(
    "nn_trainer: features `backend-wgpu` and `backend-cuda` are mutually exclusive"
);
#[cfg(not(any(
    feature = "backend-ndarray",
    feature = "backend-wgpu",
    feature = "backend-cuda"
)))]
compile_error!(
    "nn_trainer: one of `backend-ndarray`, `backend-wgpu`, `backend-cuda` must be enabled"
);

#[cfg(feature = "backend-ndarray")]
mod inner {
    use burn::backend::{Autodiff, NdArray};
    pub type InferenceBackend = NdArray<f32>;
    pub type TrainingBackend = Autodiff<InferenceBackend>;
    pub fn default_device() -> burn::tensor::Device<InferenceBackend> {
        Default::default()
    }
}

#[cfg(feature = "backend-wgpu")]
mod inner {
    use burn::backend::{wgpu::Wgpu, Autodiff};
    pub type InferenceBackend = Wgpu<f32, i32>;
    pub type TrainingBackend = Autodiff<InferenceBackend>;
    pub fn default_device() -> burn::tensor::Device<InferenceBackend> {
        Default::default()
    }
}

#[cfg(feature = "backend-cuda")]
mod inner {
    use burn::backend::{cuda::Cuda, Autodiff};
    pub type InferenceBackend = Cuda<f32, i32>;
    pub type TrainingBackend = Autodiff<InferenceBackend>;
    pub fn default_device() -> burn::tensor::Device<InferenceBackend> {
        Default::default()
    }
}

pub use inner::{default_device, InferenceBackend, TrainingBackend};
