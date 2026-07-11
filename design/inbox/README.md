# Inbox — Fast-Write Staging

One place to dump raw material between sessions. This is the **fast-write** channel; the DB (`design/design.db`) is the **slow-write** channel. Don't optimise files here — just dump. At session start (or when you ask), I read everything here, promote the load-bearing material into the DB, and either delete the source file or annotate it with where it went.

**One folder, four kinds of drops.** Name files however you like; a leading tag keeps them sorted:

## 1. Game-design brainstorm — `brainstorm-*.md` (or any name)
Raw idea dumps: mechanics, skills, board, game-shape riffs. Bullets, prose, fragments — whatever you'd write in a notebook.
→ Promoted into `backpocket` entries, `open_questions`, or `essays`.
*Examples:* "what if Bodyguard auto-triggered on Strikes too?", a holiday riff on game shape, a worry you don't want to lose.

## 2. Pasted AI chats — `chat-*.md` (ChatGPT / Perplexity / Gemini)
One file per thread. Paste the conversation in markdown (`**User:**` / `**AI:**` is fine). An optional one-line context header helps me prioritise.
→ Promoted into `backpocket` (future mechanic/fix), `essays` (substantive research artefact — e.g. a Perplexity thread), OQ drafts, or `adrs`.
*Note:* the `essays` table is the canonical home for research; this is just staging. Conversations with me (Claude Code) already live in the session log + DB — don't paste those.

## 3. Digital-implementation notes — `digital-*.md`
Architecture sketches, UI ideas, AI-opponent thinking, save-format/transport musings — anything shaping `game/` that isn't a settled decision yet.
→ Promoted into `adrs`, `open_questions`, `next_steps`, or `essays`.
*Examples:* "AI opponent should feel like X not Y", frontend Desktop/Web/Mobile split, "must survive a browser refresh" (P5 lesson), threat-zone overlay for felt-PI (OQ-64).
*(The four planning docs currently in this folder — `nn-rater-plan`, `nn-trainer-cleanup`, `alpha-beta-optimisation-catalogue`, `search-speed-benchmark-plan` — are digital notes of this kind.)*

## 4. Playtest feedback — `playtest-<name>-notes.md`
Raw qualitative/feel notes from a playtest (the prose the game log can't capture). Drop the notes here as text; keep **binary artefacts (photos, scans, exported JSON logs) in `design/raw/playtest-photos/<name>/`.**
→ Promoted into a `playtests` row + an `essays` analysis (via `/playtest`).
*Convention:* notes here, binaries in `raw/`. *(Two legacy `notes.md` files still live inside their `raw/playtest-photos/<name>/` folders — left in place because DB/stack provenance points at them. New feedback follows the convention above.)*

---

## What does NOT belong here
- Anything already in the DB — don't restate; link or update the row.
- Settled decisions — those go straight into `adrs` / `mechanics` (don't stage them here).
- Actual code — that lives in `game/`.
- Audio/video — convert to text first, or drop binaries in `design/raw/`.
