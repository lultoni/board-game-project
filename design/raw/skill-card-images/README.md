# Skill Card Images

One JPG per skill in the current catalogue. Used by the Typst rule sheets via `skill-icon("<name>")` (defined in `docs/test-scenarios/shared/template.typ`).

## Naming convention

`<skill_name>.jpg` — lowercase, underscores, matches the `skill-icon()` argument exactly. Filenames must stay in sync with `section-skill-reference()` in `docs/test-scenarios/shared/baseline-sections.typ`.

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

For canonical stat blocks (cost, effect, range), see `docs/test-scenarios/shared/baseline-sections.typ` → `section-skill-reference()`.

## Adding a new skill

1. Add the image as `<skill_name>.jpg` here.
2. Add a row in `section-skill-reference()` with `skill-icon("<skill_name>")` matching the filename.
3. Update this table.
4. Run `zsh docs/test-scenarios/build-pdfs.sh` to verify the icon resolves.

## Future (Phase B)

These are placeholder images. Phase B (visual identity, ~2027 per the Road Ahead in `old-game-versions/README.md`) will replace them with commissioned art. See `docs/game-identity-visual-naming.md`.
