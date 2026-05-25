---
name: adr
description: "Create an Architecture Decision Record when an architectural decision point emerges with multiple valid approaches. Auto-triggers when Claude needs to formally present design options to the user."
argument-hint: "<topic>"
---

# Architecture Decision Record: $ARGUMENTS

## Step 1: Research Context

Read `docs/design-principles.md`, `docs/systems-and-mechanics.md`, and `game-state/OPEN_QUESTIONS.md` to understand how this decision connects to the current design state.

If external game design knowledge is needed and you're not confident in your knowledge, trigger the `/research` skill first to gather evidence before writing the ADR.

## Step 2: Present the ADR inline

Do NOT create a separate file. Present the ADR directly in the conversation using this structure:

```
## ADR: [Decision Title]

**Date**: [today]  
**Related OQs**: [OQ numbers if applicable]

### Context
[2-3 paragraphs: what prompted this, what constraints exist, how it connects to the core fantasy]

### Option A: [Name]
**How it works**: [Concrete description]
**Pros**: [Bullet list]
**Cons**: [Bullet list]

### Option B: [Name]
[Same structure]

### Option C: [Name] (if applicable)
[Same structure]

### Recommendation
[Opinionated assessment. Always evaluate against the core fantasy: "Does this make spell combos more interesting?"]
```

Ask the user to decide.

## Step 3: After Decision

When the user decides:

1. Update the relevant project doc to reflect the decision:
   - System-level decisions → `docs/systems-and-mechanics.md`
   - Principle-level decisions → `docs/design-principles.md`
   - Decision log entry → `docs/mechanics-log/mechanics-evaluated.md`
2. Update `game-state/OPEN_QUESTIONS.md` to resolve related questions.
3. If the decision requires testing, trigger `/scenario` to create the test stack.
