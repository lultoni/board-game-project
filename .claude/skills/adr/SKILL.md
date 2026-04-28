---
name: adr
description: "Create an Architecture Decision Record when an architectural decision point emerges with multiple valid approaches. Auto-triggers when Claude needs to formally present design options to the user."
argument-hint: "<topic>"
---

# Architecture Decision Record: $ARGUMENTS

## Step 1: Determine ADR Number

Check existing files in `docs/decisions/` to find the next available ADR number. Use the format: `ADR-NNN-slug.md` (e.g., `ADR-003-skill-slot-scaling.md`).

## Step 2: Research Context

Read `game-state/CURRENT_DESIGN.md` and `game-state/OPEN_QUESTIONS.md` to understand how this decision connects to the current design state.

If external game design knowledge is needed and you're not confident in your knowledge, trigger the `/research` skill first to gather evidence before writing the ADR.

## Step 3: Write the ADR

Create `docs/decisions/ADR-NNN-slug.md` with this structure:

```markdown
# ADR-NNN: [Decision Title]

**Status**: PROPOSED  
**Date proposed**: [today]  
**Decision maker**: Elias  
**Related questions**: [OQ numbers if applicable]

## Context

[2-3 paragraphs explaining:
- What prompted this decision
- What constraints exist
- How this connects to the core fantasy and existing systems]

## Options

### Option A: [Name]
**Reference games**: [Published games that use this approach]
**How it works**: [Concrete mechanics description]
**Pros**: [Bullet list]
**Cons**: [Bullet list]

### Option B: [Name]
[Same structure]

### Option C: [Name] (if applicable)
[Same structure]

## Comparison Matrix

| Criterion | Option A | Option B | Option C |
|-----------|----------|----------|----------|
| Serves core fantasy (spell combos) | | | |
| Cognitive load | | | |
| Interaction with [relevant system] | | | |
| Testability (can we isolate this?) | | | |
| [Other relevant criteria] | | | |

## Assessment

[Your analysis of which option best serves the project. Be opinionated but transparent about trade-offs. Always evaluate against the core fantasy: "Does this make spell combos more interesting?"]

## Recommended Next Step

[What should happen after the decision is made — typically "create a test scenario" or "incorporate into Layer N".]
```

## Step 4: Present to User

Output a concise summary of the options (3-5 sentences per option) and your recommendation. Ask the user to decide.

## Step 5: After Decision

When the user decides:

1. Update the ADR status to `ACCEPTED` or `REJECTED` with the date and rationale.
2. Update `game-state/CURRENT_DESIGN.md` to reflect the decision.
3. Update `game-state/OPEN_QUESTIONS.md` to resolve related questions.
4. If the decision requires testing, trigger `/test-scenario` to create the test layer.
