# Skill Card Images

One JPG per skill in the current catalogue. Used by the Typst rule sheets via `skill-icon("<name>")` (defined in `docs/test-scenarios/shared/template.typ`).

## Naming convention

`<skill_name>.jpg` — lowercase, underscores, matches the `skill-icon()` argument exactly. Filenames must stay in sync with `section-skill-reference()` in `docs/test-scenarios/shared/baseline-sections.typ`.

## Current catalogue (15 skills)

| File | Skill | Category |
|------|-------|----------|
| `lance_thrust.jpg` | Lance Thrust | Strike |
| `hook_pull.jpg` | Hook Pull | Strike |
| `armor_breaker.jpg` | Armor Breaker | Strike |
| `rune_theft.jpg` | Rune Theft | Strike |
| `blade_tempest.jpg` | Blade Tempest | Strike |
| `rust_shield.jpg` | Rust Shield | Shield |
| `field_medic.jpg` | Field Medic | Shield |
| `armor_smith.jpg` | Armorsmith | Shield |
| `quick_dash.jpg` | Quick Dash | Move |
| `air_blast.jpg` | Air Blast | Move |
| `precision_thrust.jpg` | Precision Thrust | Move |
| `shadow_shift.jpg` | Shadow Shift | Move |
| `retreat_plan.jpg` | Retreat Plan | Move |
| `focus_strike.jpg` | Focus Strike | Mystic |
| `blade_call.jpg` | Blade Call | Mystic |

For canonical stat blocks (cost, effect, range), see `docs/test-scenarios/shared/baseline-sections.typ` → `section-skill-reference()`.

## Adding a new skill

1. Add the image as `<skill_name>.jpg` here.
2. Add a row in `section-skill-reference()` with `skill-icon("<skill_name>")` matching the filename.
3. Update this table.
4. Run `zsh docs/test-scenarios/build-pdfs.sh` to verify the icon resolves.

## Future (Phase B)

These are placeholder images. Phase B (visual identity, ~2027 per the Road Ahead in `old-game-versions/README.md`) will replace them with commissioned art. See `docs/game-identity-visual-naming.md`.
