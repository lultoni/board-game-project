# Inbox — AI Chats

Drop chat transcripts from other AI systems (ChatGPT, Gemini, Perplexity threads, etc.) here.

## How to use

- **One file per chat thread.** Name it descriptively: `chatgpt-2026-06-22-skill-rebalance.md`, `perplexity-board-game-economy.md`.
- **Format**: paste the conversation in markdown. Use `**User:**` / `**AI:**` or `## User` / `## AI` — anything readable. Don't sanitize.
- **Optional context line at the top**: one sentence on what you were trying to figure out, so I can prioritise which threads to mine.

## What happens next

When you ask me to "review the inbox" (or at session start), I'll:
1. Read new files.
2. Pull out the load-bearing insights and stage them as:
   - **`backpocket`** entries if they suggest a future mechanic / fix.
   - **`essays`** rows if the chat itself is a substantive research artefact (e.g. a Perplexity research thread).
   - **OQ** drafts if the chat raises an unresolved question.
   - **`adrs`** drafts if the chat resolved a multi-option decision.
3. Either delete the source file (fully absorbed) or annotate it with what was promoted where.

## Note on duplication with `docs/research/`

The legacy Perplexity workflow stored research threads in `docs/research/*.md` and those are already migrated into the `essays` table. Going forward, fresh AI chats land here. The `essays` table is the single canonical home — `inbox/ai-chats/` is just the staging area.

## Examples of what fits here

- A ChatGPT brainstorm about a mechanic you're stuck on.
- A Perplexity research thread on a comparable game.
- A Gemini conversation that gave you a useful framing.

## What doesn't fit here

- Conversations with me (Claude Code) — those already live in the session log + DB.
- Audio/video — convert to text first or drop in `design/raw/` instead.
