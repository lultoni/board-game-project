# Old Versions Ideas — Triage

*Every idea from `docs/research/old-versions-ideas.md` assessed against current design state. Three buckets:*

- **New / Possible** — not yet evaluated; worth discussing or testing
- **Deferred — Related** — already tracked in OPEN_QUESTIONS.md or NEXT_STEPS.md; kept for cross-reference
- **Archived / Similar** — proven not to work, already decided, or too far outside current scope

*References use OQ-N for Open Questions, ADR-N for Architecture Decision Records.*

---

## New / Possible

Ideas worth exploring when the time is right. Annotated with conditions from user feedback (Session 10).

### Skills — Skill Catalogue Expansion (connects Stack F / Cleverness II)

*These are all "over time" items — the skill catalogue grows gradually. Not priority until core combat stacks are resolved.*

| Idea | Source | Condition | User Notes |
|------|--------|-----------|------------|
| **Attack-then-reposition on single activation** (Ambush, Hakenzug, Klingenorkan) | 3j | When expanding catalogue. Don't add too many at once — focus on game systems first. | "Worth exploring but an over-time thing" |
| **Line pull** (Strömungsruf) | 3r | Needs elegant single-rule formulation with no edge cases. Staged in backpocket. | "Very interesting — execution must be elegant" |
| **Range empowerment expansion** (Bardic Inspiration, Blood Spear, Klingenruf) | 3q | Already have Focus Strike. New additions must not be OP or make others redundant. | "Expand over time, avoid redundancy" |
| **Speed boost via skill** (Eagle Vision: +1 Speed next turn) | 3s | Must analyse gap vs. existing Move skills — what does this fill that Horse Leap doesn't? | "How is this different from move skills?" |
| **Heal at range** (Plague Medicine: heal Injured on Skill Path) | 3t | Question of cost balance vs. Field Medic adjacent heal + combo alternatives. | "Worth exploring, cost vs. alternatives matters" |
| **Armor Destruction as explicit offense category** | 3m | Risk: makes Armor a dead skill if anti-Armor too strong. Balance carefully. | "Could lead to never using Armor" |

### Skills — Needs Tracking Solution First

*Blocked until `/research how board games track temporary effects on pieces` is resolved.*

| Idea | Source | Condition | User Notes |
|------|--------|-----------|------------|
| **Temporary Armor** (absorbed once, then gone) | 3l | Needs physical tracking method. Also needs a brainstorm on which game feel it serves. | "Tracking strain. Worth it only if we solve tracking AND know what feel it creates" |
| **Shield duration system** (turn-based expiry) | 3c | Same tracking problem. Also: economy balance (too expensive = never used; too cheap = unbreakable). | "Tracking + economy balance both needed" |
| **Active Guard-Bind** (Wächterband) | 3n | Cool but implementation unclear. How to track which Guard is bound? | "Temp move block for Guards — how to implement?" |

### Meta / Design

| Idea | Source | Condition | User Notes |
|------|--------|-----------|------------|
| **Piece compatibility / adjacency synergies** | 11c | User excited. Needs a proper brainstorm session. Connects to OQ-51. | "Love synergies — rewards complex setups and positioning" |
| **ROE AI evaluation factors as a playtest lens** | 9a | Use as human evaluation framework, NOT as AI opponent. The factors (Guard protection, LoS sheltering) inform what to watch during playtests. | "Interesting as lens, but don't let AI solve the game" |

### Economy

| Idea | Source | Condition | User Notes |
|------|--------|-----------|------------|
| **Rune cap (e.g. 8 max)** | 5c | User prefers making spending attractive over forcing it. Keep monitoring (OQ-46). Only if hoarding becomes a real problem despite attractive spending options. | "Cleanest fix is encouraging spending, not capping" |

---

## Deferred — Related

Ideas already tracked in the project. Included here so nothing falls through the cracks.

| Idea | Source | Tracked As | Status |
|------|--------|-----------|--------|
| **Draft from pool, then assign to Champions** | 7c / 7d | OQ-35 | Deferred post-Layer 3 |
| **Flexible piece placement / reveal-style simultaneous** | 7d | OQ-36 + OQ-48 | Deferred post-Layer 3, bundled |
| **Pick-and-ban Champion draft** | 7a | OQ-35 (draft variant) | Deferred post-Layer 3 |
| **18 Champions to draft from** | 7b | OQ-27 (piece count) / future expansion | Deferred post-v1 |
| **3 Move Slots** | 6a | mechanics-evaluated.md | Deferred — may be superseded by AP system (Layer 4) |
| **8×8 board** | 2e | OQ-1 / Layer 5 | Deferred post-Layer 3 |
| **Checkmate-style win condition** | 8c | OQ-19 / OQ-51 | Layer C (Pacing) — concrete trigger: if first Champion kill still past R20 after Layer 2 |
| **Unified AP system** | 6a analogue | OQ-26 / Layer G | Draft written in `stack-g-structure/` |
| **Hex board** | 2e analogue | OQ-42 | Reopened — needs `/research` first |
| **Capture bonus (Rune income for kills)** | 5b | OQ-47 | Closed Session 8 — performance-based income constrains strategy |
| **Centre-tile Rune bonus** | 2g / 5b | OQ-47 | Closed Session 8 (same reason as above) |
| **King advancement Rune bonus** | 5b | OQ-47 | Closed Session 8 |
| **Skill pool draft** | 7a | OQ-35 | Deferred post-Layer 3 |
| **Sacrifice piece for economy gain** | 5e | OQ-25 | Closed — 2-slot scarcity makes unworkable; post-v1 expansion variant |
| **Rune Theft balance** | 5f / 3b | OQ-34 | Monitoring in Layer 2 |
| **Minor/Major skill slot cost** | 3c analogue | OQ-50 | Deferred — design ultimate skill candidates first |
| **Combo bonus / skill synergy rewards** | 11c | OQ-51 / Layer 2 | Ready to test (Stack A, Game 2) |
| **Cascade trigger: kill → free skill slot** | 9b analogue | OQ-51 | Open candidate lever |
| **Positional payoff: haven't moved → +1 Range** | — | OQ-51 | Open candidate lever |
| **King has 3 skill slots** | — | backpocket.md (known issue) | Deferred post-v1. Risk: King becomes ultimate stay-back support with heal+buff. Test only after core combat is locked. |
| **Spirit Mage as named win-condition piece** | 10b | design-language.md | Deferred post-v1-tuning — identity/narrative phase (Phase B). |
| **Terrain stat modifier system** (damage/cost neutral) | 2f | ADR-001/002 | Deferred — only if terrain ever returns. Cognitive load + map-lock concerns. |
| **Mirror board / FEN seed generation** | 2g | ADR-001/002 | Deferred — only if terrain returns as map variant. |
| **One Champion per terrain type constraint** | 7g | ADR-001/002 | Deferred — no terrain system in place. Surface if terrain returns. |
| **Draw if only Kings remain** | — | backpocket.md (staged idea) | Deferred — trigger: only-Kings-left endgames become common and unfun. |

---

## Archived / Similar

Ideas that are definitively ruled out, already decided against, or out of scope for v1.

| Idea | Source | Why Archived |
|------|--------|-------------|
| **Terrain system** (bonuses from standing on terrain types) | 2a–2g | Removed ADR-001/002 — confirmed overhead complexity. Can return as post-v1 map variant, but the system itself is off the table for v1. |
| **Terrain confers Armor on entry** | 2b | Terrain removed (see above). |
| **Terrain affects movement speed** | 2c | Terrain removed. |
| **Skill Range modified by terrain** | 3g | Terrain removed. |
| **Champions take self-damage on standard attacks** | 4a | No self-damage mechanic in current game. Adds significant cognitive load; the current design solves attack dominance via the 1 DMG nerf (Stack A), not via self-damage. |
| **Mutual kill on Guard-vs-Champion melee** | 4b | Current game doesn't have asymmetric instant-kill melee. The 2 HP system handles this gracefully without special-case rules. |
| **Attack destroys both if same type** | 4c | Same-type mutual kill adds a layer of rules for diminishing returns. The HP system already handles trading. |
| **Persistent tile effects (death zones, blocked tiles)** | 3f / 2f | Requires per-tile effect tracking on a physical board. Overhead complexity ruled out with terrain. Feasible in a digital version. |
| **AoE skills (Tremor 2×2, Inferno 3×1)** | 3e | No AoE in current game (by design — skill paths are linear). AoE changes the fundamental spatial model. Out of scope for v1; possible post-v1 expansion. |
| **Spell reflection** (Aerial Shield / Mirror Shield) | 3h | "Reflect any spell back at caster" requires tracking which skills were cast at which target. Rule complexity cost exceeds value for v1. |
| **Rock-Paper-Scissors element advantage** | 1b | Elemental counter-system. Adds hidden complexity and type-tracking overhead. "Perfect information" design principle applies — element matchups create opaque interactions. Out of scope. |
| **Elemental identity / terrain affinity per piece** | 1a / 1f | Terrain removed. Element identities without terrain have nothing to attach to. |
| **Tile control as skill category** (Inferno, Rock Slide, blocked zones) | 3d | Requires per-tile tracking (see persistent tile effects). Physical board management overhead. Out of scope v1. |
| **Linked movement** (move to act) | 6b | Explicitly withdrawn in Session 1, confirmed P1 — unlinked preferred. Likely superseded by AP system (Layer 4). |
| **Attack phase separate from skill phase** | 6c | Superseded by current Movement → Action structure. AP system (Layer 4) may change this further. |
| **CR-style draft (strict single interleaving)** | 7e | Closed OQ-43 — restricts strategy, creates "correct" picks with small catalogue. Variant material only. |
| **Ban phase in skill draft** | 7a variant | Closed OQ-44 — from older Champion-fixed-skills model. Needs 20+ skills and different draft model. |
| **Performance-based Rune gain** (captures, centre tile, King advance) | 5b | Closed OQ-47 — forces single playstyle, constrains creative expression. Auto-economy is strategy-neutral. |
| **Economy skills as skill slots** | OQ-25 | Closed Session 8 — slot-tax too high, sacrifices combat versatility entirely. |
| **YINSH capture penalty** | — | Withdrawn ADR-002 — asymmetric when one player runs out of Guards. |
| **Rune scaling every 7 rounds** | 5a | Layer 1 accepted +2/turn with +1 every 5 rounds. Alternative cadence is moot — problem is solved. |
| **Starting player hidden Rune bid** | — | Closed OQ-45 — no first-player advantage observed. If surfaces, use Go-style komi instead. |
| **Damage escalation after Round X** | OQ-19 note | Deferred indefinitely — feels arbitrary. Checkmate win condition is the better pacing lever. |
| **3 HP for Champions/King** | OQ-18 | Scrapped before testing — would extend already-long games further. |
| **Retaliation mechanic** (overextension → opponent gets a retaliation action) | 4e | Interesting conceptually but adds a reactive rule layer. The 1 DMG nerf (Stack A) addresses the same standoff problem more elegantly. |
| **Narrative / lore** (Rabbit, Primordials, Advisor NPC) | 10a–10c | Out of scope for v1 rules/mechanics. Reserve for art/visual design phase (Phase B per timeline). |
| **Raise Undead** (10 Rune resurrection) | 3b | Extremely high Rune cost makes it non-functional in practice. Interesting in theory; catalogue bloat in v1. |
| **War Cry** (all skills +1 Range next round, 5 Runes) | 3b / 3q | Global buff lasting a full round is difficult to track physically. Focus Strike/Blade Call cover the empowerment category at manageable scope. |
| **Soul Swap** / piece swapping skills | 3a | Swap two of your own pieces. Shadow Shift already covers same-position swaps. Adding a second swap creates catalogue redundancy. |
| **Information / scouting skill** (Runenblick: reveal Rune count + steal) | 3o | Perfect information game — all information is already public. Skill has no purpose. |
| **Movement as a skill** (Schnelltritt, Escape Plan, Pferdesprung) | 3p | Already exists in current skill catalogue (Horse Leap, Shadow Shift). |
| **Free-direction push** (Federstoß) | 3i | Already exists in current skill catalogue. |
| **Class-based skill pool** (Champion class determines draftable skills) | 7f | Limits strategy freedom. Small upside (slightly simpler draft) doesn't justify restricting player choice space. |
| **Turn counter as global timer** (0.5 increments per player turn) | 6d | Originated from temp effect tracking need. Not important enough for a physical board game to track precisely — Rune count already marks progression. |
| **Full Champion roster naming** (Blacksmith, Necromancer, Bard, etc.) | 10d | Champions are blank slates by design — identity emerges from equipped skills. Pre-naming destroys "mental freedom to do anything." See `design-language.md`. |
| **Skill cost calibration** (average 2.5–3 Runes across catalogue) | 5h | Should not be math-driven. Calibrate by feel through playtesting — "never feel like you can't do stuff" vs "never feel like spending doesn't matter." |
| **Move Slot loss as debuff** (target moves one fewer piece) | 3k | Too OP — restricts opponent movement too heavily. Low priority, no current path to balanced implementation. |
