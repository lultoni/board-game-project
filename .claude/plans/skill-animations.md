# Skill Animations — Design & Build Plan

**Approved:** Session 43. Approach: each skill gets its own theatrical micro-scene enacting the fantasy. Ink-on-paper aesthetic (no bloom, no neon-VFX). Timing budget 300–700ms typical, up to 800–900ms for hero animations.

## Foundation work (must land first)

### F1. Plumb skill context through the effect pipeline
Renderer today derives effects from mailbox pre/post diffs and does not know which skill fired.

- `decodeAction` gives us `{ kind, skillId, src, target }` — thread `skillId` and `src` (casterSq) into the effect emission phase inside `ply-renderer.svelte.ts`.
- Emission sites to update:
  - `emitImpactEvents` — take casterSq + skillId, so it can pick the right choreography per skill.
  - `emitRelocationAndDeathEvents` — same.
  - Add an explicit self-cast emission path for Focus/Charge where the mailbox is unchanged and only modifier bits flip. Trigger keyed on `decoded.skillId`.
- After F1, per-skill effect selection becomes a switch on skillId at emission time. Existing generic effects (impact ring, dust, damage number, armor hex, heal ring) remain the fallback base layer.

### F2. Effect primitives (motion vocabulary)
Rather than one effect kind per skill, use a small primitive set that skills compose. Add to `viz/effects.ts`:

- `stab` — thin stroke that extends from src to target, punches past, retracts.
- `hook-pull` — curved bezier drawn out, then straightens taut while target slides.
- `sweep` — long arcing stroke through multiple squares, staggered impacts.
- `thread` — soft curved line drawn from src to target that wraps at destination.
- `glyph-travel` — a small drawn shape (shield silhouette, coin, diamond) that moves along an arc from src to target.
- `burst` — short radial ticks around a point.
- `sigil` — glyph drawn at a single square (crosshair, coil, shield-outline).
- `piece-motion` — scale/vibrate/glow applied to the piece itself (crouch, focus scale-bump, charge vibrate).

Each primitive has: `startedAt`, duration segments (attack/hold/release), color, geometry inputs. Draw code lives in `EffectsLayer.svelte`.

## Per-skill choreography

### Strikes (color #cc3a2a)

**Lance — the stab**
- Primitives: `stab`
- Long thin spear-mark grows from caster toward target (120ms), punches 8px past target center (60ms hold), retracts (200ms), fades (200ms).
- Impact ring + damage number fire on retract; target shakes on punch-through frame.
- Total ~600ms.

**Hook — the pull** *(hero animation)*
- Primitives: `hook-pull`
- Curved hook-line (bezier, pronounced drop mid-path) draws from caster to target (200ms). Catches (80ms hold). Line pulls taut over 180ms while target piece slides toward caster along the same path. Fades 200ms.
- Impact ring at final position.
- Total ~660ms.

**Break — the shatter**
- Primitives: existing armor-copper + `burst`
- Short thick chisel-mark stamps down onto target (100ms). Radial crack-lines (4–6 short jagged strokes, 3–5px, 60ms draw) emanate from impact point, fade 250ms.
- If no armor (HP damage instead): skip cracks, use damage number.
- Total ~410ms.

**Steal — the pickpocket**
- Primitives: `stab` (dashed variant) + `glyph-travel`
- Thin dashed line darts from caster to target (120ms). Impact ring fires. Small coin-glyph (filled circle w/ hairline outline, ~6px) flies back along same path to caster (200ms). Caster piece scale-pulses 105% (80ms) on arrival. Fade 100ms.
- Fixes today's invisible $-transfer problem.
- Total ~600ms.

**Tempest — the sweep** *(hero animation)*
- Primitives: `sweep`
- One continuous arcing stroke from caster curving through all affected squares (400ms draw). Each affected square's impact ring fires as the stroke passes over it (~60ms stagger). Stroke fades tail-to-head (250ms).
- Total ~650ms.

### Shield category (color #3a7acc)

**Shield (self) — the brace**
- Primitives: `piece-motion` (crouch) + `sigil` (shield silhouette)
- Piece crouches 90% (100ms), springs to 105% (120ms), settles to 100% (100ms). Concurrent: heater-shield silhouette (~22px tall, blue) draws top-down over piece (140ms), holds 200ms, fades 180ms. Existing armor-hex on pip row follows.
- The piece scale animation is what makes this readable in peripheral vision.
- Total ~620ms.

**Heal — the mending thread**
- Primitives: `thread`
- 2px green thread (60% opacity) draws from caster to ally (200ms curved). At ally: thread wraps piece into a closed loop (~14px, 120ms). Loop pulses once (grows 2px, shrinks back, 100ms) as +HP number appears. Loop and thread fade together (240ms).
- Total ~660ms.

**Plate — the shield handed over**
- Primitives: `glyph-travel` + existing armor-hex
- Small shield-glyph (heater silhouette, paper-bg fill, blue outline) travels caster→ally along arc (220ms). Settles onto ally: glyph shrinks to fit piece (80ms) while armor-hex fires below. Fade 200ms.
- Distinct from self-Shield: self is *earned* (crouch+brace), Plate is *given* (glyph handed over).
- Total ~500ms.

### Move category (color #3aaa55)

**Dash** — no change. Existing dust trail already reads well.

**Blast — the leap-and-strike**
- Primitives: existing dust trail + `burst`
- Piece slide (dust trail) covers movement. On arrival: red radial burst (4–6 short strokes, 4px, 30°/70°/110°/... angles) from landing square (120ms draw, 200ms fade). Impact ring + damage number if a piece was damaged.

**Shove — the push**
- Primitives: `stab` (arrow variant) + existing dust trail on target
- Thick arrow-stroke (4px tapering to 2px tip) draws from caster's edge toward target (100ms wind-up). Arrow-tip touches target as slide begins; arrow follows the piece for the first 60% of slide, then lets go and fades (200ms).

**Swap — the exchange**
- Primitives: `piece-motion` (curved slides) + `sigil` (diamond)
- Both pieces slide simultaneously along interlocking curved paths (existing dust trails, but curved not straight). At crossing midpoint: purple ◇ diamond glyph (8px) appears for 120ms as they pass — the "moment of exchange."

**Retreat**
- Primitives: existing dust trail + small `burst`
- Piece slide with dust trail. Add short trailing arrow-tail at origin square (3 small parallel ticks, 100ms draw, 300ms fade), concurrent with slide.

### Mystic category (color #8a4abd)

**Focus — the sharpen**
- Primitives: `sigil` (crosshair) + `piece-motion` (scale + outline)
- Four crosshair ticks converge on piece from N/S/E/W (140ms, 24px out → 16px out). On arrival: subtle 1.5px purple outline around piece silhouette, holds 300ms. Piece receives 102% scale bump for the duration. Fade 200ms.
- Total ~640ms.

**Charge — the wind-up**
- Primitives: `sigil` (coil) + `piece-motion` (vibrate)
- Spiral coil-mark draws outward from piece center (220ms, 1.5 turns, ~14px radius). Piece vibrates 1–2px random offset per frame during coil draw, damping as coil completes. Coil holds 240ms with slow clockwise rotation (~10°). Fade 180ms.
- Total ~640ms.

## Build order

1. **F1** — plumb `skillId` + `src` through emission pipeline. Nothing else works without this.
2. **F2** — add effect primitives to `viz/effects.ts` + drawing in `EffectsLayer.svelte`. Start with the primitives needed for the first hero skill.
3. **Focus + Charge** — currently zero visual feedback. Highest visibility win. Uses `sigil` + `piece-motion` primitives.
4. **Self-Shield** — most-requested improvement (missable today). Uses `piece-motion` + `sigil`.
5. **Hook** — hero animation, validates the choreography approach end-to-end.
6. **Steal** — fixes invisible $-transfer. Uses `stab` + `glyph-travel`.
7. **Tempest** — second hero, exercises `sweep`.
8. **Lance, Break, Blast, Shove, Swap, Retreat, Heal, Plate** — fill in the rest in inventory order.

## Constraints / notes

- Ink-on-paper aesthetic: no bloom, no gradients that scream digital, no neon.
- Variable stroke weight (nib pressure feel): baseWidth * (1 + 0.6 * sin(t * PI)).
- Deterministic wobble for bezier control offsets (hash of src+target) so paths feel intentional, not random.
- All new effects respect reduced-motion where existing ones do.
- Watch for perf: multi-target skills (Tempest) may push RAF hard — keep stroke geometry cheap.

## Open questions

- Do we want Charge's coil to persist as a subtle indicator on the piece while the modifier is active, or does it fade entirely after cast? (Focus has the same question with its outline.) Decision: fade entirely on cast; ProgressionPanel / piece-state already communicates the persistent modifier state.
- If a piece gets hit while Focus/Charge modifier is set, is there a "lost modifier" cue? Out of scope for this plan; revisit if playtest reveals it matters.
