---
name: scenario
description: "Create a test scenario rule sheet and feedback form for an incremental design change. Auto-triggers when a design discussion concludes with a testable change or when a new test stack is needed."
argument-hint: "<stack-X> <short description>"
---

# Test Scenario: $ARGUMENTS

## Step 1: Validate Methodology

Before writing, check the incremental testing methodology:

1. **Independence**: Is this change independent from other untested changes? If not, identify the coupling and either:
   - Bundle the coupled changes into one stack (document why), or
   - Defer this change until its dependency is tested.

2. **Stack assignment**: Which experience stack does this scenario belong to? Read `docs/test-scenarios/TESTING_PLAN.typ` to check existing stacks. If it's a new stack, define it.

3. **Ordering**: Does this change depend on results from a prior untested stack? If so, note the dependency and mark the rule sheet as requiring those results first.

4. **Isolation**: Can we attribute observed effects to THIS change alone? If the change touches multiple systems, consider decomposing further.

Read the current baseline rules from the canonical sources: `docs/design-principles.md`, `docs/systems-and-mechanics.md`, and `docs/test-scenarios/baseline/ruleset-baseline.typ` (and the shared section functions in `docs/test-scenarios/shared/baseline-sections.typ`).

## Step 2: Determine Stack Letter and Folder

Check existing folders in `docs/test-scenarios/` to find the next available stack letter (or use the existing stack folder if extending it). Folder convention:

```
docs/test-scenarios/stack-X-<short-slug>/
  stack-X-<short-slug>.typ        ← rule sheet
  stack-X-feedback.typ            ← feedback form (one per stack, shared across games in stack)
```

If the stack runs multiple games (e.g. game1, game2), name files `stack-X-game1-<slug>.typ` etc. Check existing `stack-a-cleverness/` for the convention.

After creating the files, the discovery-based `build-pdfs.sh` will pick them up automatically — run `zsh docs/test-scenarios/build-pdfs.sh` to compile PDFs.

## Step 3: Write the Rule Sheet

**Rule**: Use the composable section system — do NOT copy sections verbatim.

Start the rule sheet with:

```typst
#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Stack X — [Title]")
```

Then call section functions. Pass arguments ONLY for what this stack changes. Everything else uses baseline defaults automatically.

**Available section functions** (all in `baseline-sections.typ` — re-read it before invoking to confirm current API):

```typst
#section-goal()
#section-components()
#section-setup()
#section-round-structure()
#section-turn-structure()
#section-movement-phase()
#section-standard-attack()
#section-action-phase()
#section-skill-system()
#section-resource-economy()
#section-health-armor()
#section-bodyguard()
#section-skill-drafting()
#section-progression()
#pagebreak()
#section-skill-reference()
#section-quick-reference(overrides: (:))   // pass overrides for changed rows only
```

For changed rules, inline the changed section directly with a `⚡ CHANGED:` callout and a before/after table — don't call the baseline function for that section.

For the Quick Reference table: call `#section-quick-reference(overrides: ("Concept Name": [new content]))` to override only the rows your stack changes. Do not copy the whole table.

Add a header block before the section calls with:
- Version (use `#BASELINE_VERSION` from baseline-sections.typ), source lineage, feedback form pointer.
- `#note-box[]` summarising what changes in this stack.
- "What we're testing", "Hypothesis", "Watch for" bullets.

## Step 4: Write the Feedback Form

Create `stack-X-feedback.typ` by importing the shared feedback functions from `docs/test-scenarios/shared/feedback-baseline.typ`. Write only Section A (stack-specific observables) and Section B (stack-specific hypothesis questions). Use the imported `section-c-skeleton(extra-questions: (...))` for Section C — do not rebuild it from scratch.

Section A: Add kill-timing fields (always include first Guard kill + first Champion kill round fields) + any stack-specific observables. Include armor totals.

Section B: 6–8 questions testing this stack's hypothesis. Use `#fq[...]` for auto-numbering — do not hardcode question numbers.

Section C OQ-monitoring: For each OQ marked TRACKING in `game-state/OPEN_QUESTIONS.md` for this stack, pass it as an extra question to `section-c-skeleton`.

Include a comparison rating row labelling the dimension being compared (e.g. "Guards feel vs prior stack:").

## Step 5: Update Living Documents

### `docs/test-scenarios/TESTING_PLAN.typ`
- Add the new stack to its row in the Testing Stacks table.
- Add it to the "Ready to print now" table if PDFs are built.
- Update the Mermaid decision tree if the new stack creates a new branch or decision point.
- Run `zsh docs/test-scenarios/build-pdfs.sh` to rebuild TESTING_PLAN.pdf.

### `docs/systems-and-mechanics.md`
- Add the new stack to the Incremental Test Plan table with status "Ready to test".

### `game-state/NEXT_STEPS.md`
- Add the new playtest as a prioritised action item.

### `game-state/STATUS.md`
- Update Current Focus / Next Action to point at the new stack if it's the next thing to play.

### `game-state/OPEN_QUESTIONS.md`
- Link the relevant OQ(s) to this test stack.

## Step 6: Confirm

Output a summary:
1. What the stack tests and which experience stack it belongs to.
2. The hypothesis.
3. Where the rule sheet and feedback form were saved.
4. Which section functions were called with non-default arguments.
5. Any methodology concerns (coupling, dependencies).
