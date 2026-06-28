# Contributing

Quick notes on working in this repo.

## Branches

- `main` is the integration branch; releases are cut by tagging
  `vX.Y.Z` and pushing the tag (CI builds + uploads to a draft Release).
- For non-trivial work, branch off `main`, open a PR, get a green build,
  squash-merge.

## Before opening a PR

```bash
cd game
cargo check                                                 # default features (ndarray + wgpu)
cargo test -p core_engine                                   # engine unit tests
cd frontend && npm run check && npm run test                # type-check + vitest
```

The full `nn_trainer` test suite is slow (~30 min on a Mac). Reserve it
for end-of-phase boundaries, not per-commit. `cargo check` between
commits is enough to catch type errors.

## Design knowledge

All design decisions live in `design/design.db` (SQLite). See
[`CLAUDE.md`](CLAUDE.md) for query patterns. Never restate facts in
markdown — link by row ID instead.

## Backends

The trainer compiles in one or more burn backends:

- `backend-ndarray` (CPU, always)
- `backend-wgpu` (GPU via Metal / Vulkan / DX12)
- `backend-cuda` (NVIDIA CUDA — Linux + CUDA 12.x at build time)

Defaults are `ndarray + wgpu`. The CUDA build is a separate release
artefact because `burn-cuda` requires the CUDA Toolkit at link time.
Runtime backend choice lives in the `/training` UI.

## Releases

```bash
git tag v0.1.0 && git push --tags
```

triggers `.github/workflows/release.yml`. Three jobs (macOS arm64,
Linux x86_64, Linux x86_64 + CUDA) build, bundle, and attach artefacts
to a draft Release for the tag. Publish the draft after smoke-testing
the downloads.
