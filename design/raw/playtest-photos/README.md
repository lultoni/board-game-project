# Playtest Results

Raw photos and scans of handwritten feedback forms and tracking sheets, one folder per playtest session.

## Folder convention

`<player-A>-vs-<player-B>-<DD_MM_YY>/` — e.g. `elias-vs-jonathan-24_04_26/`.

Inside each folder: phone photos of feedback forms, tracking sheets, and any annotated rule sheets. Filenames are whatever the camera produced — analysis happens in the corresponding `docs/research/playtest-N-analysis.md`.

## Storage policy

Raw photos are committed to the repo for now. As of Session 17, this folder is ~117 MB.

If repo size becomes a problem (rough threshold: 250 MB or noticeable clone slowdown), the migration options in priority order:

1. **Git LFS** for `playtest-results/**` — keeps photos in-repo logically, but stores them out-of-band.
2. **Out-of-tree storage** (e.g. a synced folder, cloud bucket) with only the analysis `.md` files committed.
3. **Compress in place** — re-encode photos at lower resolution before commit.

No action required this session — just record the policy point. Re-evaluate if/when the repo crosses the threshold.

## What lives here vs. elsewhere

- **Raw photos / scans** — here.
- **Transcribed + analysed playtest** — `docs/research/playtest-N-analysis.md`.
- **Decisions resulting from the playtest** — `docs/mechanics-log/mechanics-evaluated.md` (cross-linked by Source OQ + Evidence file).
