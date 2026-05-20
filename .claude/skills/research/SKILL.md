---
name: research
description: "Generate a Perplexity research request with project context when external game design knowledge is needed. Auto-triggers when a knowledge gap is identified about game mechanics, comparable games, player psychology, or design patterns."
argument-hint: "<topic>"
---

# Research Request: $ARGUMENTS

## Step 1: Check for Existing Research

Search `docs/research/` for any existing files that cover this topic or a closely related one. If found, read the file and assess whether it already answers the question. If it does, summarise the existing findings instead of generating a new request.

## Step 2: Build Context Block

Read the current design state from the canonical trio: `docs/design-principles.md`, `docs/systems-and-mechanics.md`, and `game-state/OPEN_QUESTIONS.md`. Identify which specific systems or design questions relate to this research topic.

## Step 3: Generate Perplexity Prompt

Output a formatted research request the user can paste into Perplexity:

```
RESEARCH REQUEST: [Topic]

Context: We are designing a 2-player perfect-information tactical board game where players command armies on a grid, equipping Champions with skills/spells and spending Runes to activate them. The core fantasy is discovering and executing clever spell/skill combos.

[Include 2-3 sentences of specific context about which system or question this research relates to, drawn from systems-and-mechanics.md and OPEN_QUESTIONS.md.]

Questions:
1. [Specific question 1]
2. [Specific question 2]
3. [Specific question 3 — if needed]

Please include:
- Specific examples from published games
- Designer commentary or postmortems where available
- Any relevant academic/GDC research
- Concrete mechanics, not just philosophy
```

## Step 4: Wait and Process

Tell the user to:
1. Paste the prompt into Perplexity (or their preferred research tool).
2. Save the results as a markdown file in `docs/research/` with a descriptive filename.
3. Tell you the filename when done.

When the user provides the filename:
1. Read the saved research file.
2. Summarise the key findings relevant to the current design.
3. Identify any findings that should update `docs/systems-and-mechanics.md`, `game-state/OPEN_QUESTIONS.md`, or `game-state/NEXT_STEPS.md`.
4. Propose specific design implications (don't just summarise — connect to our game).
