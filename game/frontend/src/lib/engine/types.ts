// Unified TS types for the engine-bridge boundary (Tauri IPC).

export interface PositionView {
  /** [p1, p2, kings, champions, guards] as 5 × u64. */
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
   *  `BodyguardChoice` ply. Renderer-only — legality flows through the
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

/** L8 — snapshot of the in-progress draft. Returned by `draftState()` and
 *  used by the /draft/ route to drive picker UI legality hints. */
export interface DraftStateView {
  /** Number of `DraftTurn` plies committed so far (0..12). Reads 12 once
   *  the engine has transitioned to Phase::Move. */
  turnNo: number;
  /** 0 = P1, 1 = P2. Undefined once `turnNo === 12`. */
  sideToMove: number;
  /** `usedSlots[piece][slot]` — true iff that mailbox slot is filled.
   *  Layout: pieces 0..6 = P1 (King at 0, Champions 1..5 by ascending sq),
   *  pieces 6..12 = P2 (same internal order), slot ∈ {0,1}. */
  usedSlots: boolean[][];
}

/** L8 — a single side's loadout: 6 [skill1, skill2] pairs.
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
  total:        number;
}

/** Per-square eval breakdown — one entry per board square (0..63). Empty
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
  /** L8 — open a fresh match in `Phase::Draft`. Caller drives 12 DraftTurn
   *  plies via `tryApply` / `stepAi`; engine transitions to Phase::Move
   *  automatically. */
  createEngineWithDraft(configJson?: string): Promise<void>;
  /** L8 — open a fresh match that bypasses draft, with both sides' loadouts
   *  already applied. Engine validates loadouts and rejects same-skill-on-
   *  same-piece pairs. */
  createEngineWithLoadouts(
    configJson: string | undefined,
    p1Loadout: SideLoadout,
    p2Loadout: SideLoadout,
  ): Promise<void>;
  /** L8 — snapshot of the in-progress draft. Cheap; safe to call per UI
   *  refresh. Returns `turnNo === 12` once the draft has completed. */
  draftState(): Promise<DraftStateView>;
  positionView(): Promise<PositionView>;
  legalActions(): Promise<Uint32Array>;
  tryApply(action: number): Promise<StepResult>;
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
   *  restricted variant was removed as dead surface — match uses `stepAi`. */
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
