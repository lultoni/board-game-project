# ROE — Goofing Around (Design Spreadsheet)

*A design brainstorm spreadsheet from the Project ROE era. Two sheets: a Champion roster grid exploring possible character classes, and an economy simulation modelling Rune gain vs. skill costs across 40 rounds.*

---

## Sheet 1: Base — Champion Roster Grid

### Champions by Terrain Affinity

Champions were mapped to terrain types, creating natural board positioning archetypes.

| Terrain  | Champion 1       | Champion 2   | Champion 3   | Champion 4    | Champion 5   |
|----------|------------------|--------------|--------------|---------------|--------------|
| Plains   | Blacksmith (M)   | Bard (M)     | Fire Witch (O)| Paladin (H)  | Engineer (O) |
| Forest   | Thief (H)        | Scavenger (H)| Musketeer (H) | Earth Witch (E)| Wildkeeper (H)|
| Mountain | Necromancer (O)  | Vampire (×)  | Monk (D)     | Stone Golem (×)| Harpy (×)   |
| Water    | Siren (×)        | Water Mage (E)| Healer (E)  | —             | —            |

*Class codes: O = Offense, H = Hybrid, M = Boost/Mystic, E = Elemental, D = Defense, × = cut/unassigned*

### Confirmed Champion Classes (6 classes, 3 per class)

| Joker       | Boost       | Offense    | Defense    | Tile Control | Mobility    |
|-------------|-------------|------------|------------|--------------|-------------|
| Thief       | Blacksmith  | Vampire    | Monk       | Wildkeeper   | Siren       |
| Necromancer | Healer      | Fire Witch | Paladin    | Engineer     | Water Mage  |
| Scavenger   | Bard        | Musketeer  | Earth Witch| Stone Golem  | Harpy       |

### Extended Champion Pool (brainstormed but not assigned)

| Category A   | Category B  | Category C  | Category D  | Category E |
|--------------|-------------|-------------|-------------|------------|
| Alchemist    | Druid       | Hunter      | Spirit Mage | Trapper    |
| General      | Archer      | —           | —           | —          |

### Race Distribution (brainstorm)

| Race   | Humans | Dwarfs | Elves | Orcs | Other |
|--------|--------|--------|-------|------|-------|
| Count  | 6      | 2      | 3     | 3    | 4     |

### Gender Distribution

| Male | Female | Other |
|------|--------|-------|
| 5    | 5      | 8     |

### Skill Slot Analysis

*How many base and added abilities each class has — used to balance the draft pool.*

| Class       | Joker | Boost | Offense | Defense | Tile Control | Mobility |
|-------------|-------|-------|---------|---------|--------------|----------|
| Base        | 3     | 6     | 6       | 6       | 6            | 6        |
| Added       | 3     | 5     | 5       | 3       | 5            | 3        |
| **Total**   | **6** | **11**| **11**  | **9**   | **11**       | **9**    |
| % of pool   | 10.5% | 19.3% | 19.3%   | 15.8%   | 19.3%        | 15.8%    |

*Joker class deliberately smaller — represents the wildcard/special role.*

### Draft Parameters (Ranger = Champion in this context)

| Variant     | Pool Size | Picks | Bans | Left after draft | Notes                              |
|-------------|-----------|-------|------|------------------|------------------------------------|
| Old (12)    | 12        | 5     | 4    | 3                | Original design                    |
| New (18)    | 18        | 5     | 4    | 9                | Only one per terrain type          |
| New (20)    | 20        | 5     | 8    | 7                | Only one ranger per type in game   |

### Board Size Scaling Reference

*Comparison of tile counts and ratios when scaling board size.*

| Size  | Tiles | Ratio vs 8×8 |
|-------|-------|--------------|
| 8×8   | 64    | 1.00×        |
| 9×9   | 81    | 1.27×        |
| 10×10 | 100   | 1.56×        |
| 11×11 | 121   | 1.89×        |
| 12×12 | 144   | 2.25×        |
| 13×13 | 169   | 2.64×        |
| 14×14 | 196   | 3.06×        |
| 15×15 | 225   | 3.52×        |
| 16×16 | 256   | 4.00×        |

### Terrain Stat Modifiers

*How terrain affects piece stats when standing on it.*

| Terrain  | Armor | M Range | S Range | S Damage | S Cost |
|----------|-------|---------|---------|----------|--------|
| Water    | −1    | 0       | +1      | 0        | 0      |
| Forest   | +1    | 0       | −1      | 0        | 0      |
| Plains   | −1    | +1      | 0       | 0        | 0      |
| Mountain | 0     | −1      | +1      | 0        | 0      |

*No terrain affects skill damage or cost — the combat knobs are movement range and skill range only.*

---

## Sheet 2: Rune Amount Change — Economy Simulation

*Models Rune accumulation and how many skills a player can realistically activate per round, using the original ROE economy parameters.*

### Parameters

| Min Skill Cost | Avg Skill Cost | Max Skill Cost | Rune gain increase every N rounds | Skill slots increase every N rounds | Start using skills from round |
|----------------|----------------|----------------|------------------------------------|--------------------------------------|-------------------------------|
| 1              | 2.59           | 4              | 7                                  | 10                                   | 5                             |

### Economy Simulation (Rounds 1–39)

*"Real SkU" = realistic skill uses that round given available Runes. "Skill Use Dif" = shortfall vs. available skill slots.*

| Round | Rune Gain | Skill Slots | Runes Available | SkU (avg cost) | Shortfall | SkU (max cost) | Shortfall |
|-------|-----------|-------------|-----------------|----------------|-----------|----------------|-----------|
| 1     | +1        | 2           | 5               | 0              | 2         | 0              | 2         |
| 2     | +1        | 2           | 6               | 0              | 2         | 0              | 2         |
| 3     | +1        | 2           | 7               | 0              | 2         | 0              | 2         |
| 4     | +1        | 2           | 8               | 0              | 2         | 0              | 2         |
| 5     | +1        | 2           | 9               | 2              | 0         | 2              | 0         |
| 6     | +1        | 2           | 10              | 1              | 1         | 0              | 2         |
| 7     | +2        | 2           | 12              | 1              | 1         | 1              | 1         |
| 8     | +2        | 2           | 14              | 1              | 1         | 0              | 2         |
| 9     | +2        | 2           | 16              | 1              | 1         | 1              | 1         |
| 10    | +2        | 3           | 18              | 0              | 3         | 0              | 3         |
| 11    | +2        | 3           | 20              | 1              | 2         | 1              | 2         |
| 12    | +2        | 3           | 22              | 1              | 2         | 0              | 3         |
| 13    | +2        | 3           | 24              | 1              | 2         | 1              | 2         |
| 14    | +3        | 3           | 27              | 1              | 2         | 0              | 3         |
| 15    | +3        | 3           | 30              | 1              | 2         | 1              | 2         |
| 16    | +3        | 3           | 33              | 1              | 2         | 1              | 2         |
| 17    | +3        | 3           | 36              | 1              | 2         | 1              | 2         |
| 18    | +3        | 3           | 39              | 2              | 1         | 0              | 3         |
| 19    | +3        | 3           | 42              | 1              | 2         | 1              | 2         |
| 20    | +3        | 4           | 45              | 1              | 3         | 1              | 3         |
| 21    | +4        | 4           | 49              | 1              | 3         | 1              | 3         |
| 22    | +4        | 4           | 53              | 2              | 2         | 1              | 3         |
| 23    | +4        | 4           | 57              | 2              | 2         | 1              | 3         |
| 24    | +4        | 4           | 61              | 1              | 3         | 1              | 3         |
| 25    | +4        | 4           | 65              | 2              | 2         | 1              | 3         |
| 26    | +4        | 4           | 69              | 1              | 3         | 1              | 3         |
| 27    | +4        | 4           | 73              | 2              | 2         | 1              | 3         |
| 28    | +5        | 4           | 78              | 2              | 2         | 1              | 3         |
| 29    | +5        | 4           | 83              | 2              | 2         | 1              | 3         |
| 30    | +5        | 5           | 88              | 1              | 4         | 2              | 3         |
| 31    | +5        | 5           | 93              | 2              | 3         | 1              | 4         |
| 32    | +5        | 5           | 98              | 2              | 3         | 1              | 4         |
| 33    | +5        | 5           | 103             | 2              | 3         | 1              | 4         |
| 34    | +5        | 5           | 108             | 2              | 3         | 2              | 3         |
| 35    | +6        | 5           | 114             | 3              | 2         | 1              | 4         |
| 36    | +6        | 5           | 120             | 2              | 3         | 2              | 3         |
| 37    | +6        | 5           | 126             | 2              | 3         | 1              | 4         |
| 38    | +6        | 5           | 132             | 2              | 3         | 2              | 3         |
| 39    | +6        | 5           | 138             | 3              | 2         | 1              | 4         |

### Key Observations

- **Rounds 1–4**: Players hoard. No skills activated — Runes accumulate to ~8–9 before spending begins.
- **Round 5**: First real skill use. 9 Runes covers ~2 average-cost skills or ~2 minimum-cost.
- **Rounds 7+**: Rune gain doubles. Skills become more frequent but still never fill all skill slots.
- **The shortfall never closes**: Even at round 39 with 6 Rune/round gain and 5 skill slots, players average only 2–3 skill activations per round — always shy of filling all slots. The game was designed so you could *never* do everything you wanted.
- **Implication for current design**: The current game (start 6 Runes, +2/turn from R2) was tuned to avoid the dead rounds 1–4 problem — confirmed by accepting Layer 1.
