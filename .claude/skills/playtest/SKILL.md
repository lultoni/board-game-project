---
name: playtest
description: "Analyse playtest results from photos/scans in playtest-results/. Transcribes handwritten feedback, game logs, and synthesises findings. Auto-triggers when user mentions playtest results or references playtest-results/ files."
argument-hint: "<playtest-number>"
---

# Playtest Analysis: Playtest $ARGUMENTS

## Step 1: Read All Playtest Materials

Read ALL files in the relevant playtest subfolder — images AND any .md side-notes files. Do not skip any file.

Also read the test scenario rule sheet that was used (check `docs/test-scenarios/` for the relevant layer) to understand what was being tested and what to watch for.

Also read `game-state/OPEN_QUESTIONS.md` — specifically any OQ marked **TRACKING** or with **Evaluation criteria** notes for this stack. These are the monitoring hypotheses you must answer from the data. (OQ numbers are not hardcoded in this skill — read the live file to find which ones apply.)

## Step 2: Transcribe

For each handwritten document, transcribe with maximum fidelity before interpreting. Errors in transcription cascade into wrong design conclusions.

### Game logs

- Reconstruct **every round row** — Runes (start+gain−spent=end), skills used, events/notes.
- Preserve the **exact wording** of all margin notes, exclamations, and annotations — these are named moments and design signals (e.g. "peak move!", "gradients", "stalling — very nice!").
- Note the **final round played** explicitly — the last row with an entry is the end of the game.
- Note any **post-game annotations** separately (written after the game ended, not during a round).
- Flag hard-to-read entries with [unclear] rather than guessing.

### Feedback forms

Transcribe each question answer **exactly as written** before interpreting. Common misread patterns to watch for:

- **Circle position**: Read which number on a scale is actually circled — don't estimate from nearby text.
- **"gut so" vs neutral**: "gut so" or "good like that" is an explicit design affirmation, not just a description. Flag it.
- **Soft flags vs hard flags**: "a bit — aber für jetzt okay — note for later" is a deferred soft flag, not an urgent problem. Preserve this nuance.
- **Crossed-out / blank questions**: If a player crossed out a question or left it blank, record that explicitly — don't infer an answer. Common case: a player who didn't play Playtest 1 cannot answer "Opening vs Playtest 1."
- **"?" answers**: A "?" means the player was uncertain — do not convert this into a yes/no or a count.
- **German handwriting**: Read carefully before translating. Common errors: "Schild" (shield) ≠ "Skill"; "limiterit" ≠ "Bremse" (both mean "limited/brake" but the nuance differs); "start" ≠ "jetzt".
- **Retaliation notes on expensive skills**: A "yes, reachable" answer with a note about retaliation risk is a richer insight than just "yes" — preserve both parts.

### Side-notes files

Side-notes (.md files in the playtest folder) are written by the game runner during or after the game. Treat every bullet point as a distinct design signal. Go through them exhaustively:

- Rules clarifications made mid-game → new OQ entries or ruleset fixes
- Skill-specific observations → skill balance flags
- New ideas or variants → new OQ entries (deferred)
- Layout/print notes → Typst/PDF fix items
- Table/progression corrections → rule sheet fix items
- Emotional or social observations → design signal notes

## Step 2.5: Structured Data Extraction

For each player's tracking sheet and game log, produce two blocks: (A) Rune/Capture data, (B) Behavioral pattern analysis.

### Block A — Tracking Data

```markdown
### Tracking Data — [Player name]

**Rune economy**
- Starting Runes: [value]
- First round a skill was used: Round [N]
- Rounds where unspent Runes exceeded 6: [list rounds, or "none"]
- Rounds where all Runes were spent: [list rounds, or "none"]
- Largest single-turn Rune spend: [N Runes on Round X]

**Captures & key events** (from Events/Notes column)
| Round | Event |
|-------|-------|
| [N] | [description] |
- Final round played: Round [N]
- Total captures made: [N]
- Total captures suffered: [N]
- Post-game annotations: [any notes written after the final round row]
```

### Block B — Behavioral Pattern Analysis

This block answers "what were players actually doing?" — extracted from the game log, not the feedback form. The goal is to surface patterns players don't consciously notice or report.

```markdown
### Behavioral Patterns — [Player name]

**Skill usage frequency**
| Skill | Times used | Rounds used | Typical context |
|-------|-----------|-------------|-----------------|
| [name] | [N] | [R1, R4, R7...] | [e.g. "finisher on Injured piece", "opener every turn", "paired with Blade Call"] |

- Most-used skill: [name] ([N] uses)
- Least-used / never used: [names]
- Skills used 3+ times: [list] — potentially over-relied upon
- Skills used exactly once or twice: [list] — possibly situational or underpowered

**Attack vs. skill balance**
- Standard attacks made: [N total, estimated from log]
- Skill activations made: [N total]
- Attack-to-skill ratio: [N attacks per skill use] — compare against Playtest baseline (P1: high, P2: mid)

**Combo attempts (if Layer 2+)**
- Multi-Champion combos attempted: [N]
- Combos that succeeded: [N]
- Which skills were paired: [list skill pairs]
- What blocked failed attempts: [LoS, no second Champion in range, etc.]

**Positioning and movement patterns** (inferred from event log)
- Rounds before first forward movement: [N]
- First contact round: [N] (first melee exchange or skill hit on an opponent)
- Evidence of standoff: [yes/no — describe if yes]
- Guard behavior: [used as screens / used offensively / died early / survived long]

**Armor usage** (if tracked)
- Total armor granted (you / opponent): [N / N]
- Rounds with active armor: [list]
- Armor vs. damage ratio: [approx — did armor absorb a meaningful share of incoming damage?]
```

**Synthesis note**: After completing both players' Block B, write 3–5 cross-player observations comparing their patterns. Look for: one player offensively dominant while other defends; both players stuck in standoff simultaneously; skill choices that mirror vs. diverge; Rune spending rhythms that synchronise or oppose.

## Step 2.8: Independent Per-Player Transcription (Multi-Agent)

Before synthesising, spawn two independent agents — one per player — to transcribe and extract behavioral patterns from that player's materials only. This prevents each player's data from biasing interpretation of the other's.

**How to do this:**

Spawn two agents in parallel (single message, two Agent tool calls):

- **Agent 1** receives: Player A's game log image(s) + feedback form image(s) + instructions to produce Block A (Tracking Data) and Block B (Behavioral Patterns) for Player A only. The agent must NOT read Player B's materials.
- **Agent 2** receives: Player B's game log image(s) + feedback form image(s) + instructions to produce Block A and Block B for Player B only. The agent must NOT read Player A's materials.

Each agent prompt must include:
1. The exact file paths to read (player-specific only).
2. The Block A and Block B templates from Step 2.5.
3. The instruction: "Do not read the other player's files. Produce only this player's data blocks. Do not try to compare with or anticipate the other player's results."

After both agents complete, collect their outputs and proceed to Step 3 synthesis — only at that point are both datasets combined for cross-player comparison.

**Why this matters**: If you read both players' materials sequentially, earlier data shapes interpretation of later data. A surprising skill usage pattern in Player A's log will prime you to look for (and find) it in Player B's — even if it isn't there. Independent extraction keeps the behavioral signal clean.

## Step 3: Synthesise Findings

Create `docs/research/playtest-N-analysis.md` with:

```markdown
# Playtest N Analysis

**Date**: [date]
**Players**: [names]
**Variants/Layer tested**: [what rule sheet was used]
**Game length**: [exact final round] / [session time if noted]

## Tracking Data
[Block A data — one per player]

## Behavioral Patterns
[Block B data — one per player, then cross-player synthesis]

## Raw Transcriptions
[All transcribed feedback and logs, with exact quotes preserved]

## Key Findings
[Numbered list, ordered by impact. Each finding must cite the specific source — quote or round number. Distinguish: confirmed positives, confirmed problems, soft flags (deferred), and open questions.
IMPORTANT: Separate findings into two categories:
  - **Self-reported** (player said X in the feedback form or verbally)
  - **Behaviorally observed** (extracted from the game log — the player did X but may not have noticed or named it)]

## Answers to Test Scenario Questions
[Go through the post-game questions from the test scenario rule sheet and answer each based on evidence. Where players disagreed, note both answers.]

## Implications for Design
[What should change, what was confirmed, what needs more testing. Separate: urgent actions, deferred items, design ideas.]

## Comparison to Previous Playtests
[How do these results compare to prior playtests — use a table]
```

## Step 3.5: OQ Metric Evaluation

After writing the synthesis, go through **every OQ that is TRACKING or has Evaluation criteria for this layer** in `game-state/OPEN_QUESTIONS.md`. For each one, write a short verdict block:

```markdown
### OQ-[N]: [Title]

**Hypothesis**: [What was expected]
**Evidence**: [Specific data points from this playtest — quotes, round numbers, counts]
**Verdict**: Confirmed / Partially confirmed / Inconclusive / Contradicted
**Recommended action**: [Update OQ status / Accept / Defer / Open new OQ / No change]
```

Include this section in the analysis doc under the heading `## OQ Evaluations`.

Cover at minimum:
- Every OQ explicitly marked **TRACKING (Layer N)** for this playtest
- Every OQ with **Evaluation criteria (Session X)** that maps to this layer
- Any OQ whose status should change based on the new data, even if not explicitly flagged

## Step 4: Cascade Updates

After writing the analysis, update the following living documents:

### `docs/systems-and-mechanics.md`
- Update Playtest Evidence section with new findings.
- Update Design Health Check scores if evidence warrants.
- Update Incremental Test Plan table (mark stack as tested, note result).

### `game-state/OPEN_QUESTIONS.md`
- Resolve any questions answered by this playtest.
- Add new questions raised by findings — one OQ per distinct issue.
- Update existing OQs with new evidence even if not resolved.
- Update status of all OQs covered in the Step 3.5 verdicts.

### `game-state/NEXT_STEPS.md`
- Update priorities based on findings.
- Add new action items if the playtest revealed issues.

### `docs/mechanics-log/mechanics-evaluated.md`
- If the playtest confirms or withdraws a mechanic, update its entry.
- If new mechanics/variants were discussed at the table (from side-notes), add them as new entries with status "Raised in Playtest N".

### `game-state/SESSION_LOG.md`
- Add a sub-section under the current session entry noting the playtest analysis.

## Step 5: Decision Tree Routing

Read `docs/test-scenarios/TESTING_PLAN.typ` — specifically the "Entry Conditions Per Stack" table and the "Decision Tree" tables (Phase 1 and Phase 2). **TESTING_PLAN.typ is the source of truth for routing thresholds — consult it directly rather than relying on numbers cached in this skill.**

Map the playtest metrics to the decision tree's branching criteria:

1. **Champion kill round** — extract from Block A tracking data. Compare against the thresholds defined in TESTING_PLAN.typ (currently: before R15 strong, R15–R20 partial, after R20 pacing urgent → Stack C — verify these are still current).
2. **Standoff persistence** — extract from Block B behavioral patterns (first contact round, evidence of standoff, forward movement timing). If standoff persists → Stack F.
3. **Bodyguard triggers** — extract from feedback form or game log. Compare against TESTING_PLAN's accepted/partial/broken thresholds.
4. **Any other routing criteria** mentioned in the entry conditions table (combo ceiling, board feel, draft staleness).

Write a routing block:

```markdown
## Decision Tree Routing

**Key metrics for routing:**
- First Champion kill: Round [N] → [before R15 / R15–R20 / after R20]
- Standoff observed: [yes/no — describe]
- Bodyguard triggers: [N]
- Combo attempts: [N successful / N attempted]

**Decision tree path:** [Stack X] → [result branch] → recommended next: **[Stack Y]**

**Reasoning:** [1-2 sentences explaining why this branch applies based on the data]
```

Include this section in the analysis doc before the final summary.

## Step 6: Present Summary

Output a concise summary to the user:
1. Top 3–5 findings (one sentence each, with source — self-reported or behavioral).
2. Which OQ verdicts changed (resolved/updated).
3. Key behavioral pattern not captured in any feedback form question.
4. **Decision tree recommendation**: which stack to run next, citing the specific metric that drives the routing.
5. Any pre-staged responses from `docs/backpocket.md` that are now triggered by the results.
