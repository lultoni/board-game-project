# Inbox — Digital Implementation Notes

Drop notes about the **digital implementation** in `game/` here — architecture sketches, UI ideas, AI-opponent thinking, save-format musings, transport choices, anything that shapes the code in `game/` without yet being a settled decision.

## How to use

- **One file per topic.** Name it however you like: `2026-06-22-rust-frontend-split.md`, `ai-opponent-feel.md`, `save-format-thoughts.md`.
- **No structure required.** Bullets, prose, fragments, ASCII mockups — whatever helps you think.
- **Continuing an earlier thought?** Append to the existing file.

## What happens next

At session start (or when you ask), I'll:
1. Read every file in this folder.
2. Promote load-bearing material into the DB:
   - **`adrs`** — decisions with multiple viable options resolved here (e.g. "Tauri vs egui").
   - **`open_questions`** — unresolved architecture questions worth tracking.
   - **`next_steps`** — concrete implementation tasks for `game/`.
   - **`essays`** — substantive analyses (e.g. AI-opponent design write-up).
3. Either delete the source file (fully absorbed) or annotate it with what was promoted where.

## What fits here

- "I want the AI opponent to feel like X, not Y."
- A sketch of how the frontend split should work (Desktop / Web / Mobile).
- Save-format requirements (e.g. "must survive a browser refresh" — P5 lesson).
- Multiplayer transport thinking (relay-fallback P2P vs self-hosted server).
- UI-flow ideas: how the draft phase looks, how piece selection works, how the threat-zone overlay surfaces felt-PI (OQ-64).
- A bug or constraint you noticed while playing other digital tactics games and want to remember when implementing ours.

## What doesn't fit here

- **Game-design ideas** (mechanics, skills, board) → `design/inbox/brainstorm/`.
- **Pasted AI chats** (ChatGPT/Perplexity/Gemini transcripts) → `design/inbox/ai-chats/`.
- **Settled decisions** → straight into `adrs` (don't stage them here).
- **Actual code** → `game/` itself.

## Relation to `game/README.md`

`game/README.md` is the public-facing status page for the folder ("what's in here, what's open"). This inbox is the private scratchpad. Once `game/` has real code, an architecture overview will graduate from here into `adrs` and probably a `design_docs` row.
