# Scenario Format

Plain-text formats the engine's debug harness understands. Frozen so the designer can author scenarios in parallel with engine work.

**Status (Session 29):** FEN encoder/parser implemented, plus strict-mode setup validator (slice 0). Action-text parser and `.scenario` runner still deferred.

---

## 1. FEN — position serialisation

Single-line, space-separated:

```
<board> <to_move> <phase> <actions_remaining> <p1_money> <p2_money> <pending_modifiers>
```

### Board

Eight `/`-separated ranks, **rank 8 first** (top of display), rank 1 last (bottom). Within each rank, files run `a..h` left-to-right.

Each rank is a sequence of:
- **Piece tokens**: `K`/`C`/`G` for P1 King/Champion/Guard; lowercase `k`/`c`/`g` for P2.
- **Run-length digits** `1..8` for empty squares.

The squares per rank must sum to exactly 8.

### Piece state bracket

A piece token may carry a `[hp/armor/combo/skill1/skill2]` suffix:

```
C[1/2/0/3/7]
```

When the bracket is **omitted**, the piece defaults to `2/0/0/0/0` (full HP, no armor, no combo, no skills). The encoder emits the bracket iff at least one field is non-default.

Field ranges:
- `hp` ∈ 0..=2
- `armor` ∈ 0..=3
- `combo` ∈ 0..=7
- `skill1`, `skill2` ∈ 0..=15 (0 = unequipped)

### Trailing scalars

- `<to_move>` ∈ `{P1, P2}`
- `<phase>` ∈ `{M, S}` — Move phase or Skill phase
- `<actions_remaining>` decimal 0..=255
- `<p1_money>`, `<p2_money>` decimal 0..=65535
- `<pending_modifiers>` decimal 0..=255 (bit 0 = FOCUS, bit 1 = CHARGE; other bits reserved)

### Fields not in FEN

These are turn-scoped or derived and reset each turn boundary:

- `tracked_enemies` / `tracked_enemies_len`
- `champion_credit`
- `zobrist` (derived from the rest)

FEN represents between-turn state. Mid-turn search state stays in-memory only.

### Example

```
7k/8/8/8/8/8/8/K7 P1 M 2 6 6 0
```

P1 King at a1, P2 King at h8, otherwise empty. P1 to move, Move phase, 2 actions, 6 money each, no modifiers.

### Strict vs lax parsing

Two entry points, same grammar:

- **`from_fen(s)`** — structural validity only. Verifies the board syntax parses, every rank sums to 8 squares, exactly one King per side, and bracketed fields are in-range. Accepts any reachable mid-game position (captures, depleted armies, etc.).
- **`from_fen_strict(s)`** — also enforces **Stack M setup invariants**: per side, exactly 1 King + 5 Champions + 6 Guards, and the two Kings on different files. Use this for setup-position scenarios and any FEN that claims to be a legal starting position.

Mid-game scenarios (slice 1+ make/unmake tests, opening-fragment captures) must use plain `from_fen`. The strict variant will reject them as soon as a piece is taken.

`setup_stack_m()` itself produces a known-valid position by construction; it does not run the validator.

---

## 2. Action text — single-action notation

Used by the (future) `gamedbg apply / trace` CLI and `.scenario` files.

```
<action>      ::= <move> | <skill> | 'endphase' | 'endturn'
<move>        ::= <square> '-' <square>
<skill>       ::= <square> '*' <target-or-dir> ':' <skill-name>
<target-or-dir> ::= <square> | <direction>
<direction>   ::= 'N' | 'NE' | 'E' | 'SE' | 'S' | 'SW' | 'W' | 'NW'
<square>      ::= file rank      ; file a..h, rank 1..8
```

Examples:

```
e2-e4              ; move a piece from e2 to e4
e2*e4:Lance        ; cast Lance from e2 at e4
e2*Shove:NE        ; direction-only skill (no target square)
endphase           ; end current phase voluntarily
endturn            ; end turn (closes both phases)
```

`<skill-name>` resolves against the skill ID table built in slice 4. Until that lands, scenarios using `*` will fail to parse.

---

## 3. Scenario file (`.scenario`) — single test

A `.scenario` file declares one starting position, one action, and one or more assertions. The slice-0 runner reads `tests/scenarios/*.scenario`.

```
# <human description>.scenario
given: <fen>
action: <action-text>
expect_position: <fen-after>     ; optional
expect_legal:    true | false    ; optional
expect_trace:                    ; optional
  - <line>
  - <line>
```

Rules:
- Lines starting with `#` are comments.
- `given` and `action` are required.
- At least one `expect_*` clause must be present.
- Multi-action sequences are out of scope for slice 0 — author them as multiple files chained by FEN, or wait for the slice-N batch-runner extension.

The runner reports per-file pass/fail with a diff of position-after when `expect_position` mismatches.

---

## 4. Designer workflow

1. Write a `.scenario` file describing the exact engine behaviour you want to lock in.
2. Run `cargo test -p core_engine` (or the future `gamedbg scenario <file>`).
3. The runner asserts the engine matches; if not, the diff tells you what changed.
4. New rules in slice N add new scenario files. Old scenarios stay green as regressions.
