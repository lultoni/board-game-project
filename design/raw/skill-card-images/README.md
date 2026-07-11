# Skill Card Images

One JPG per skill in the current catalogue. Originally authored for the (now-archived) Typst rule sheets. The live game consumes skill metadata from the `SKILLS` registry in `game/frontend/src/lib/engine/skills.ts`, not these images; they are kept as reference art / placeholders for future visual identity.

## Naming convention

`<skill_name>.jpg` — lowercase, underscores. The 15 filenames match the skill catalogue; canonical stat blocks (cost, effect, range) live in `design/RULES.md` → Skill Reference.

## Current catalogue (15 skills)

| File | Skill | Category |
|------|-------|----------|
| `lance_thrust.jpg` | Lance | Strike |
| `hook_pull.jpg` | Hook | Strike |
| `armor_breaker.jpg` | Break | Strike |
| `rune_theft.jpg` | Steal | Strike |
| `blade_tempest.jpg` | Tempest | Strike |
| `rust_shield.jpg` | Shield | Shield |
| `field_medic.jpg` | Heal | Shield |
| `armor_smith.jpg` | Plate | Shield |
| `quick_dash.jpg` | Dash | Move |
| `air_blast.jpg` | Blast | Move |
| `precision_thrust.jpg` | Shove | Move |
| `shadow_shift.jpg` | Swap | Move |
| `retreat_plan.jpg` | Retreat | Move |
| `focus_strike.jpg` | Focus | Mystic |
| `blade_call.jpg` | Charge | Mystic |

For canonical stat blocks (cost, effect, range), see `design/RULES.md` → Skill Reference.

## Adding a new skill

1. Add the image as `<skill_name>.jpg` here (optional — reference art only).
2. Add the skill to the `SKILLS` registry in `game/frontend/src/lib/engine/skills.ts` and to `design/RULES.md` → Skill Reference.
3. Update this table.

## Future (Phase B)

These are placeholder images. Phase B (visual identity) will replace them with commissioned art. See the visual-identity design doc: `SELECT body FROM design_docs WHERE id='design-doc-game-identity-visual-naming';`.
