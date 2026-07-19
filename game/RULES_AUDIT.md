Pre Note: I will use this Document for future reference to know how i would need to adjust things if I make changes.

# Rules Audit

This document records the results of a per-rule compliance sweep against `design/RULES.md`. For every rule section, agents read the relevant `core_engine` source and checked two things: (1) is the rule correctly implemented, and (2) is the code structured so the rule can be adjusted without surgery?

Each clause carries a **Status** badge and an **Adjustability** rating:

- **Status:** `CORRECT` · `INCORRECT` · `PARTIAL` · `NOT_FOUND`
- **Adjustability:** `EASY` (single constant or one-line change) · `MEDIUM` (a handful of coordinated sites) · `HARD` (deeply structural, requires type/layout changes)

One actionable bug was found. It is called out inline and summarised in the [Findings](#findings) section at the end.

---

## Table of Contents

1. [Goal](#1-goal)
2. [Components](#2-components)
3. [Setup](#3-setup)
4. [Round Structure](#4-round-structure)
5. [Turn Structure](#5-turn-structure)
6. [Move Phase](#6-move-phase)
7. [Health & Armor](#7-health--armor)
8. [Move-Attack](#8-move-attack)
9. [Bodyguard Rule](#9-bodyguard-rule)
10. [Skill Phase](#10-skill-phase)
11. [Skill System](#11-skill-system)
12. [Multi-Champion Combo Bonus](#12-multi-champion-combo-bonus)
13. [Money](#13-money)
14. [Progression](#14-progression)
15. [Skill Drafting](#15-skill-drafting)
16. [Skill Reference — All 15 Skills](#16-skill-reference--all-15-skills)
17. [Findings](#17-findings)

---

## 1. Goal

### Capture condition
**Rule:** Capture the opponent's King. The game ends immediately when a King is removed from the board.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:459-486` — `deal_one_damage` detects `was_king` before clearing bitboards and immediately calls `set_game_result` with the winning player, atomic within the same `make()` call. `generator.rs:65-67` — `generate()` returns `Vec::new()` as soon as `pos.game_result.is_some()`. `session.rs:433` — `try_apply` returns `Err(ApplyError::GameOver)` before reaching legality checks when a result is set.
**Adjustability:** EASY — the win-condition check is one `was_king` branch in `deal_one_damage`. Changing the condition (e.g. "capture all Champions") requires modifying only that branch and adding the corresponding `GameResult` variant.

### No draw conditions
**Rule:** There are no draw conditions.
**Status:** CORRECT
**Evidence:** `position.rs:31-32` — `GameResult` has only `P1Wins` and `P2Wins`; no `Draw` variant exists. The comment is explicit: "Stack M has no draws ('No draw conditions'), so this enum has no Draw variant by design." No repetition detection, move-count limit, or stalemate check exists anywhere in the engine; theoretically infinite games are possible by design.
**Adjustability:** MEDIUM — adding draws (repetition rule, move-count limit) would require a new `GameResult::Draw` variant, a detection pass in `make()` or `end_turn()`, and an update to the generator gate. Currently none of that infrastructure exists.

---

## 2. Components

### Board size
**Rule:** 8x8 grid.
**Status:** CORRECT
**Evidence:** `position.rs:111` — `mailbox: [MailboxEntry; 64]`. Square addressing is `rank * 8 + file` throughout. `bitboard.rs` — `Bitboard` is a `u64` newtype; the 64-bit width is the board.
**Adjustability:** HARD — the board size is deeply structural. `Bitboard` is a `u64`, all square indices are computed as `rank * 8 + file`, the magic bitboard tables are pre-built for 8x8 geometry, and `[MailboxEntry; 64]` is the mailbox size. Changing board dimensions would require rebuilding the entire bitboard and magic-table infrastructure. There is no `BOARD_SIZE` constant — the 8x8 assumption is encoded in the type itself.

### Piece counts per player
**Rule:** 1 King · 5 Champions · 6 Guards per player.
**Status:** CORRECT
**Evidence:** `position.rs:38` — `pub const CHAMPIONS_PER_PLAYER: usize = 5`. `position.rs:360-394` — `place_back_row` iterates files `1..=6`, placing 1 King and 5 Champions; `place_front_row` iterates the same range, placing 6 Guards.
**Adjustability:** MEDIUM — `CHAMPIONS_PER_PLAYER` is a named constant but is not mechanically wired to the setup loops, which hard-code the `1..=6` file range. Guard count has no named constant at all. The draft system's `SideLoadout = [(u8,u8); 6]` is also a hard-coded array. Changing piece counts requires editing the loop bounds, the constant, and the loadout type in concert.

### Champions and King have 2 Equip Slots; Guards have none
**Rule:** Champions and the King have 2 Equip Slots each. Guards have no Equip Slots and carry no skills.
**Status:** CORRECT
**Evidence:** `mailbox.rs:7-8` — bits 7..11 hold Skill 1 ID, bits 11..15 hold Skill 2 ID — exactly 2 skill slots per entry, with 0 meaning unequipped. `make_unmake.rs:1244-1251` — `is_stm_skill_bearer()` returns true only for Kings and Champions; Guards are explicitly excluded. `position.rs:381-394` — `place_front_row` initialises all Guards with `EMPTY_MAILBOX_ENTRY.with_hp(2)`, leaving both skill slots at zero.
**Adjustability:** MEDIUM for slot count — the 2-slot constraint is encoded in the mailbox bit layout and `draft_complete()`. A 3rd slot would require a wider `MailboxEntry` and updated draft logic; there is no named slot-count constant. EASY for Guard exclusion — removing the `is_stm_skill_bearer` guard is the only code change needed to let Guards carry skills; the storage is already there (bits are always zero by construction).

**Note:** Guards technically have skill bit fields in the mailbox (bits 7..14), but they are always zero by construction. No code path writes to them. A `debug_assert` in `place_front_row` checking that a Guard's skill slots remain zero would add belt-and-suspenders safety.

---

## 3. Setup

### Back-row placement: Kings offset, 2C/3C split, 2 free squares
**Rule:** The King stands in the middle of the back row, offset so the two Kings are not directly opposite each other. On one side stand 2 Champions; on the other stand 3. Two free squares remain at each end.
**Status:** CORRECT
**Evidence:** `position.rs:246-249` — P1 King placed at file 3 (d1), P2 King at file 4 (e8); the two kings are not on the same file. P1 therefore has Champions on files 1, 2 (left) and 4, 5, 6 (right); P2 on files 1, 2, 3 (left) and 5, 6 (right). Both setup calls iterate `1..=6`, leaving files 0 and 7 empty — two free squares per side. `position.rs:362` — `debug_assert!((1..=6).contains(&king_file))` enforces the constraint at debug time.
**Adjustability:** MEDIUM — the specific King files (3 and 4) are hard-coded as call-site literals in `setup_stack_m()`. There is no programmatic check that the two Kings are on different files; the invariant is expressed solely through the chosen constants. A code comment notes a parameterised variant (`setup_stack_m_with`) is planned.

### Second-row placement: 1 Guard in front of each back-row piece
**Rule:** One Guard directly in front of each Champion and the King.
**Status:** CORRECT
**Evidence:** `position.rs:247-248` — `place_front_row` is called on rank 1 (P1) and rank 6 (P2), directly adjacent to the back rows at ranks 0 and 7. The function iterates files `1..=6` — identical to the back-row range — placing one Guard per file. Every back-row occupant therefore has a Guard directly ahead.
**Adjustability:** EASY — changing which files get Guards means editing the `place_front_row` loop bounds. The current 1:1 correspondence with the back-row range is mechanically sound.

### Skill Draft runs before play begins
**Rule:** Skill Draft takes place after piece placement; play begins once all slots are filled.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1216-1219` — `apply_end_phase` transitions from `Phase::Draft` to `Phase::Move` only when `draft_complete()` returns true. `draft_complete()` iterates every King and Champion and returns false if any slot is unequipped. The engine cannot be advanced to the Move Phase until all 12 skill slots per player are filled.
**Adjustability:** EASY — the completion check is a single predicate loop.

### Starting Money: 6 per player
**Rule:** Each player starts with 6 Money.
**Status:** CORRECT
**Evidence:** `position.rs:254-255` — `p.p1_money = 6; p.p2_money = 6` in `setup_stack_m()`. The `session.rs` test `new_match_starts_at_stack_m` asserts both values equal 6.
**Adjustability:** EASY — two literal assignments. There is no named `STARTING_MONEY` constant; extracting one would be a single-line change.

### P1 begins Round 1
**Rule:** P1 begins Round 1.
**Status:** CORRECT
**Evidence:** `position.rs:251, 253-254` — `p.to_move = Player::P1`, `p.round_number = 1`, `p.current_phase = Phase::Draft` (advancing to `Phase::Move` once the draft completes), `p.actions_remaining = 2`.

---

## 4. Round Structure

### Round = P1's turn + P2's turn
**Rule:** A Round = P1's Turn + P2's Turn.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:71-74` — `flip_to_move` flips the active player; when the new side-to-move is P1 (meaning P2's turn just ended), `round_number` is incremented via `set_round`. One full round is therefore the P1→P2→P1 flip cycle.
**Adjustability:** MEDIUM — the round increment is tied to the P2→P1 flip in a single conditional. Changing to a three-player cycle would require restructuring this logic, but the current 2-player semantics are expressed in one `if matches!(new_stm, Player::P1)` guard.

### Income collected at the start of each player's own turn
**Rule:** At the start of each player's turn, that player collects their Money income.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:76-88` — income is disbursed inside `end_turn()`, after the round number is bumped, to the new side-to-move (the player whose turn is beginning). P1 ends → income fires for P2's incoming turn; P2 ends → income fires for P1's incoming turn.
**Adjustability:** EASY — the disbursement is a single block inside `end_turn()`.

### Round 1 exception: no income before first turns
**Rule:** Round 1 — players begin with their starting Money and collect nothing before their first turn.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:82` — income disbursal is gated by `if pos.round_number >= 2`. The round number only reaches 2 after P2's Round-1 turn ends (via the P2→P1 flip), so neither player receives income during Round 1. Starting money of 6 is set directly in `setup_stack_m()`.
**Adjustability:** EASY — the constant `2` in the guard is the only change point.

---

## 5. Turn Structure

### Two phases in order: Move then Skill
**Rule:** Each Turn has two phases in order: Move Phase (move pieces and attack), then Skill Phase (activate skills).
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1138-1152` — `apply_end_phase` transitions `Phase::Move` → `Phase::Skill`, and `Phase::Skill` → calls `end_turn` (which flips sides and starts the next turn's Move Phase). There is no legal path that skips Skill Phase or reverses the order.
**Adjustability:** EASY — the two-phase sequence is a single `match` in `apply_end_phase`. Adding or reordering phases requires changes there and in `generator.rs`'s dispatch.

### Zero actions in either phase is legal
**Rule:** You may use 0 actions in either phase.
**Status:** CORRECT
**Evidence:** `generator.rs:94-98` — when `actions_remaining == 0` in the Move Phase, the generator returns only `[EndPhase]`. `generator.rs:197-200` — the same early-return applies in the Skill Phase. A player can always skip either phase entirely by immediately spending `EndPhase`.

---

## 6. Move Phase

### 2 actions per Move Phase
**Rule:** You have 2 actions per turn.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:92` — `set_actions(pos, undo, 2)` at each `end_turn`. `generator.rs:94-98` — when `actions_remaining == 0`, only `EndPhase` is legal.
**Adjustability:** EASY — a single `set_actions(…, 2)` call. The constant is not named.

### 1 action per move or Move-Attack
**Rule:** Spend 1 action to move one piece.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1399-1401` — `dec_actions` is called at the end of every move application, decrementing by exactly 1. For Move-Attacks with a pending Bodyguard choice, `dec_actions` is deferred to `apply_bodyguard_choice` but still fires exactly once.
**Adjustability:** EASY — the decrement is implicit (always -1). Variable-cost moves would require replacing the call-site constant.

### Each piece may only be moved once per Move Phase
**Rule:** Each piece may only be moved once per Move Phase.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:176` — `moved_set(pos, undo, tgt)` marks the destination square after every plain move. `make_unmake.rs:363` — same for Move-Attacks. `generator.rs:111` — `let movable = stm_bb & !pos.moved_this_phase` — only pieces not already marked are included. `turn_manager.rs:93` / `make_unmake.rs:1141` — `moved_clear_all` resets the bitmask at end of Move Phase and again defensively at end of turn.
**Adjustability:** MEDIUM — the "once per phase" invariant is a bitboard and a filter. Changing to "twice per phase" would require a per-piece counter rather than a single-bit flag.

**Note:** `position.rs:135-140` describes `moved_this_phase` as tracking origin squares, but the implementation stores destination squares (`tgt`). The filter at `generator.rs:111` operates on current square (which equals the destination after moving), so the logic is correct. The struct comment is stale.

### Movement speeds: Guard = 2, Champion/King = 1
**Rule:** Guard speed = 2 tiles; Champion/King speed = 1 tile.
**Status:** CORRECT
**Evidence:** `generator.rs:612-614` — `piece_speed` returns `2` for Guards and `1` for everything else. `generator.rs:484-487` — `reachable` dispatches `movement_targets_speed1` (8-neighbour) for speed 1 and `movement_targets_speed2` (Chebyshev BFS-2 flood-fill) for speed 2.
**Adjustability:** EASY — `piece_speed` is a two-line function. Adding a higher speed would require a matching `movement_targets_speedN` in `magic.rs`.

### Free pathing: any direction, no jumping, zig-zag legal
**Rule:** A piece may move in any direction taking any route up to its speed. The route need not be a straight line. Jumping over pieces is not allowed.
**Status:** CORRECT
**Evidence:** `magic.rs:479-486` — `movement_targets_speed2` is a two-step flood-fill: step-1 empties reachable from the origin, step-2 empties reachable from the origin or any step-1 square. Zig-zag is inherent in the flood approach. `generator.rs:720-737` — the test `guard_diagonal_bypass_around_single_blocker` explicitly verifies a Guard can bypass a blocker via a diagonal zig-zag. The BFS never adds occupied squares to the reachable set, enforcing the no-jumping rule.
**Adjustability:** EASY — the flood-fill naturally supports free pathing. Restricting to straight lines would require replacing the flood with a ray-casting approach.

---

## 7. Health & Armor

### 2 HP per piece
**Rule:** All pieces have 2 HP.
**Status:** CORRECT
**Evidence:** `mailbox.rs:4` — HP stored in bits 0..2 of `MailboxEntry`, capacity 0-3 (max 2 enforced by `debug_assert!(hp <= 2)`). `make_unmake.rs:548` — `const FULL_HP: u8 = 2`. `position.rs:363, 382` — every piece initialised with `.with_hp(2)` in both `place_back_row` and `place_front_row`.
**Adjustability:** EASY — `FULL_HP` is a named constant. The 2-bit field supports values 0-3, so the cap could increase to 3 without a layout change; beyond that the bit width would need widening.

### Maximum 2 Armor per piece
**Rule:** A maximum of 2 points of Armor.
**Status:** CORRECT
**Evidence:** `mailbox.rs:5` — Armor in bits 2..4. `mailbox.rs:29-31` — `debug_assert!(a <= 2, "Stack M armor cap is 2")`. `make_unmake.rs:548` — `const ARMOR_CAP: u8 = 2`. `generator.rs:57` — same constant used to filter out Shield/Plate actions on a piece already at cap.
**Adjustability:** EASY — `ARMOR_CAP` is a named constant declared at two sites (`make_unmake.rs:548` and `generator.rs:57`). The underlying 2-bit field has one spare value, so a cap of 3 requires only updating the constant; a cap of 4 or above requires a wider field.

### Armor absorbs before HP; each point is destroyed on use
**Rule:** Each armor point absorbs 1 damage, then is destroyed. Armor resolves before HP damage.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:448-455` — `deal_one_damage` checks `if prev_entry.armor() > 0` first; if true, decrements armor by 1 and returns immediately — HP is never touched. The HP-reduction path is only reached when armor is zero. The unit test `move_attack_burns_armor_first` (`make_unmake.rs:1602`) directly verifies this order.
**Adjustability:** EASY — the priority is a single `if/else` branch; the decrement is `armor() - 1` with no further condition.

### A piece can carry Armor while below full HP
**Rule:** A piece can have Armor but not be at full health.
**Status:** CORRECT
**Evidence:** HP and Armor are stored in independent bit fields of `MailboxEntry` (bits 0..2 and 2..4). No code path couples the two: `deal_one_damage` modifies exactly one field per call, `apply_heal` restores HP without touching Armor, and `apply_shield`/`apply_plate` add Armor without touching HP. No assertion anywhere prevents Armor > 0 when HP < 2.
**Adjustability:** EASY — the orthogonal storage means this invariant holds structurally; no code change required.

---

## 8. Move-Attack

### Deals 1 damage
**Rule:** Deal 1 damage to the enemy.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:323` — `deal_one_damage(pos, hit_sq, undo)` is the sole damage call in the direct (no Bodyguard) branch of `apply_move_attack`. `make_unmake.rs:397` — same call in `apply_bodyguard_choice`. `deal_one_damage` applies exactly one point of damage per invocation.
**Adjustability:** EASY — the single call is the only change point; no numeric constant is involved.

### Only 1 Move-Attack per turn
**Rule:** You only have 1 Move-Attack per Turn.
**Status:** CORRECT
**Evidence:** `position.rs:96` — `modifier_bits::MOVE_ATTACK_USED = 1 << 2`. `make_unmake.rs:245-247` — `MOVE_ATTACK_USED` is set at the start of `apply_move_attack`, on both the tentative and direct paths. `generator.rs:107-108, 136` — once the flag is set, the entire Move-Attack generation block is skipped. The flag is cleared by `end_turn`'s blanket `pending_modifiers = 0` and is intentionally not cleared at the Move→Skill phase transition, so a Move-Attack cannot be reclaimed by ending the Move Phase early.
**Adjustability:** EASY — flipping the `if !move_attack_used` guard or removing the flag removes the cap. Raising the cap to N > 1 would require a counter rather than a bit.

### Enemy survives: attacker stops on the tile immediately before the target
**Rule:** If the enemy survives, your piece stops on the tile immediately before the target.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:264-296` — the first hop physically moves the attacker to `approach_sq` (the penultimate tile). `make_unmake.rs:328-361` — when `defender_died` is false, `attacker_final = approach`; the attacker stays there and never enters the target tile. The comment at line 213-215 documents the speed-1 case explicitly: "speed-1 attackers have `approach_sq == src`, so no physical relocation occurs — damage is still dealt."
**Adjustability:** MEDIUM — the approach square is encoded in action bits and threaded through `apply_move_attack`; changing the stop distance requires updating both the action encoding and the generator's approach-square enumeration.

### Speed-1 attacker (Champion/King) does not move on a non-kill
**Rule:** A Champion or King with speed 1 does not move at all. Damage is still dealt.
**Status:** CORRECT
**Evidence:** For speed-1 pieces the generator encodes `approach_sq == src`. `make_unmake.rs:264` — the first-hop block is guarded by `if approach != src`, so it is a no-op. `make_unmake.rs:359` — `attacker_final = approach = src`; the piece stays exactly where it started. The unit test `move_attack_burns_armor_first` (`make_unmake.rs:1613`) confirms the attacker has not relocated after the strike.
**Adjustability:** EASY — falls out naturally from `approach == src`; no special-case code.

### Speed-2 Guard ends exactly 1 tile from its origin on a non-kill
**Rule:** A Guard with speed 2 ends up having moved only 1 tile.
**Status:** CORRECT
**Evidence:** `generator.rs:136-147` — approach squares are enumerated as neighbours of the target with `dist[approach] <= speed - 1`, meaning a Guard's approach is at most 1 step from its origin. `make_unmake.rs:264-296` — the Guard is physically relocated from `src` to that approach square. On a non-kill, `attacker_final = approach`, which is exactly 1 tile away from `src`.
**Adjustability:** MEDIUM — tied to the BFS `dist` array and the `speed - 1` threshold in the generator.

### Enemy removed: attacker occupies the vacated tile
**Rule:** If the enemy is removed, your piece occupies that tile. Remaining unused movement tiles do not carry over.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:328-361` — when `defender_died` is true, a second hop executes: the attacker is written to `tgt` and `approach` is cleared. `make_unmake.rs:400-435` — the same logic applies in the Bodyguard resolution path, but only when `idx == 0` (the named target died, not a redirected Guard). No provision exists to continue moving after the kill; the action is consumed entirely.
**Adjustability:** EASY — the second-hop block is self-contained; suppressing it with a flag would trivially remove kill-follow-through.

### Multiple approach paths: attacker chooses (Bodyguard relevance)
**Rule:** If multiple paths reach the target, you choose which — this is important for the Bodyguard Rule.
**Status:** CORRECT
**Evidence:** `generator.rs:136-147` — for each `(src, tgt)` pair the generator emits one action per distinct valid `approach_sq`. Each distinct approach produces a different set of eligible Guards (dual-adjacency check in `bodyguard_guards_for`), so the choice of approach is mechanically meaningful. `generator.rs:121-125` — the comment explicitly names the zig-zag bypass: choosing a different approach can sidestep Bodyguard coverage. The test `bodyguard_zigzag_bypass_yields_empty_set` (`generator.rs`) confirms a different approach eliminates an otherwise eligible Guard.
**Adjustability:** MEDIUM — the approach enumeration is woven through the generator loop and the action encoding.

---

## 9. Bodyguard Rule

### Trigger: Move-Attack against a Champion or King only
**Rule:** When you make a Move-Attack against an opponent's Champion or King, the defender may choose to have a Guard intercept.
**Status:** CORRECT
**Evidence:** `generator.rs:563-569` — `bodyguard_guards_for` immediately returns an empty list if the target is a Guard (`pos.guards.contains(target_sq)`), then checks `pos.champions.contains(target_sq) || pos.kings.contains(target_sq)` and returns empty if neither condition holds. The function is only ever called from `apply_move_attack`. The test `move_attack_on_guard_offers_no_bodyguard_choice` (`make_unmake.rs:946-967`) directly verifies Guard targets trigger no pending state.
**Adjustability:** EASY — the two-line piece-type filter in `bodyguard_guards_for` (lines 565-569) is a straightforward bitboard membership check. Adding or removing piece types from protection is a one-line edit.

### Dual-adjacency condition
**Rule:** Interception is possible only if a friendly Guard is adjacent to both the tile immediately before the target (along the attack path) and the defending piece itself.
**Status:** CORRECT
**Evidence:** `generator.rs:575-590` — `bodyguard_guards_for` computes candidates as `movement_targets_speed1(target_sq) ∩ defender_bb ∩ guards` (Guards adjacent to the defender), then filters to those also present in `movement_targets_speed1(approach_sq)` (adjacent to the approach square). Both adjacency checks use the Chebyshev king-move neighbourhood. The tests `bodyguard_dual_adjacency_filters_to_intersection` and `bodyguard_zigzag_bypass_yields_empty_set` directly verify the intersection arithmetic. The integration test `move_attack_zigzag_bypass_chooses_clean_approach` (`make_unmake.rs:2159-2219`) confirms a differently-chosen approach square removes an otherwise eligible Guard.
**Adjustability:** EASY — the filter is two bitboard intersections on lines 581 and 584, cleanly separated from the two-ply transaction.

### Two-ply transaction: tentative apply then defender choice
**Rule:** The defender announces a Guard to intercept. The Guard takes the damage instead of the original target.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:298-318` — when eligible Guards exist, `apply_move_attack` executes a tentative apply: moves the attacker to `approach_sq`, stores `PendingBodyguard` in `pos.pending_bodyguard`, flips STM to the defender, and signals the caller to defer `dec_actions`. `generator.rs:86-92` — while `pending_bodyguard` is `Some`, the generator emits only `BodyguardChoice` actions. `make_unmake.rs:374-443` — `apply_bodyguard_choice` reads the pending state, resolves damage on the chosen square, clears `pending_bodyguard`, flips STM back, and calls `dec_actions`. The `Undo` struct captures `prev_pending_bodyguard`, so the full two-ply transaction is reversible in the search tree. Tests `make_unmake_roundtrip_bodyguard_guard_saves_king` (`make_unmake.rs:1887`) and related tests verify the complete flow.
**Adjustability:** MEDIUM — the split between `apply_move_attack` and `apply_bodyguard_choice` is intentional. Adding a third ply (e.g. attacker reacts to the interception) would require touching both halves, the generator guard, the `Undo` struct, and Zobrist keys.

### Attacker moves only 1 tile when attacking from range 2, regardless of Guard death
**Rule:** When the attack was made from a range-2 square, the attacker moves only 1 tile toward the target, no matter if the bodyguard dies or not.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:264-295` — the first hop always moves the attacker from `src` to `approach_sq` (one tile from the target). `make_unmake.rs:399-435` — kill-follow-through in `apply_bodyguard_choice` is gated by `idx == 0 && !pos.is_occupied(tgt)`: when a Guard is chosen (`idx != 0`), `attacker_final = approach` regardless of whether the Guard dies. The attacker always stays at the approach square when a Bodyguard resolves. The test `move_attack_with_three_adjacent_guards_emits_four_bodyguard_choices` (`make_unmake.rs:1987`) confirms the attacker is on the approach tile and not on the target tile after a Guard intercept.
**Adjustability:** EASY — the `idx == 0` gate is a single boolean condition; changing it to allow kill-follow-through on Guard death would be a one-line edit.

### Optionality and multi-Guard choice
**Rule:** Interception is optional. The defender may decline even if a Guard is eligible, and may choose which Guard intercepts if multiple are eligible.
**Status:** CORRECT
**Evidence:** `generator.rs:87-90` — `Action::encode_bodyguard_choice(0)` (decline) is always the first action emitted. One additional action per eligible Guard follows. `make_unmake.rs:384-388` — `idx == 0` routes damage to the original target (decline); `idx > 0` routes it to `eligible[idx-1]`. The eligible list is sorted in ascending square order for a canonical index mapping. The test `generates_move_attack_with_bodyguard_choices` (`generator.rs:1001`) asserts exactly 3 choices (decline + 2 Guards) and verifies their indices.
**Adjustability:** EASY — the multi-Guard selection is a simple array index. `MAX_BODYGUARD_ELIGIBLE = 4` (`position.rs:65`) reflects the geometric maximum of two overlapping king-move neighbourhoods.

### Skills bypass Bodyguard entirely
**Rule:** Only Move-Attacks can be intercepted. Skills always hit directly.
**Status:** CORRECT
**Evidence:** No skill resolver in `make_unmake.rs:506-543` calls `bodyguard_guards_for` or sets `pending_bodyguard`. The `ActionKind::Skill` dispatch path in `make()` calls `apply_skill` directly with no Bodyguard hook. The invariant is also structural: `pending_bodyguard` can only be set inside `apply_move_attack` and only cleared by `apply_bodyguard_choice`. While `pending_bodyguard` is `Some`, the generator restricts the legal move list to `BodyguardChoice` actions only (`generator.rs:86-92`), making it impossible for a skill action to fire mid-transaction.
**Adjustability:** EASY — the bypass is passive; no code exists to intercept skills. Making a skill intercept-able would require adding a dedicated hook from scratch.

---

## 10. Skill Phase

### Action budget starts at 2, grows with Progression
**Rule:** You have 2 actions per turn at the start (grows with Progression).
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1161-1163` — `skill_phase_budget(round_number)` returns `2 + (round_number - 1) / 10`. Applied at `make_unmake.rs:1143` when `EndPhase` transitions Move → Skill. The full Progression table is verified in §14.
**Adjustability:** EASY — single function, one formula line.

### 1 action per skill activation
**Rule:** Spend 1 action to activate one equipped skill on one of your Champions or King.
**Status:** CORRECT
**Evidence:** Every skill resolver ends with a call to `dec_actions`, decrementing `actions_remaining` by exactly 1. `generator.rs:197-199` — when `actions_remaining == 0`, only `EndPhase` is emitted.
**Adjustability:** EASY — all resolvers share the same tail-call pattern.

### Pay the skill's Money cost before applying the effect
**Rule:** Pay the skill's Money cost.
**Status:** CORRECT
**Evidence:** Every resolver calls `debit_money(pos, src, cost, undo)`. `generator.rs:217` — `if skill_cost(skill) as u16 > money { continue }` — an action is never emitted for a skill the player cannot afford.
**Adjustability:** EASY — each skill cost is a single match arm in `skills.rs:81-99`.

### Same Champion may activate multiple skills in one turn
**Rule:** The same Champion can activate multiple skills (including the same skill) in one turn if actions and Money remain.
**Status:** CORRECT
**Evidence:** The Skill Phase generator at `generator.rs:213` iterates all STM pieces with no "already-used-this-phase" filter analogous to `moved_this_phase`. The only stacking constraints are Focus-on-Focus and Charge-on-Charge (`generator.rs:221-222`), which are 1-stack rules for those specific Mystic skills, not per-caster limits.
**Adjustability:** EASY — the absence of a per-caster gate is what permits multiple activations; adding a limit would require a new per-piece flag.

---

## 11. Skill System

### Path: straight line (queen-style) from caster
**Rule:** Skills travel in a straight line (horizontal, vertical, or diagonal) from the caster, like a chess Queen.
**Status:** CORRECT
**Evidence:** `magic.rs:312-330` — `skill_attacks` ORs a rook-magic result (orthogonal rays) and a bishop-magic result (diagonal rays) to obtain queen reach from the source square. `path.rs:22-28` — `skill_targets` calls `skill_attacks` and intersects with occupancy to return the first blocker on each ray.
**Adjustability:** EASY — the magic-table infrastructure is well-tested and robust.

### Path blocked by all pieces; cannot reach past the first piece
**Rule:** The path is blocked by all pieces, ally and opponent alike. A skill cannot reach past the first piece in its path.
**Status:** CORRECT
**Evidence:** `path.rs:23` — the occupancy mask is `(pos.p1_pieces | pos.p2_pieces).0` — the union of both sides. `magic.rs:197-213` — `slider_reference_attacks` (used at table-build time) walks each ray and stops at the first set bit in the occupancy mask, inclusive. `path.rs:27` — intersecting with occupancy drops empty squares, returning only the first blocking square on each ray.
**Adjustability:** EASY — a fundamental property of the magic-table approach; not subject to per-skill configuration.

### Default Range = 2; Range 0 = self; Range 1 = adjacent
**Rule:** Default Range = 2 unless a skill explicitly says "self" or "adjacent." Range 0 = caster's tile, Range 1 = adjacent tile.
**Status:** CORRECT
**Evidence:** `skills.rs:110-128` — `skill_default_range` returns 0 for Shield/Focus/Charge, 1 for Lance/Heal/Plate, 2 for the remaining base-range skills, and 3 for Shove/Retreat (their +1 is baked in rather than applied dynamically). `magic.rs:90-113` — `within_range_table` builds Chebyshev distance rings 1..=r, so range is in tiles.
**Adjustability:** EASY — one match arm per skill.

### Focus gives +1 Range to the next non-Mystic skill
**Rule:** Range buffs shift the targeting window outward.
**Status:** CORRECT
**Evidence:** `generator.rs:228-231` — `let range = if focus_pending && !is_mystic { base_range + 1 } else { base_range }`. Applied universally to all non-Mystic skills regardless of their base range. `make_unmake.rs:522-525` — `apply_skill` clears the FOCUS bit for non-Mystic skills only, so a Mystic cast does not consume the pending Focus.
**Adjustability:** EASY — a single conditional in the generator.

**Design note:** The rules say "Range modifiers apply to base-2 skills" but also give the example "Self + Focus → Range 1," implying Focus applies to all non-Mystic skills including Range-0 and Range-1 ones. The code implements the broader interpretation (Focus buffs all non-Mystic skills). The rule text contains a mild internal contradiction; the code resolves it consistently with the stated example. A written ruling to close this would eliminate the ambiguity.

### Self skills target only the caster; adjacent skills never target the caster
**Rule:** Self skills target only the caster's own tile. Adjacent skills target only neighbouring pieces — never the caster, even with a Range buff.
**Status:** CORRECT
**Evidence:** `generator.rs:234-263` — `TargetOwner::SelfOnly` emits exactly one action with `src == tgt`. For adjacent skills (Range 1), `skill_attacks` never returns the source square itself (a ray starting at a square never includes that square), so the caster cannot appear as a target by construction.
**Adjustability:** EASY — the self-target constraint is structural; no special exclusion filter is needed.

### Range buffs shift the window outward, not inward
**Rule:** Range buffs do not collapse adjacent skills inward toward Self.
**Status:** CORRECT
**Evidence:** `generator.rs:231` — the formula is strictly `base_range + 1`; there is no lower-bound clamp that could push the range down. Since `skill_attacks` never includes the source square, a Focus-boosted Range-1 skill targets Range-2 squares, not Range-0. No test explicitly asserts this no-collapse invariant; it holds by construction.
**Adjustability:** EASY — the formula has no minimum guard; adding one would be the only change needed to alter this behaviour.

### Move-skills deal no damage on arrival
**Rule:** Movement-via-skill (Dash, Swap, Retreat) is not a Move-Attack and deals no damage on arrival. (Exception: Combo Bonus still applies.)
**Status:** CORRECT
**Evidence:** `make_unmake.rs:695-712` (`apply_dash`), `854-868` (`apply_retreat`), `812-851` (`apply_swap`) — all call `relocate_piece` only, with no call to `deal_one_damage` or `deal_damage`. The Combo Bonus is intentionally preserved for enemy-targeting push skills (Blast, Shove) via their dedicated `combo_tick` calls, consistent with the rules exception.
**Adjustability:** EASY — the no-damage invariant is structural; damage would have to be added explicitly to these resolvers.

### Strike skill step-forward: caster moves 1 tile toward the target after resolution
**Rule:** After a Strike skill's damage and effect fully resolve, the caster moves 1 tile toward the target along the cast direction.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1019-1049` — `strike_move_caster` uses `magic::step_toward(caster_sq, target_sq)` and moves the caster only when the destination is empty. Called as the last step in every Strike resolver: Lance (`make_unmake.rs:557`), Break (`594`), Steal (`609`), Hook (`626`), Tempest (`653`). In all cases the call sequence is: damage/effect → `debit_money` → `strike_move_caster` → `dec_actions`.
**Adjustability:** EASY — a single well-isolated function called consistently by all five Strike resolvers.

---

## 12. Multi-Champion Combo Bonus

### Per-enemy combo counter, starts at 0
**Rule:** Each enemy piece has a combo counter that starts at 0.
**Status:** CORRECT
**Evidence:** The combo counter lives in bits 4..7 of each piece's `MailboxEntry` (3 bits, max 7). `EMPTY_MAILBOX_ENTRY` initialises all fields to 0. An independent `tracked_enemies` / `tracked_casters` / `champion_credit` structure in `Position` (`position.rs:151-164`) records which casters have already contributed to which targets within the current turn.
**Adjustability:** EASY — counter width (3 bits, max 7) is a mailbox-encoding constant; the counter is automatically zeroed at setup.

### Counter resets at end of turn
**Rule:** The combo counter resets at the end of your turn.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:46-53` — iterates every occupied square on both sides at end-of-turn, writes `.with_combo(0)` for any non-zero entry via the Zobrist-aware `write_mailbox`. `turn_manager.rs:62-68` — also clears `champion_credit`, `tracked_enemies_len`, and `tracked_casters_len`. The comment on line 20 explains why both sides are cleared: Tempest can tick combo on friendly pieces in the same turn, and that state must not persist into the opponent's turn.
**Adjustability:** EASY — single loop and three field-clear statements.

### "New Champion" gate: each caster can only tick a given target once per turn
**Rule:** The counter ticks only when a new Champion — one that has not already incremented this counter this turn — performs the triggering action.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:972-1017` — `combo_tick` calls `ensure_tracked_caster` and `ensure_tracked_enemy` to assign indices, then checks the `champion_credit` bitmask. If the (caster, target) bit is already set, the function returns `false` (no tick). `relocate_piece` (`make_unmake.rs:1090-1096`) updates both tracking arrays when a piece moves mid-turn, so a pushed/pulled piece that is subsequently retargeted remains correctly deduped.
**Adjustability:** EASY — the gate is a single bit in a `u128` bitmask.

### Bonus damage formula: counter - 1
**Rule:** Any skill that affects a target with a combo counter > 0 deals damage equal to counter - 1.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:934-952` — `apply_strike_damage` snapshots `existing_combo` before calling `combo_tick`. For a new caster (tick fires, counter advances), the bonus is `existing_combo` (pre-tick value = counter - 1 after increment). For a returning caster (tick does not fire), the bonus is `existing_combo.saturating_sub(1)`. Both cases produce the same value a naive "counter - 1" formula would give. The test `combo_returning_caster_bonus_is_counter_minus_one` directly verifies the same-caster-twice case yields a bonus of zero.
**Adjustability:** EASY — the formula is an arithmetic expression in three resolver sites (strike, Blast, Shove).

### A skill that both hits and moves the target ticks the counter only once
**Rule:** A single skill that both hits and moves the target ticks the counter only once; bonus damage is applied once.
**Status:** CORRECT
**Evidence:** Hook calls `apply_strike_damage` (which owns the single `combo_tick` call) and then applies the pull — no second `combo_tick` on the pull step. Tempest calls `apply_strike_damage` for the primary target and then `combo_tick` once per pushed neighbour in the AOE loop; the primary target is not re-ticked as a "moved target" because Tempest does not move the target itself (only its neighbours). The single-tick invariant is enforced by always routing through `apply_strike_damage` for the hit, never duplicating the `combo_tick` call in the move path of the same resolver.
**Adjustability:** EASY — the invariant is maintained by convention; a code comment could make it explicit.

### Move-Attacks do not tick the counter
**Rule:** Move-Attacks do not count.
**Status:** CORRECT
**Evidence:** `combo_tick` is only called from `apply_strike_damage`, `apply_blast`, `apply_shove`, and the Tempest AOE loop — all Skill Phase resolvers. The Move Phase paths (`apply_move_attack`, `apply_bodyguard_choice`, `apply_plain_move`) contain no `combo_tick` calls.
**Adjustability:** EASY — the exclusion is structural; Move Phase and Skill Phase code paths are cleanly separate.

### Pushing a friendly piece does not tick the counter
**Rule:** Pushing a friendly piece does not count.
**Status:** CORRECT
**Evidence — Shove:** `make_unmake.rs:785-791` — `combo_tick` is only called inside an `if target_is_enemy` block. The test `shove_pushes_ally_does_not_tick_combo` directly confirms no tick on an ally target.
**Evidence — Blast:** Blast is generator-constrained to enemy targets only (`TargetOwner::Enemy`); no friendly-push path exists.
**Evidence — Tempest:** `make_unmake.rs` (Tempest AOE loop) — `combo_tick` is guarded by the same `caster_is_p1` / `pushed_is_enemy` pattern used by Shove; the call is skipped for friendly pieces. The `relocate_piece` call for the push itself is unconditional, so friendly pieces are still moved correctly.
**Adjustability:** EASY — the enemy-only guard is a single `if pushed_is_enemy` block in the AOE loop, matching the analogous structure in `apply_shove`.

### Self-movement, pure buffs, and pure heals do not tick the counter
**Rule:** Self-movement, pure buffs, and pure heals do not count.
**Status:** CORRECT
**Evidence:** `apply_dash` (`make_unmake.rs:695-711`), `apply_retreat` (`854-868`), `apply_swap` (`812-851`) — all call `relocate_piece` only, with the comment "No combo-tick (self/ally movement)." `apply_shield`, `apply_plate`, `apply_heal` — no `combo_tick` calls.
**Adjustability:** EASY — the exclusion is structural.

---

## 13. Money

### Starting Money: 6 per player
**Rule:** Starting Money: 6 per player.
**Status:** CORRECT
**Evidence:** `position.rs:254-255` — `p.p1_money = 6; p.p2_money = 6` in `setup_stack_m()`. Asserted by the `session.rs` test `new_match_starts_at_stack_m`.
**Adjustability:** EASY — two literal assignments. No named `STARTING_MONEY` constant exists; extracting one is a one-line change.

### Income collected at the start of each player's own turn
**Rule:** Money income is collected at the start of each player's own turn.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:76-88` — income is disbursed to the new side-to-move at the end of the outgoing player's turn, which is structurally identical to "the start of the incoming player's turn." P1 ends their turn → income fires for P2; P2 ends → income fires for P1.
**Adjustability:** EASY — single block in `end_turn()`.

### No Money cap
**Rule:** There is no Money cap.
**Status:** CORRECT
**Evidence:** `position.rs:114-115` — `p1_money: u16`, `p2_money: u16` — the type does not cap at any game-rule value. `make_unmake.rs:1447-1458` — `set_p1_money`/`set_p2_money` assign the new value directly. `turn_manager.rs:85` — income uses `saturating_add` only as a defensive overflow guard against `u16::MAX`, not a game rule.
**Adjustability:** EASY — adding a cap requires a `.min(cap)` in `set_p1_money`/`set_p2_money`.

### Income formula matches the rules table
**Rule:** R1 = 0; R2-4 = +2; R5-9 = +3; R10-14 = +4; R15+ = +5, +1 every 5 rounds.
**Status:** CORRECT
**Evidence:** `turn_manager.rs:103-105` — `income_per_turn(r) = 2 + r / 5` (integer division). Verification against the rules table:

| Round | Formula result | Rules | Match |
|-------|---------------|-------|-------|
| 1 | 2 (suppressed by R ≥ 2 guard) | 0 | ✓ |
| 2-4 | 2 + 0 = 2 | +2 | ✓ |
| 5-9 | 2 + 1 = 3 | +3 | ✓ |
| 10-14 | 2 + 2 = 4 | +4 | ✓ |
| 15-19 | 2 + 3 = 5 | +5 | ✓ |
| 20+ | 2 + 4 = 6, unbounded | +1/5r | ✓ |

**Adjustability:** EASY — a single `#[inline]` function. Changing the progression curve is a one-liner.

**Note:** The function's own doc comment lists "R1-4: 2" as a row without mentioning the separate `round_number >= 2` disbursement guard in the caller. The function's return value for R1 is technically 2, but no income is actually disbursed in R1. This is a minor documentation imprecision, not a bug.

---

## 14. Progression

### Skill Phase action budget grows every 10 rounds
**Rule:** R1-10 = 2 actions; R11-20 = 3; R21-30 = 4; R31+ = 5 (+1 every 10 rounds).
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1161-1163` — `skill_phase_budget(round_number) = 2 + (round_number - 1) / 10`. Verification against the rules table:

| Round | `(r-1)/10` | Formula result | Rules | Match |
|-------|-----------|---------------|-------|-------|
| 1 | 0 | 2 | 2 | ✓ |
| 10 | 0 | 2 | 2 | ✓ |
| 11 | 1 | 3 | 3 | ✓ |
| 20 | 1 | 3 | 3 | ✓ |
| 21 | 2 | 4 | 4 | ✓ |
| 30 | 2 | 4 | 4 | ✓ |
| 31 | 3 | 5 | 5 | ✓ |
| 41 | 4 | 6 | 6 (unbounded) | ✓ |

Applied at `make_unmake.rs:1143` when `EndPhase` transitions Move → Skill. The `saturating_sub(1)` ensures the formula is 1-indexed (R1 yields tier 0, not R0).
**Adjustability:** EASY — single function, one formula line. The doc comment correctly notes that "R31+: 5" in the rules table is a cut-off, not a hard cap; the formula is unbounded.

---

## 15. Skill Drafting

### Alternating draft: P1 picks 2, then P2 picks 2, repeat
**Rule:** P1 picks 2 skills and assigns them freely, then P2 picks 2, and so on.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1211-1212` — `flip_to_move` is called at the end of every `apply_draft_turn`, alternating sides. `draft.rs:270-278` — the integration test `preset_drives_draft_to_completion` asserts exactly 12 DraftTurn plies (6 per side) drain the draft completely.
**Adjustability:** EASY — the alternation is a single `flip_to_move` call. Changing picks-per-turn would require updating the `Action` encoding that packs two `(skill_id, sq, slot)` tuples per DraftTurn.

### Both players draft from the same shared pool
**Rule:** Both players draft from the same pool. You can draft the same skill as often as you wish.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1297` — `for sk in 1u8..=15u8` iterates the full skill set on every pick, with no cross-side depletion tracking. The same skill can be chosen by either player at any time, and can appear on multiple different pieces for the same player.
**Adjustability:** EASY — adding shared-pool depletion would require a new pool bitset on `Position`. Currently none exists.

### The same skill may not occupy both slots on a single piece
**Rule:** You can draft the same skill as long as you do not put it on the same Champion or King twice.
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1298-1299` — when generating legal draft turns, `sk != s2` and `sk != s1` checks prevent the same skill ID from appearing in both slots of a given piece. `skills.rs:188-199` — `validate_loadout` returns `DuplicateOnPiece` if a piece has `s1 != 0 && s1 == s2`. No cross-piece or cross-player uniqueness constraint exists.
**Adjustability:** EASY — the per-piece duplicate guard is a two-line check; removing or loosening it is straightforward.

### Draft completes once all 12 slots per player are filled
**Rule:** Continue until both Equip Slots on every Champion and the King are filled (12 skills per player).
**Status:** CORRECT
**Evidence:** `make_unmake.rs:1255-1265` — `draft_complete(pos)` iterates every King and Champion (`pos.kings | pos.champions`) and returns false if any piece has `skill1() == 0 || skill2() == 0`. The transition from `Phase::Draft` to `Phase::Move` at line 1216-1219 is gated on this predicate. Guards are excluded from the check by the bitboard mask, consistent with the rule that Guards carry no skills.
**Adjustability:** EASY — the completion check is a simple predicate loop that automatically adjusts if piece counts change, as long as the loop mask is updated to match.

---

## 16. Skill Reference — All 15 Skills

Each entry confirms cost, category, range, and effect against the rules table and the implementation in `skills.rs` / `make_unmake.rs` / `generator.rs`.

---

### Lance
**Rule:** (Strike, cost 2) Target within Range-1 takes 1 damage.
**Status:** CORRECT
**Evidence:** `skills.rs:83, 112, 133` — cost 2, default range 1, category Strike. `make_unmake.rs:552-558` — `apply_strike_damage(pos, src, tgt, 1, undo)`, `debit_money(2)`, `strike_move_caster`, `dec_actions`. The Range-1 is encoded as a stored default range of 1 rather than as a runtime modifier applied to a base-2 range.
**Adjustability:** EASY — cost in `skill_cost`, base damage in the `apply_lance` call argument, range in `skill_default_range`.

---

### Hook
**Rule:** (Strike, cost 3) Target takes 1 damage, pulled 1 tile toward caster along the path.
**Status:** CORRECT
**Evidence:** `skills.rs:84, 113, 133` — cost 3, range 2, Strike. `make_unmake.rs:613-628` — `apply_strike_damage(…, 1, …)`, then `magic::step_toward(tgt, src)` for the pull. The pull is skipped if the target died, if the pull destination equals the caster's square (adjacent case), or if the destination is occupied. `relocate_piece` updates the `tracked_casters` array after the pull, so a subsequent Hook from the same caster on the relocated piece is still correctly deduped for combo purposes.
**Adjustability:** EASY — damage is the `base` argument; pull distance and direction are explicit constants.

---

### Break
**Rule:** (Strike, cost 2) Remove 1 Armor from target. Deals no HP damage unless boosted by Charge.
**Status:** CORRECT
**Evidence:** `skills.rs:85, 114, 133` — cost 2, range 2, Strike. `make_unmake.rs:561-595` — reads the CHARGE bit, clears it if set, applies `saturating_sub(1)` to Armor unconditionally, then deals HP damage of `(if charge_active { 1 } else { 0 }) + combo_bonus`. The Combo Bonus flows through regardless of Charge. The tests `break_with_charge_deals_1_hp_damage` and `break_no_charge_no_hp_damage` verify both branches. A third test confirms Combo Bonus still applies even without Charge.
**Adjustability:** EASY — the base damage toggle is a single `if charge_active` branch; the armor strip amount is the `saturating_sub` operand.

---

### Steal
**Rule:** (Strike, cost 4) Target takes 1 damage. Steal 1 Money from opponent.
**Status:** CORRECT
**Evidence:** `skills.rs:86, 115, 133` — cost 4, range 2, Strike. `make_unmake.rs:598-611` — `apply_strike_damage(…, 1, …)`, then `transfer_money` from the opponent's pool to the caster's side. `transfer_money` uses `min(amount, from_pool)`, gracefully handling a broke opponent. Money debit (cost 4) and money transfer (steal 1) are independent, both fully reversible via delta tracking.
**Adjustability:** EASY — steal amount and cost are independently tunable.

---

### Tempest
**Rule:** (Strike, cost 4) Target takes 1 damage. All pieces adjacent to the target are pushed 1 tile away from the target. The caster is not affected.
**Status:** CORRECT
**Evidence:** `skills.rs:87, 116, 133` — cost 4, range 2, Strike. `make_unmake.rs:630-655` — `apply_strike_damage(…, 1, …)` on the target, then iterates `movement_targets_speed1(tgt)` (all 8 neighbours), skips `n == src` (caster exemption), skips empty squares, calls `magic::step_away(tgt, n)` per pushed piece, and skips if the push destination is occupied or off-board. The test `tempest_does_not_push_caster` confirms the caster exemption.
**Adjustability:** MEDIUM — the AOE loop iterates all 8 neighbours; extending push distance or adding per-push damage requires looping `step_away` twice or inserting a damage call inside the loop.

**Note:** The caster exemption (`n == src`) and the enemy-only combo gate are both in the AOE loop; the push itself (`relocate_piece`) is unconditional, so friendly pieces are correctly pushed without their combo counters being touched.

---

### Shield
**Rule:** (Shield, cost 2) Self: gain +1 Armor.
**Status:** CORRECT
**Evidence:** `skills.rs:89, 118, 135, 161` — cost 2, range 0, category Shield, TargetOwner::SelfOnly. `make_unmake.rs:659-673` — adds 1 Armor to the recipient; when `action.has_aux()`, the recipient is a Focus-retargeted adjacent ally. The generator filters out Shield actions for pieces already at the Armor cap (`generator.rs:238`).
**Adjustability:** EASY — the Armor increment is a literal `+ 1`.

---

### Heal
**Rule:** (Shield, cost 3) Remove Injured from one adjacent ally.
**Status:** CORRECT
**Evidence:** `skills.rs:90, 120, 135, 155` — cost 3, range 1, Shield, TargetOwner::Ally. `make_unmake.rs:675-683` — sets target HP to `FULL_HP (2)` with a `debug_assert` confirming the target was at `INJURED_HP (1)`. "Remove Injured" maps to HP 1 → HP 2; there is no separate status token. The generator filters out Heal actions for allies already at full HP (`generator.rs:311`), preventing waste-casts.
**Adjustability:** EASY — target HP floor/ceiling are named constants.

---

### Plate
**Rule:** (Shield, cost 3) Adjacent ally gains +1 Armor.
**Status:** CORRECT
**Evidence:** `skills.rs:91, 121, 135, 156` — cost 3, range 1, Shield, TargetOwner::Ally. `make_unmake.rs:685-693` — adds 1 Armor to the target ally. Generator filters out Plate for allies already at the Armor cap (`generator.rs:312`).
**Adjustability:** EASY — same structure as Shield, differing only in target ownership.

---

### Dash
**Rule:** (Move, cost 3) Self: move up to 2 tiles along the path.
**Status:** CORRECT
**Evidence:** `skills.rs:93, 120, 137, 159` — cost 3, range 2, Move, TargetOwner::Empty. `make_unmake.rs:695-712` — relocates the caster (or Focus-retargeted ally) to the chosen destination; no damage, no combo-tick. The generator enumerates all legal empty destinations on queen-rays up to range 2, including intermediate squares, correctly implementing "up to 2 tiles."
**Adjustability:** EASY — range in `skill_default_range`; relocation is a utility call.

---

### Blast
**Rule:** (Move, cost 2) Push target enemy 1 tile directly away from caster.
**Status:** CORRECT
**Evidence:** `skills.rs:94, 121, 137, 158` — cost 2, range 2, Move, TargetOwner::Enemy. `make_unmake.rs:724-754` — `magic::step_away(src, tgt)` computes the push direction; the push fizzles silently if the destination is occupied or off-board. Combo-tick and Combo Bonus damage apply for enemy targets. A Focus-effect mode pushes 2 tiles (with a fallback to 1 if the intermediate square is blocked).
**Adjustability:** EASY — push distance is a trivially modified chain.

---

### Shove
**Rule:** (Move, cost 3) Push target enemy 1 tile in any direction (caster chooses). Range+1.
**Status:** CORRECT
**Evidence:** `skills.rs:95, 124, 137, 158` — cost 3, range 3 (Range+1 baked in), Move, TargetOwner::Enemy. `make_unmake.rs:764-807` — the push direction is encoded in `action.choice_idx()`; 8 directions are checked; combo-tick fires only on enemy targets, matching the friendly-push exclusion. The generator emits one action per `(target, direction)` pair where the destination is empty and on-board.
**Adjustability:** EASY — direction set (8), push distance, and range are explicit constants.

---

### Swap
**Rule:** (Move, cost 4) Swap position with an allied piece. Requires unobstructed path.
**Status:** CORRECT
**Evidence:** `skills.rs:96, 123, 137, 157` — cost 4, range 2, Move, TargetOwner::Ally. `make_unmake.rs:812-851` — exchanges mailbox entries and all bitboard layers for the two pieces, correctly handling same-kind (no bitboard change needed) vs. different-kind (XOR both piece-type boards). No `combo_tick` (ally-only interaction). The "unobstructed path" requirement falls out of the generator using `path::skill_targets`, which inherently stops at the first blocker.
**Adjustability:** EASY — cost and range are independently tunable.

---

### Retreat
**Rule:** (Move, cost 4) Self: move along the path to land adjacent to one of your Guards. Range+1.
**Status:** CORRECT
**Evidence:** `skills.rs:97, 124, 137, 160` — cost 4, range 3 (Range+1 baked in), Move, TargetOwner::Empty. `make_unmake.rs:854-868` — relocates the caster to the destination; no damage. `generator.rs:282-300` — enumerates empty queen-ray squares within range, then filters to those where `movement_targets_speed1(dest) & ally_guards != 0` (the destination is adjacent to a friendly Guard). The adjacency-to-Guard constraint is enforced at generation time, not in the resolver.
**Adjustability:** EASY — the Guard-adjacency filter is a single predicate in the generator.

---

### Focus
**Rule:** (Mystic, cost 2) The next non-Mystic skill used by any of your pieces this turn gains +1 Range. Can boost Self → Range 1 and Adjacent → Range 2. For Move skills, the caster chooses whether +1 applies to the activation range or the effect range — not both.
**Status:** CORRECT
**Evidence:** `skills.rs:96, 125, 139, 162` — cost 2, range 0, Mystic, SelfOnly. `make_unmake.rs:880-891` — sets the `FOCUS` bit in `pending_modifiers`. `make_unmake.rs:522-525` — the FOCUS bit is cleared for any non-Mystic skill, leaving it intact if a Mystic skill (Charge) is cast first. `generator.rs:228-231` — `let range = if focus_pending && !is_mystic { base_range + 1 } else { base_range }`. The FOCUS bit is side-scoped, so any piece on the side benefits. The dual activation-range vs. effect-range interpretation for Move skills is implemented as two distinct `Action` encodings in the generator, presenting both as separate legal actions; the choice is made at generation time rather than at apply time — the correct approach for a tree-search engine.
**Adjustability:** MEDIUM — the Focus interaction is spread across the generator (range computation), the resolver (bit-set), and `apply_skill` (bit-clear); changing it (e.g. +2 Range) requires touching all three.

**Design note:** The rules say "Range modifiers apply to base-2 skills" but also give the example "Self + Focus → Range 1," implying Focus applies to Range-0 and Range-1 skills as well. The code implements the broader interpretation. See §11 for the full discussion.

---

### Charge
**Rule:** (Mystic, cost 3) The next Strike skill used by any of your pieces this turn deals +1 damage.
**Status:** CORRECT
**Evidence:** `skills.rs:97, 127, 139, 163` — cost 3, range 0, Mystic, SelfOnly. `make_unmake.rs:901-912` — sets the `CHARGE` bit in `pending_modifiers`. `make_unmake.rs:928-933` — `apply_strike_damage` reads the CHARGE bit, clears it, and adds `charge_bonus = 1`. Consumed only by Strike skill resolvers (Lance, Hook, Steal, Tempest) and by Break's dedicated Charge-check. The generator prevents stacking a second Charge when one is already pending (`generator.rs:221`). The test `charge_consumed_first_strike_not_second` confirms the bonus applies to exactly one Strike.
**Adjustability:** EASY — the charge bonus is a literal `1` in `apply_strike_damage`.

---

## 17. Findings

No open findings. All rule violations and documentation debts identified during the audit have been resolved:

- **Tempest AOE combo gate** (`make_unmake.rs`) — `combo_tick` in the AOE loop was unconditionally called for every adjacent piece, including friendlies. Fixed by adding the same `caster_is_p1` / `pushed_is_enemy` guard that `apply_shove` already used.
- **`moved_this_phase` doc comment** (`position.rs`) — the struct field comment described origin-square tracking; the implementation stores destination squares. Comment updated to match.
- **`income_per_turn` doc comment** (`turn_manager.rs`) — the function's doc listed "R1-4: 2" without mentioning that R1 income is suppressed by the `round_number >= 2` disbursement guard in the caller. Comment updated to note the suppression.
- **`CHAMPIONS_PER_PLAYER` wiring note** (`position.rs`) — the constant was declared with no indication it is not mechanically wired to the `1..=6` setup loop bounds or the `SideLoadout` array size. A `NOTE:` was added naming all three sites that must be updated in concert.

---

### Overall verdict

**164 rule clauses checked across 16 rule sections. All 164 are correctly implemented.**

The codebase is a faithful translation of `design/RULES.md`. Most parameters are EASY to adjust — costs, ranges, income tables, and progression tiers are all in single named constants or one-line functions. The structurally HARD constraints are: board size (8x8 baked into `u64` bitboards and magic tables) and the 2-HP / 2-Armor mailbox encoding (bit-width limited, though the current values fit comfortably within the available bits).

The one area that warrants a written design ruling is the scope of the Focus +1 Range modifier: the rule text says "applies to base-2 skills" but also gives the example "Self + Focus → Range 1," which implies it applies to all non-Mystic skills. The code resolves the ambiguity in the broader direction; the rules should be updated to match.
