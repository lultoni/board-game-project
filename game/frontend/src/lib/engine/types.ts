// Unified TS types for the engine-bridge boundary (Tauri IPC).

export interface PositionView {
  /** [p1, p2, kings, champions, guards] as 5 x u64. */
  bitboards: BigUint64Array;
  /** 64 entries, u16 packed (see decodeMailbox). */
  mailbox: Uint16Array;
  toMove: number;
  currentPhase: number;
  actionsRemaining: number;
  roundNumber: number;
  p1Money: number;
  p2Money: number;
  pendingModifiers: number;
  gameResult: number;
  zobrist: bigint;
  /** Engine-owned Bodyguard state. `null` in the common case; populated
   *  between the attacker's tentative Move-Attack and the defender's
   *  `BodyguardChoice` ply. Renderer-only - legality flows through the
   *  legal-actions buffer, which is restricted to BodyguardChoice variants
   *  while this is non-null. */
  pendingBodyguard: PendingBodyguardView | null;
}

export interface PendingBodyguardView {
  /** Pre-hop square of the attacker (where it sat before Move-Attack). */
  attackerSrc: number;
  /** Current square of the attacker (post first-hop relocation). Equal to
   *  `attackerSrc` for speed-1 attackers. */
  attackerNow: number;
  /** Defender square (Champion/King under attack). */
  targetSq: number;
  /** Eligible Bodyguard Guard squares in canonical ascending order. The
   *  k-th entry (0-indexed) corresponds to `BodyguardChoice(idx=k+1)`. */
  eligible: number[];
}

export interface StepResult {
  appliedAction: number;
  score: number;
  depth: number;
  nodes: bigint;
  thoughtMs: number;
  gameResult: number;
}

/** L8 - snapshot of the in-progress draft. Returned by `draftState()` and
 *  used by the /draft/ route to drive picker UI legality hints. */
export interface DraftStateView {
  /** Number of `DraftTurn` plies committed so far (0..12). Reads 12 once
   *  the engine has transitioned to Phase::Move. */
  turnNo: number;
  /** 0 = P1, 1 = P2. Undefined once `turnNo === 12`. */
  sideToMove: number;
  /** `usedSlots[piece][slot]` - true iff that mailbox slot is filled.
   *  Layout: pieces 0..6 = P1 (King at 0, Champions 1..5 by ascending sq),
   *  pieces 6..12 = P2 (same internal order), slot ∈ {0,1}. */
  usedSlots: boolean[][];
}

/** L8 - a single side's loadout: 6 [skill1, skill2] pairs.
 *  Piece order: King at index 0, Champions 1..5 by ascending starting square.
 *  Slot value 0 = empty (only valid during Phase::Draft). */
export type SideLoadout = readonly [
  readonly [number, number],
  readonly [number, number],
  readonly [number, number],
  readonly [number, number],
  readonly [number, number],
  readonly [number, number],
];

export type FinalResultByte = 0 | 1 | 2 | 3; // P1Win | P2Win | Draw | Aborted

/** Engine-canonical skill metadata (mirror of Rust `SkillMetadata`, camelCase
 *  off the wire). The synchronous `SKILLS` table in `engine/skills.ts` is
 *  asserted against this by `skills.contract.test.ts`. */
export interface SkillMetadataWire {
  id: number;
  key: string;
  category: string;
  cost: number;
  defaultRange: number;
  targetOwner: string;
  hasFocusModeChoice: boolean;
  needsDirectionPick: boolean;
}

/** Engine-canonical game constants (mirror of Rust `GameConstants`). */
export interface GameConstantsWire {
  phaseMove: number;
  phaseSkill: number;
  phaseDraft: number;
  modifierFocus: number;
  modifierCharge: number;
  modifierMoveAttackUsed: number;
  playerP1: number;
  playerP2: number;
  gameOngoing: number;
  gameP1Wins: number;
  gameP2Wins: number;
  skillCount: number;
}

/** Per-component decomposition of the static heuristic eval. Mirrors the
 *  Rust `search::evaluator::EvalBreakdown` struct 1:1 (snake_case field
 *  names as they come off the Tauri IPC wire). `total` = sum(*_p1) - sum(*_p2)
 *  in the non-terminal case. */
export interface EvalBreakdown {
  material_p1:  number;
  material_p2:  number;
  hp_p1:        number;
  hp_p2:        number;
  armor_p1:     number;
  armor_p2:     number;
  skills_p1:    number;
  skills_p2:    number;
  money_p1:     number;
  money_p2:     number;
  mobility_p1:  number;
  mobility_p2:  number;
  threat_p1:    number;
  threat_p2:    number;
  skill_act_p1: number;
  skill_act_p2: number;
  /** E9 - max castable offensive range flag (raw, e.g. 2..=4). Weighted
   *  into `total` by OFFENSIVE_RANGE_WEIGHT (500) on the Rust side. */
  offensive_range_p1: number;
  offensive_range_p2: number;
  total:        number;
}

/** Per-square eval breakdown - one entry per board square (0..63). Empty
 *  squares carry `occupied: false` with all terms zero. Piece kinds:
 *  0=empty, 1=guard, 2=champion, 3=king. Skill availabilities are
 *  fixed-point on `SKILL_AVAIL_MAX = 256` (percent = fp * 100 / 256). */
export interface SquareBreakdown {
  sq: number;
  occupied: boolean;
  is_p1: boolean;
  piece_kind: number;
  hp: number;
  armor: number;
  skill1_id: number;
  skill2_id: number;

  material: number;
  hp_term: number;
  armor_term: number;
  skills_term: number;
  mobility_term: number;
  exposure_term: number;
  coverage_term: number;
  piece_total: number;

  skill1_avail_fp: number;
  skill2_avail_fp: number;
  n_attackers: number;
  n_adj_guards: number;
  mobility_raw: number;
  empty_ring_total: number;
  empty_ring_shielded: number;
}

/** Full per-square breakdown plus side-level context (money, tempo,
 *  reconciled total). Sum of all squares' owner-signed totals + side
 *  money/tempo terms equals `total` for non-terminal positions. */
export interface EvalBreakdownBySquare {
  squares: SquareBreakdown[];
  p1_money: number;
  p2_money: number;
  p1_money_cap: number;
  p2_money_cap: number;
  p1_money_term: number;
  p2_money_term: number;
  p1_tempo_term: number;
  p2_tempo_term: number;
  total: number;
  terminal: boolean;
}

export interface EngineClient {
  version(): Promise<string>;
  createEngine(configJson?: string): Promise<void>;
  /** L8 - open a fresh match in `Phase::Draft`. Caller drives 12 DraftTurn
   *  plies via `tryApply` / `stepAi`; engine transitions to Phase::Move
   *  automatically. */
  createEngineWithDraft(configJson?: string): Promise<void>;
  /** L8 - open a fresh match that bypasses draft, with both sides' loadouts
   *  already applied. Engine validates loadouts and rejects same-skill-on-
   *  same-piece pairs. */
  createEngineWithLoadouts(
    configJson: string | undefined,
    p1Loadout: SideLoadout,
    p2Loadout: SideLoadout,
  ): Promise<void>;
  /** L8 - snapshot of the in-progress draft. Cheap; safe to call per UI
   *  refresh. Returns `turnNo === 12` once the draft has completed. */
  draftState(): Promise<DraftStateView>;
  positionView(): Promise<PositionView>;
  legalActions(): Promise<Uint32Array>;
  /** Encode a raw action u32 to canonical notation (e.g. "a1-b2", "b2*d4:Tempest").
   *  Stateless — does not require an active engine handle. */
  actionToNotation(raw: number): Promise<string>;
  /** Engine-canonical skill table. Stateless (no engine handle). Used only by
   *  the contract test that guards the synchronous `SKILLS` mirror. */
  skillMetadata(): Promise<SkillMetadataWire[]>;
  /** Engine-canonical game constants. Stateless. Used only by the contract
   *  test that guards the synchronous constants in `engine/skills.ts`. */
  gameConstants(): Promise<GameConstantsWire>;
  /** Apply a human action. `turnStartedMs` (a `Date.now()` reading captured
   *  when the current turn/phase began) lets the engine record human decision
   *  time in telemetry; omit / pass 0 for non-live contexts (replay, inspector,
   *  snapshot rebuild) where think-time is meaningless. */
  tryApply(action: number, turnStartedMs?: number): Promise<StepResult>;
  stepAi(onDepth?: (depth: number, score: number) => void): Promise<StepResult>;
  /** Compute the static heuristic evaluation of the current board position.
   *  Returns the full per-component breakdown; `total` is P1-POV
   *  (positive = P1 ahead). */
  heuristicEval(): Promise<EvalBreakdown>;
  /** Per-square variant: returns the full 64-square breakdown plus side-
   *  level money/tempo terms. Used by the eval-diagnostic hover overlay. */
  heuristicEvalBySquare(): Promise<EvalBreakdownBySquare>;
  /** Inspector variant: runs the search regardless of seat kind so HvH
   *  positions can also ask "what would the AI play here?". The seat-
   *  restricted variant was removed as dead surface - match uses `stepAi`. */
  requestAiMoveForced(): Promise<StepResult>;
  /** Inspector iterative-deepening: runs ID up to `maxDepth` with no time
   *  bound. Caller drives the deepening loop by stepping `maxDepth` up
   *  by 1 each call and polling cancellation between calls. */
  requestAiMoveAtDepth(maxDepth: number): Promise<StepResult>;
  positionFen(): Promise<string>;
  snapshotJson(): Promise<string>;
  restoreFromSnapshot(json: string): Promise<void>;
  matchLogJson(): Promise<string | null>;
  /** Latest `PlyRecord` JSON (the newest entry in the match log). `null` when
   *  `auto_log` is off or no plies recorded yet. Used by the telemetry
   *  persistence layer to write per-ply incrementally without re-serialising
   *  the entire log. */
  latestPlyJson(): Promise<string | null>;
  /** Subscribe to the `background-eval-ready` event the engine emits after a
   *  human ply's time-bounded background search completes (Change 5 Part B).
   *  The callback fires with the affected engine handle; the consumer then
   *  reads `latestPlyJson()` to pick up the freshly-annotated `background_eval`.
   *  Returns an unlisten function. On WASM this is a no-op returning a no-op
   *  unlisten (no background thread there). */
  onBackgroundEvalReady(cb: (handle: number) => void): Promise<() => void>;
  /** AIvAI producer (Change 6): start a background thread that plays the whole
   *  AI-vs-AI game to completion from `viewSnapshotJson` (so producer + view
   *  share start_fen+config), re-installing both seat evaluators (from_snapshot
   *  resets them to heuristic). The producer appends each ply to its own log
   *  and emits `aivai-progress`; the frontend log-player advances a separate
   *  view engine at display cadence. No-op on non-Tauri clients. */
  startAivaiProducer(
    viewSnapshotJson: string,
    p1: { source: "heuristic" | "run" | "blessed"; id?: string | null },
    p2: { source: "heuristic" | "run" | "blessed"; id?: string | null },
  ): Promise<void>;
  /** Non-joining read of the producer's currently-published MatchLog JSON
   *  (raw actions + ply count). `null` when no producer is running. */
  aivaiProducerLog(): Promise<string | null>;
  /** Abort + JOIN the producer, returning its final authoritative MatchLog
   *  JSON. Awaited on leaving an AIvAI match: the join guarantees the in-flight
   *  ply is appended + the log finalised before the caller persists it, so the
   *  saved log length equals exactly what the producer computed. `null` when no
   *  producer was running. */
  stopAivaiProducer(): Promise<string | null>;
  /** Subscribe to `aivai-progress` (the producer appended a ply). The callback
   *  fires with the producer's current ply count (the ceiling the log-player
   *  advances toward) and whether the producer has finished. Returns an
   *  unlisten fn. No-op returning a no-op unlisten on non-Tauri clients. */
  onAivaiProgress(cb: (plies: number, done: boolean) => void): Promise<() => void>;
  finaliseLog(result: FinalResultByte): Promise<void>;
  /** Free engine resources (Tauri only; no-op on WASM). */
  dispose(): Promise<void>;
  /** Install a position evaluator on an AI seat. `source = "heuristic"` is
   *  the default; `"run"` and `"blessed"` load an `NnEvaluator` from the
   *  respective rater directory. WASM-side: no-op (NN inference isn't
   *  bundled into the web build). */
  setAiEvaluator(
    source: "heuristic" | "run" | "blessed",
    id?: string | null,
    runDir?: string | null,
  ): Promise<void>;
}
