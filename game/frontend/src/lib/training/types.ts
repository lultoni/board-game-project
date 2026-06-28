// Wire-format types for the Training Observatory.
//
// These mirror the serde shapes produced by the Tauri commands in
// game/crates/tauri_wrapper/src/lib.rs (which in turn mirror the structs in
// game/crates/nn_trainer/src/). When the Rust side changes, keep these in
// sync — serde uses snake_case here (the snapshot/live/matrix structs use
// the default rename, not camelCase like the engine wrappers).

export type TrainingPhase = "idle" | "training" | "gauntlet" | "bookkeeping";

/** Mirrors `BackendInfo` from `list_backends`. `id` is the lowercase tag
 *  echoed back to `start_training_run`'s `backend` argument. */
export interface BackendInfo {
  id: "cpu" | "wgpu" | "cuda";
  label: string;
  is_default: boolean;
}

export interface PopulationMember {
  rater_id: string;
  parent_id: string | null;
  lineage: number;
  generation: number;
  wins: number;
  losses: number;
  draws: number;
  alive: boolean;
}

export interface ActiveMatch {
  challenger: string;
  defender: string;
  game_index: number;
  games_total: number;
  ply: number;
  think_ms: number;
  bracket: string;
}

export interface StatusSnapshot {
  format_version: number;
  written_at_ms: number;
  phase: TrainingPhase;
  generation: number;
  round: number;
  eta_seconds: number | null;
  population: PopulationMember[];
  active_match: ActiveMatch | null;
}

export interface EvalBars {
  challenger_nn: number | null;
  defender_nn: number | null;
  heuristic: number | null;
}

export interface LivePosition {
  format_version: number;
  written_at_ms: number;
  fen: string;
  last_action: string;
  ply: number;
  challenger: string;
  defender: string;
  game_index: number;
  games_total: number;
  evals: EvalBars;
}

export interface BracketWinRate {
  games_played: number;
  candidate_wins: number;
  baseline_wins: number;
  indecisive: number;
}

export interface IndexEntry {
  id: string;
  stem: string;
  accepted_at: string;
  /** Parent rater id; null for the founding entry. Optional for
   *  backwards-compat with index.json files written before the field
   *  was added. */
  parent_id?: string | null;
  bracket_results: Record<string, BracketWinRate>;
}

export type Track = "fast" | "slow" | "overall";

export interface RaterIndex {
  format_version: number;
  entries: IndexEntry[];
  tracks: Partial<Record<Track, string>>;
}

export interface MatrixEntry {
  challenger: string;
  defender: string;
  bracket: string;
  result: BracketWinRate;
}

export interface GauntletMatrix {
  format_version: number;
  entries: MatrixEntry[];
}

export interface WeightStats {
  layer: string;
  mean: number;
  std: number;
  min: number;
  max: number;
  nanCount: number;
}

export interface RaterInspection {
  raterId: string;
  forwardOutput: number;
  /** Centipawn-scale conversion factor. Calibrated value from sidecar, or
   *  the DEFAULT_EVAL_SCALE fallback (3000) for un-calibrated raters.
   *  Multiply `forwardOutput * evalScale` for the centipawn-scale score. */
  evalScale: number;
  weightStats: WeightStats[];
}
