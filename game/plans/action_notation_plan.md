# Action Notation Plan

New file: `crates/core_engine/src/state/action_notation.rs`, next to `fen.rs`.

Two public functions:
- `action_to_notation(action: Action, pending: Option<&PendingBodyguard>) -> String`
- `notation_to_action(s: &str, pos: &Position) -> Result<Action, NotationError>`

The `pending` parameter is `None` for every action family except BodyguardChoice. For BG redirects, it resolves the guard's square. When `None` is passed for a BG redirect (e.g. encoding a historical action without context), falls back to `bg<N>` (numeric index). The decoder accepts both `bga5` (square) and `bg2` (index) forms.

---

## The format (final)

```
Plain move:                    a1-b2
Move-Attack (speed-1):         c3xd5
Move-Attack (speed-2 Guard):   c3xd5@c4        (@approach only when approach != src)
Skill:                         b2*d4:Tempest
Skill + focus-effect mode:     b2*d4:Blast~
Skill + Focus-retarget,
  aux == target:               b2*c3:Shield>
  aux != target:               b2*d4:Dash>c3
Skill + Shove direction:       b2*d4:Shove:NE
EndPhase:                      endphase
EndTurn:                       endturn
Draft:                         draft Lance@a1:1+Shield@b2:2
Bodyguard decline:             bgX
Bodyguard redirect:            bga5            (Guard's square)
```

Square notation: file letter `a–h` + rank digit `1–8`. `sq = (rank-1)*8 + file_idx`, `file = sq%8`, `rank = sq/8+1`.

Skill names (ID → name, 1-indexed): Lance Hook Break Steal Tempest Shield Heal Plate Dash Blast Shove Swap Retreat Focus Charge.

Shove directions: `choice_idx` 0–7 → N NE E SE S SW W NW (from magic.rs `neighbour_in_dir` tests).

Draft slot: 1-indexed in notation (`1` or `2`), 0-indexed in `encode_draft_turn` (subtract 1 on decode, add 1 on encode).

---

## Part A — Implementing action_notation.rs

### A1. Module declaration

**Edit:** `crates/core_engine/src/state.rs`

Add after `pub mod fen;`:
```rust
pub mod action_notation;
pub use action_notation::NotationError;
```

**Edit:** `crates/core_engine/src/lib.rs`

Add to the existing `pub use` block (alongside `FenError`):
```rust
pub use state::action_notation::{action_to_notation, notation_to_action, NotationError};
```

---

### A2. NotationError

Mirror `FenError` from `fen.rs`. Derive `Debug, Clone, PartialEq, Eq`. Implement `Display` and `std::error::Error {}`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotationError {
    EmptyInput,
    BadSquare(String),
    UnknownSkill(String),
    UnknownDirection(String),
    NoPendingBodyguard,
    BadBodyguardSquare(String),
    TrailingInput(String),
    UnexpectedChar { pos: usize, ch: char },
}
```

---

### A3. Skill ID ↔ name table

Inline in `action_notation.rs`. Do not touch `skills.rs`.

```rust
fn skill_name(id: u8) -> Option<&'static str> {
    Some(match id {
        1  => "Lance",   2  => "Hook",    3  => "Break",
        4  => "Steal",   5  => "Tempest", 6  => "Shield",
        7  => "Heal",    8  => "Plate",   9  => "Dash",
        10 => "Blast",   11 => "Shove",   12 => "Swap",
        13 => "Retreat", 14 => "Focus",   15 => "Charge",
        _  => return None,
    })
}

fn skill_id_from_name(s: &str) -> Option<u8> {
    Some(match s {
        "Lance"   => 1,  "Hook"    => 2,  "Break"   => 3,
        "Steal"   => 4,  "Tempest" => 5,  "Shield"  => 6,
        "Heal"    => 7,  "Plate"   => 8,  "Dash"    => 9,
        "Blast"   => 10, "Shove"   => 11, "Swap"    => 12,
        "Retreat" => 13, "Focus"   => 14, "Charge"  => 15,
        _         => return None,
    })
}
```

Shove = 11, Shield = 6, Dash = 9, Retreat = 13 — these IDs matter for the `has_aux` encoding branch.

---

### A4. Square helpers

```rust
pub fn sq_to_notation(sq: u8) -> String {
    debug_assert!(sq < 64);
    let file = (b'a' + sq % 8) as char;
    let rank = sq / 8 + 1;
    format!("{}{}", file, rank)
}

pub fn notation_to_sq(s: &str) -> Result<u8, NotationError> {
    let b = s.as_bytes();
    if b.len() != 2
        || !(b'a'..=b'h').contains(&b[0])
        || !(b'1'..=b'8').contains(&b[1])
    {
        return Err(NotationError::BadSquare(s.to_string()));
    }
    Ok((b[0] - b'a') + (b[1] - b'1') * 8)
}
```

---

### A5. action_to_notation

Signature:
```rust
pub fn action_to_notation(action: Action, pending: Option<&PendingBodyguard>) -> String
```

Process the three action families in priority order:

**1. BodyguardChoice (bit 31 set)**

- `bg_guard_idx() == 0` → `"bgX"`
- `bg_guard_idx() > 0`:
  - If `pending` is `Some(pb)` and `idx` is in range: resolve `pb.eligible[idx - 1]` to a square and emit `format!("bg{}", sq_to_notation(pb.eligible[idx - 1 as usize]))` — e.g. `"bga5"`.
  - If `pending` is `None` or idx is out of range: fall back to `format!("bg{}", idx)` — e.g. `"bg1"`. This only happens when encoding a historical action where the pending state is no longer available. Document this in a module-level comment.

**2. DraftTurn (bit 30 set)**

```rust
let (s1, sq1, slot1) = action.draft_pick1();
let (s2, sq2, slot2) = action.draft_pick2();
format!("draft {}@{}:{}+{}@{}:{}",
    skill_name(s1).unwrap_or("?"), sq_to_notation(sq1), slot1 + 1,
    skill_name(s2).unwrap_or("?"), sq_to_notation(sq2), slot2 + 1)
```

**3. Regular action — branch on kind()**

- `EndPhase` → `"endphase"`
- `EndTurn`  → `"endturn"`
- `Move`:
  - `has_approach()` false → `format!("{}-{}", sq_to_notation(src), sq_to_notation(tgt))`
  - `has_approach()` true (Move-Attack):
    - base: `format!("{}x{}", sq_to_notation(src), sq_to_notation(tgt))`
    - if `approach_sq() != src` → append `format!("@{}", sq_to_notation(approach_sq()))`
    - `choice_idx` is NOT encoded — the BG selection is a separate BodyguardChoice ply.
- `Skill`:
  - base: `format!("{}*{}:{}", sq_to_notation(src), sq_to_notation(tgt), skill_name(skill_id()).unwrap_or("?"))`
  - if `focus_effect_mode()` → append `"~"`
  - if `has_aux()`:
    - if `aux_sq() == tgt` → append `">"`
    - else → append `format!(">{}", sq_to_notation(aux_sq()))`
  - if `skill_id() == 11` (Shove) → append `format!(":{}", DIRS[choice_idx() as usize])` where `DIRS = ["N","NE","E","SE","S","SW","W","NW"]`

Suffix order: `~` then `>` then `:DIR`. `~` and `>` can coexist on the same action; Shove and `has_aux` cannot.

---

### A6. notation_to_action

Signature:
```rust
pub fn notation_to_action(s: &str, pos: &Position) -> Result<Action, NotationError>
```

Branch on the first recognisable prefix of the trimmed input:

| Prefix / pattern | Family |
|---|---|
| `""` | `Err(EmptyInput)` |
| `"endphase"` | EndPhase |
| `"endturn"` | EndTurn |
| `"bgX"` | BodyguardChoice decline |
| `"bg"` + letter | BodyguardChoice redirect by square |
| `"bg"` + digit | BodyguardChoice redirect by raw index |
| `"draft "` | DraftTurn |
| contains `'*'` | Skill |
| contains `'x'` | Move-Attack |
| contains `'-'` | Plain move |
| anything else | `Err(UnexpectedChar)` |

After parsing, assert no characters remain or return `Err(TrailingInput)`.

**EndPhase / EndTurn:** `Action::encode(0, 0, ActionKind::EndPhase/EndTurn, 0, 0)`.

**BodyguardChoice decline (`bgX`):** `Action::encode_bodyguard_choice(0)`.

**BodyguardChoice by square (e.g. `bga5`):** Strip `"bg"` prefix, pass remainder (e.g. `"a5"`) to `notation_to_sq`. Look up square in `pos.pending_bodyguard`:
- `None` → `Err(NoPendingBodyguard)`
- found at index k in `eligible[0..eligible_len]` → `Action::encode_bodyguard_choice(k as u8 + 1)`
- not found → `Err(BadBodyguardSquare(...))`

**BodyguardChoice by index (e.g. `bg2`):** Strip `"bg"` prefix, parse remainder as `u8`. Validate `<= BG_CHOICE_MAX_IDX`. `Action::encode_bodyguard_choice(idx)`.

**DraftTurn:** Strip `"draft "` prefix. Split on `'+'` → two pick strings. Parse each as `<SkillName>@<sq>:<slot>`:
- Split on `'@'` → skill name and `<sq>:<slot>`.
- Split on `':'` → square string and slot char.
- `skill_id_from_name` — `Err(UnknownSkill)` if not found.
- `notation_to_sq` — `Err(BadSquare)` if bad.
- slot `'1'` → 0, `'2'` → 1, else `Err(UnexpectedChar)`.
- `Action::encode_draft_turn(sk1, sq1, sl1, sk2, sq2, sl2)`.

**Plain move:** Split on `'-'`. Parse two squares. `Action::encode(src, tgt, ActionKind::Move, 0, 0)`.

**Move-Attack:** Split on `'x'`. Left = src. Right may contain `'@'`:
- If `'@'` present: split → tgt and approach.
- If absent: tgt = right, approach = src (speed-1 default).
- `Action::encode_move_attack(src, tgt, /*choice_idx=*/0, approach)`.

**Skill:** Split on `'*'` → src string and remainder. Split remainder on first `':'` → tgt string and suffix string.

Parse suffix left to right:
1. Read skill name up to `'~'`, `'>'`, `':'`, or end. `skill_id_from_name` → `Err(UnknownSkill)`.
2. If next char `'~'` → `focus_effect = true`, consume.
3. If next char `'>'` → `has_aux = true`, consume. Peek next 2 chars: if `[a-h][1-8]` then consume and parse as `aux_sq`; else `aux_sq = tgt` (bare `>`).
4. If next char `':'` → consume. Read remaining string as direction name. Look up in `DIRS`. `Err(UnknownDirection)` if not found. This is `choice_idx`.
5. Assert empty remainder or `Err(TrailingInput)`.

Construct action:
- Neither: `Action::encode(src, tgt, ActionKind::Skill, skill_id, choice_idx)`
- Only `has_aux`: `Action::encode_with_aux(src, tgt, ActionKind::Skill, skill_id, choice_idx, aux_sq)`
- Only `focus_effect`: `Action::encode_focus_effect(src, tgt, ActionKind::Skill, skill_id, choice_idx)`
- Both: `encode_with_aux(...)` then `a.0 |= 1 << 22` (no single constructor covers this combination)

---

### A7. Tests

| # | Name | What it covers |
|---|---|---|
| 1 | `sq_roundtrip_all_64` | All 64 squares round-trip through sq_to_notation / notation_to_sq |
| 2 | `sq_corners` | sq=0 → "a1", sq=63 → "h8" |
| 3 | `sq_bad_input` | "z9", "a9", "i1", "" all → Err(BadSquare) |
| 4 | `plain_move_roundtrip` | a1-b2 encode→string→decode |
| 5 | `move_attack_speed1` | c3xd5, approach==src, no @ suffix |
| 6 | `move_attack_speed2` | c3xd5@c4, approach != src |
| 7 | `skill_basic` | b2*d4:Tempest |
| 8 | `skill_focus_effect_mode` | b2*d4:Blast~, bit 22 set |
| 9 | `skill_focus_retarget_aux_eq_target` | b2*c3:Shield>, aux==tgt |
| 10 | `skill_focus_retarget_aux_ne_target` | b2*d4:Dash>c3, aux!=tgt |
| 11 | `skill_focus_effect_and_aux_combined` | Both bit 22 and has_aux; ~ and > both present and round-trip |
| 12 | `skill_shove_all_8_directions` | Loop N/NE/…/NW, encode→decode→encode |
| 13 | `endphase_roundtrip` | "endphase" |
| 14 | `endturn_roundtrip` | "endturn" |
| 15 | `draft_roundtrip` | draft Lance@a1:1+Shield@b2:2 |
| 16 | `draft_slot2` | Pick using slot 2 on both entries |
| 17 | `draft_extremal` | skill=15 (Charge), sq=h8, slot2 both picks |
| 18 | `bodyguard_decline` | bgX → encode_bodyguard_choice(0) |
| 19 | `bodyguard_redirect_by_square` | bga5 with PendingBodyguard containing a5 at index 0 → encode_bodyguard_choice(1); also verify action_to_notation with pending resolves back to "bga5" |
| 20 | `bodyguard_redirect_by_index` | bg2 → encode_bodyguard_choice(2) |
| 21 | `bodyguard_encode_without_pending` | action_to_notation with pending=None for a BG redirect emits "bg1" fallback |
| 22 | `bodyguard_no_pending` | bga5 when pending_bodyguard=None → Err(NoPendingBodyguard) |
| 23 | `bodyguard_square_not_in_eligible` | bga5 when eligible list does not contain a5 → Err(BadBodyguardSquare) |
| 24 | `error_unknown_skill` | "a1*b2:Nuke" → Err(UnknownSkill) |
| 25 | `error_bad_square` | "z9-a1" → Err(BadSquare) |
| 26 | `error_trailing_input` | "endphaseXXX" → Err(TrailingInput) |
| 27 | `error_empty_input` | "" → Err(EmptyInput) |
| 28 | `error_unknown_direction` | "a1*b2:Shove:XX" → Err(UnknownDirection) |

---

## Part B — Wiring

### B1. `crates/core_engine/src/telemetry.rs` — unify notation

**The problem:** `telemetry::notation::fmt_action` is a second independent action renderer. It must be deleted and replaced by `action_to_notation`. The complication is that `fmt_action` takes `&ActionDecoded`, which by the time it is called has already lost the `pending_bodyguard` context needed to resolve a BG redirect to a guard square.

**Root cause:** In `session.rs::try_apply_timed` (line 449), `make()` is called before `ActionDecoded::from_action` is constructed (line 463). By the time `from_action` runs, `pending_bodyguard` has been cleared by `make()`.

**Fix — two coordinated changes:**

**Change 1: `ActionDecoded` gains a `notation` field.**

In `telemetry.rs`, add a field to `ActionDecoded`:
```rust
pub notation: String,
```
This field is populated at record time with the canonical action notation string, computed *before* `make()` is called, when `pending_bodyguard` is still live.

**Change 2: `session.rs::try_apply_timed` snapshots notation before make().**

Currently (lines 439–449):
```rust
let pre = if self.log.is_some() {
    // ... snapshot_pre(pos) ...
    Some((seat_player, seat_kind, ...))
};
let undo = make_unmake::make(&mut self.position, action);  // line 449
```

Add notation capture inside the `pre` block, before `make()`:
```rust
let notation = action_to_notation(action, self.position.pending_bodyguard.as_ref());
```

Pass `notation` through the `pre` tuple into the `PlyRecord` construction at line 460. Set `action: ActionDecoded::from_action_with_notation(action, notation)`.

Add a constructor to `ActionDecoded`:
```rust
pub fn from_action_with_notation(a: Action, notation: String) -> Self {
    let mut d = Self::from_action(a);
    d.notation = notation;
    d
}
```

`#[serde(default)]` on `notation` so legacy match-log JSON (no `notation` field) still deserialises — default to empty string, which downstream tools can detect and re-derive if needed.

**Change 3: `fmt_action` is deleted; `to_text` calls `d.notation` directly.**

In `telemetry::notation`:
- Delete `fn sq_name` (duplicates `sq_to_notation`).
- Delete `fn fmt_action`.
- In `to_text`, replace `fmt_action(&p.action)` with `&p.action.notation`.

The same-session snapshot replay path in `from_snapshot_with_clock` (line 333) also calls `ActionDecoded::from_action(action)` — update it to `ActionDecoded::from_action_with_notation(action, action_to_notation(action, None))`. For replayed history, `pending_bodyguard` context is gone, so `pending=None` is correct; BG redirects in replayed logs will fall back to `bg<N>` notation. This is acceptable — the notation in the original live PlyRecord was already captured correctly.

**Tests to update in telemetry.rs:** `notation_has_header_and_per_ply_lines` and `notation_marks_eval_swing` construct `PlyRecord` stubs with `action: ActionDecoded::from_action(Action::default())`. After this change, update those stubs to use `ActionDecoded::from_action_with_notation(Action::default(), "endturn".to_string())` (or whatever action suits the test). The test assertions on the formatted string output will need to be updated to match the new compact notation format (`"a1-b2"` instead of `"Move a1→b2"`, `"endphase"` instead of `"EndPhase"`, etc.).

---

### B2. `crates/nn_trainer/src/run.rs`

**Current (around line 616):**
```rust
last_action: format!("{:?}", action),
```

**After:**
```rust
last_action: core_engine::action_to_notation(*action, None),
```

`pending_bodyguard` context is not available at this call site (training self-play has no pending state at the point of logging). `None` is correct; BG redirects fall back to `bg<N>`.

---

### B3. `crates/search_bench/src/main.rs`

**Current:** `action_brief(a: Option<Action>) -> String` manually formats kind/src/target/raw-hex.

**After:**
```rust
fn action_brief(a: Option<Action>) -> String {
    match a {
        None      => "(none)".to_string(),
        Some(act) => core_engine::action_to_notation(act, None),
    }
}
```

Check whether `ActionKind` import at top of `main.rs` is still used elsewhere after removing the old body; remove it if not.

---

### B4. `crates/tauri_wrapper`

No changes. The frontend's `action-label.ts` serves the game UI with visually richer formatting (arrows, direction labels) intentionally different from the engine's machine-readable notation. These serve different audiences and should remain independent.

---

## Files changed summary

| File | Change |
|---|---|
| `crates/core_engine/src/state/action_notation.rs` | **Create** |
| `crates/core_engine/src/state.rs` | Add `pub mod action_notation;` + `pub use` |
| `crates/core_engine/src/lib.rs` | Add `pub use state::action_notation::{...}` |
| `crates/core_engine/src/telemetry.rs` | Add `notation: String` to `ActionDecoded`; delete `fmt_action` + `sq_name`; update `to_text`; update affected tests |
| `crates/core_engine/src/session.rs` | Capture notation before `make()`; pass to `ActionDecoded::from_action_with_notation` |
| `crates/nn_trainer/src/run.rs` | Replace `format!("{:?}", action)` with `action_to_notation(*action, None)` |
| `crates/search_bench/src/main.rs` | Replace `action_brief` body |

---

## Gotchas

1. **`focus_effect_mode` + `has_aux` combined:** No single constructor covers both. Use `encode_with_aux(...)` then `a.0 |= 1 << 22`.

2. **Move-Attack `choice_idx` is not encoded in notation.** The BG selection is a separate BodyguardChoice ply. Decoder always sets `choice_idx = 0` for Move-Attack.

3. **BodyguardChoice: encode with context, fall back without.** `action_to_notation` takes `Option<&PendingBodyguard>`. Pass `self.position.pending_bodyguard.as_ref()` at the one live call site in `session.rs`. All other call sites pass `None` and get `bg<N>` fallback. Document this in the module comment.

4. **`ActionDecoded::notation` must be populated before `make()`.** The only correct place is the `pre` snapshot block in `session.rs::try_apply_timed`. Anywhere else, `pending_bodyguard` is gone.

5. **Legacy JSON compatibility.** Add `#[serde(default)]` to `ActionDecoded::notation` so existing match-log files (without the field) still deserialise. Default to empty string.

6. **Telemetry test stubs need updating.** Tests that construct `PlyRecord` manually must use `from_action_with_notation`. Update the expected string assertions to match the new compact format.

7. **Shove direction bounds.** `choice_idx` must be 0..=7. `debug_assert` in encoder; `Err(UnknownDirection)` in decoder for out-of-range.

8. **Draft slot is 1-indexed in notation, 0-indexed in bits.** Decoder subtracts 1; encoder adds 1.

9. **`endphase`/`endturn` zero out src/target.** Canonical decoder always constructs with zeros.
