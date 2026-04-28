---
name: scenario
description: "Create a test scenario rule sheet and feedback form for an incremental design change. Auto-triggers when a design discussion concludes with a testable change or when a new test layer is needed."
argument-hint: "<layer-N> <short description>"
---

# Test Scenario: $ARGUMENTS

## Step 1: Validate Methodology

Before writing, check the incremental testing methodology:

1. **Independence**: Is this change independent from other untested changes? If not, identify the coupling and either:
   - Bundle the coupled changes into one layer (document why), or
   - Defer this change until its dependency is tested.

2. **Stack assignment**: Which experience stack does this layer belong to? Read `docs/test-scenarios/TESTING_PLAN.typ` to check existing stacks. If it's a new stack, define it.

3. **Ordering**: Does this change depend on results from a prior untested layer? If so, note the dependency and mark the rule sheet as requiring those results first.

4. **Isolation**: Can we attribute observed effects to THIS change alone? If the change touches multiple systems, consider decomposing further.

Read `game-state/CURRENT_DESIGN.md` to get the current baseline rules.

## Step 2: Determine Layer Number and Folder

Check existing folders in `docs/test-scenarios/` to find the next available layer number. Create a new folder:

```
docs/test-scenarios/layer-N-<short-slug>/
  layer-N-<short-slug>.typ        ← rule sheet
  layer-N-feedback.typ            ← feedback form
```

After creating the files, add them to `docs/test-scenarios/build-pdfs.sh` and run `zsh docs/test-scenarios/build-pdfs.sh` to compile PDFs.

## Step 3: Write the Rule Sheet

**Rule**: Use the composable section system — do NOT copy sections verbatim.

Start the rule sheet with:

```typst
#import "../shared/template.typ": *
#import "../shared/baseline-sections.typ": *
#show: template.with(title: "Test Layer N — [Title]")
```

Then call section functions. Pass arguments ONLY for what this layer changes. Everything else uses baseline defaults automatically.

**Available section functions** (all in `baseline-sections.typ`):

```typst
#section-goal()
#section-components()
#section-setup(start-runes: 4, layer1-accepted: false)
#section-round-structure()
#section-turn-structure()
#section-movement-phase()
#section-standard-attack(damage: 2, changed: false)
#section-combo-bonus(enabled: false)      // Only include if testing combo bonus
#section-action-phase()
#section-skill-system()
#section-resource-economy(start-runes: 4, layer1-accepted: false, changed: false)
#section-health-armor()
#section-bodyguard(adjacency: "both", changed: false)
#section-skill-drafting()
#section-progression()
#pagebreak()
#section-skill-reference()
#section-quick-reference(
  attack-damage: 2,
  bodyguard-adjacency: "both",
  layer1-accepted: false,
  show-combo-bonus: false,
)
```

**Layer 1 carry-forward rule**: If Layer 1 economy is accepted, always pass `start-runes: 6, layer1-accepted: true` to `section-setup` and `section-resource-economy`.

Add a header block before the section calls with:
- Version, source lineage, feedback form pointer
- `#note-box[]` summarising what changes in this layer
- "What we're testing", "Hypothesis", "Watch for" bullets

## Step 4: Write the Feedback Form

Copy `docs/test-scenarios/shared/feedback-baseline.typ` to `layer-N-feedback.typ`.

Replace every `[LAYER: ...]` placeholder:

1. **Title**: Layer name
2. **note-box**: 1–2 sentence summary of the change
3. **Section A**: Add kill-timing fields (always include first Guard kill + first Champion kill round fields) + any layer-specific observables. Include armor totals.
4. **Section B**: 6–8 questions testing this layer's hypothesis. Number from 1.
5. **Section C OQ-monitoring**: For each OQ marked TRACKING or with Evaluation criteria for this layer in `game-state/OPEN_QUESTIONS.md`, add one question. The baseline template already includes standard monitors for OQ-10, OQ-11, OQ-34, OQ-46. Add any additional layer-specific monitoring OQs.
6. **Comparison rating row**: Label the dimension being compared (e.g. "Guards feel vs prior layer:").

## Step 5: Update Living Documents

### `docs/test-scenarios/TESTING_PLAN.typ`
- Add the new layer to its stack row in the Testing Stacks table.
- Add it to the "Ready to print now" table if PDFs are built.
- Update the Mermaid decision tree if the new layer creates a new branch or decision point.
- Run `zsh docs/test-scenarios/build-pdfs.sh` to rebuild TESTING_PLAN.pdf.

### `game-state/CURRENT_DESIGN.md`
- Add the new layer to the Incremental Test Plan table with status "Ready to test".

### `game-state/NEXT_STEPS.md`
- Add the new playtest as a prioritised action item.

### `game-state/OPEN_QUESTIONS.md`
- Link the relevant OQ(s) to this test layer.

## Step 6: Confirm

Output a summary:
1. What the layer tests and which stack it belongs to.
2. The hypothesis.
3. Where the rule sheet and feedback form were saved.
4. Which section functions were called with non-default arguments.
5. Any methodology concerns (coupling, dependencies).
