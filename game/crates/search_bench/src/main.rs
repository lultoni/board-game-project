//! Search-speed benchmark for `core_engine`.
//!
//! See `design/inbox/digital/search-speed-benchmark-plan.md` for the design.
//! Manual-run-only tool - not part of CI.
//!
//! Usage:
//!   cargo run -p search_bench --release -- \
//!       --corpus game/bench/corpus/corpus.txt \
//!       --mode depth --depth 6 --runs 5 \
//!       --out game/bench/results/run.json
//!
//!   cargo run -p search_bench --release -- \
//!       --corpus game/bench/corpus/corpus.txt \
//!       --mode time --time-ms 1000 --runs 5 \
//!       --out game/bench/results/run.json
//!
//!   cargo run -p search_bench --release -- --determinism
//!       (runs the corpus 10x at depth 6 and asserts identical node counts)
//!
//!   cargo run -p search_bench --release -- --report --eval custom \
//!       --fen "<fen>"
//!       (prints the chosen evaluator's full per-piece + side-term breakdown for
//!        one FEN — no search. Omit --fen to dump every corpus position instead.)
//!
//! Output format: structured JSON (one object per position + an aggregate
//! block). Field names are intentionally stable; downstream diff tooling
//! reads them by name.

use core_engine::game_logic::action::Action;
use core_engine::game_logic::skills::{skill_from_id, skill_key};
use core_engine::search::alpha_beta::{find_best_with_evaluator, SearchResult};
use core_engine::search::counters::{self, Snapshot as CounterSnapshot, ATTACKER_LIST_HIST_BUCKETS};
use core_engine::search::evaluator::{
    builtin, evaluate_report, BreakdownDetail, Evaluator,
};
use core_engine::search::transposition::{Stats as TtStats, TranspositionTable};
use core_engine::state::Position;
use core_engine::state::fen::from_fen;

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

const DEFAULT_DEPTH: u8 = 6;
const DEFAULT_RUNS: usize = 5;
const TT_MB: usize = 64;

/// One corpus entry. Parsed from `corpus.txt`.
#[derive(Debug, Clone)]
struct CorpusEntry {
    id: String,
    category: String,
    /// Optional minimum depth at which the tactical assertion applies.
    expected_best_move_depth_n: Option<u8>,
    /// Optional inclusive score range the tactical assertion must lie in.
    expected_score_range: Option<(i32, i32)>,
    /// Optional list of acceptable best-move encodings (Action.0 raw u32).
    expected_best_moves: Vec<u32>,
    fen: String,
}

/// One measurement of one position in one mode.
#[derive(Debug, Clone)]
struct Measurement {
    nodes: u64,
    depth: u8,
    score: i32,
    best_move: Option<Action>,
    time_ms: f64,
    nodes_per_sec: f64,
    tt_probes: u64,
    tt_hits: u64,
    tt_hit_rate: f64,
    ebf: f64,
    counters: CounterSnapshot,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Depth(u8),
    Time(u64),
}

struct Args {
    corpus_path: PathBuf,
    mode: Mode,
    runs: usize,
    out_path: Option<PathBuf>,
    determinism: bool,
    determinism_runs: usize,
    eval_only: bool,
    eval_iterations: u64,
    eval_choice: EvalChoice,
    /// `--report`: print the evaluator's per-piece + side-term breakdown (no search).
    report: bool,
    /// Single FEN for `--report` mode; if `None`, `--report` iterates the corpus.
    fen: Option<String>,
    /// `--no-qs`: disable quiescence search at the depth-0 boundary (sets
    /// `alpha_beta::DISABLE_QS`). Used to measure whether QS is pulling its
    /// weight — the search-cliff work needs a QS-on vs QS-off A/B.
    no_qs: bool,
}

/// Which evaluator the search / report uses. `Builtin(id)` resolves through the
/// in-crate [`builtin::BUILTINS`] registry (so `custom`, `heuristic`, and any
/// future builtin work by id). `Nnue` is the ns-50 NNUE Phase-0 A/B lever: it
/// swaps in a quantized `NnueEvaluator` (refresh-per-call) so the search sweep
/// measures the NNUE eval cost as an NPS ratio vs the heuristic. For the fixed-
/// depth speed gate the net's weights are irrelevant (only forward cost), so
/// `nnue` builds a fresh in-process net.
#[derive(Clone, PartialEq, Eq)]
enum EvalChoice {
    Builtin(String),
    Nnue,
}

/// Build the evaluator for a run. Boxed so both variants share one `&dyn
/// Evaluator` path; constructed once per process, not per position. Builtin ids
/// resolve through the crate registry; an unknown id is a hard error listing the
/// available ids.
fn build_evaluator(choice: &EvalChoice) -> Box<dyn Evaluator> {
    match choice {
        EvalChoice::Builtin(id) => builtin::make(id).unwrap_or_else(|| {
            let ids: Vec<&str> = builtin::BUILTINS.iter().map(|b| b.id).collect();
            eprintln!("unknown --eval builtin '{id}'; available: {}, nnue", ids.join(", "));
            std::process::exit(2);
        }),
        EvalChoice::Nnue => {
            use nn_trainer::{Mlp, MlpConfig, NnueEvaluator, QuantScales, NUM_FEATURES};
            let device = Default::default();
            let model: Mlp<nn_trainer::InferenceBackend> =
                MlpConfig::new().with_input_dim(NUM_FEATURES).init(&device);
            Box::new(NnueEvaluator::from_mlp(&model, QuantScales::default()))
        }
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage: search_bench --corpus <path> (--depth N | --time-ms M) [--runs N] [--out <path>] [--eval <id>|nnue]");
    eprintln!("       search_bench --determinism [--corpus <path>] [--depth N] [--determinism-runs N] [--eval <id>|nnue]");
    eprintln!("       search_bench --eval-only [--corpus <path>] [--eval-iterations N] [--out <path>]");
    eprintln!("       search_bench --report [--eval <id>|nnue] (--fen \"<fen>\" | --corpus <path>)");
    eprintln!();
    eprintln!("Mode is inferred: --depth ⇒ depth mode; --time-ms ⇒ time mode.");
    eprintln!("Passing both, or neither, is an error (use --determinism / --eval-only / --report to opt out).");
    eprintln!("--eval selects the leaf evaluator: any builtin id (custom, heuristic, …) or 'nnue' (ns-50 A/B). Default custom.");
    eprintln!("--report prints the evaluator's per-piece + side-term breakdown for a FEN (or every corpus position) — no search.");
    eprintln!("--no-qs disables quiescence search (QS-on vs QS-off A/B); combine with --depth / --time-ms as usual.");
    std::process::exit(2);
}

fn parse_args() -> Args {
    let mut corpus_path = PathBuf::from("game/bench/corpus/corpus.txt");
    let mut depth: Option<u8> = None;
    let mut time_ms: Option<u64> = None;
    let mut runs = DEFAULT_RUNS;
    let mut out_path: Option<PathBuf> = None;
    let mut determinism = false;
    let mut determinism_runs = 10usize;
    let mut eval_only = false;
    let mut eval_iterations: u64 = 100_000;
    let mut eval_choice = EvalChoice::Builtin("custom".to_string());
    let mut report = false;
    let mut fen: Option<String> = None;
    let mut no_qs = false;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--corpus" => {
                corpus_path = PathBuf::from(argv.get(i + 1).cloned().unwrap_or_else(|| {
                    eprintln!("--corpus needs a path");
                    std::process::exit(2);
                }));
                i += 2;
            }
            "--depth" => {
                depth = Some(argv
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--depth needs a number");
                        std::process::exit(2);
                    }));
                i += 2;
            }
            "--time-ms" => {
                time_ms = Some(argv
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--time-ms needs a number");
                        std::process::exit(2);
                    }));
                i += 2;
            }
            "--runs" => {
                runs = argv
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--runs needs a number");
                        std::process::exit(2);
                    });
                i += 2;
            }
            "--out" => {
                out_path = Some(PathBuf::from(argv.get(i + 1).cloned().unwrap_or_else(|| {
                    eprintln!("--out needs a path");
                    std::process::exit(2);
                })));
                i += 2;
            }
            "--determinism" => {
                determinism = true;
                i += 1;
            }
            "--eval-only" => {
                eval_only = true;
                i += 1;
            }
            "--report" => {
                report = true;
                i += 1;
            }
            "--no-qs" => {
                no_qs = true;
                i += 1;
            }
            "--fen" => {
                fen = Some(argv.get(i + 1).cloned().unwrap_or_else(|| {
                    eprintln!("--fen needs a FEN string");
                    std::process::exit(2);
                }));
                i += 2;
            }
            "--eval-iterations" => {
                eval_iterations = argv
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--eval-iterations needs a number");
                        std::process::exit(2);
                    });
                i += 2;
            }
            "--determinism-runs" => {
                determinism_runs = argv
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--determinism-runs needs a number");
                        std::process::exit(2);
                    });
                i += 2;
            }
            "--eval" => {
                let v = argv.get(i + 1).cloned().unwrap_or_else(|| {
                    eprintln!("--eval needs a value (a builtin id, e.g. custom|heuristic, or nnue)");
                    std::process::exit(2);
                });
                eval_choice = match v.as_str() {
                    "nnue" => EvalChoice::Nnue,
                    // Any other value is a builtin id; validity is checked at
                    // construction (build_evaluator) so the error can list ids.
                    other => EvalChoice::Builtin(other.to_string()),
                };
                i += 2;
            }
            "--help" | "-h" => print_usage_and_exit(),
            other => {
                eprintln!("unknown arg: {}", other);
                print_usage_and_exit();
            }
        }
    }

    // Mode inference. Determinism runs at fixed depth (its own path); it may
    // take --depth but doesn't need a mode. Eval-only and report skip search.
    let mode = if determinism || eval_only || report {
        Mode::Depth(depth.unwrap_or(DEFAULT_DEPTH))
    } else {
        match (depth, time_ms) {
            (Some(_), Some(_)) => {
                eprintln!("error: pass --depth OR --time-ms, not both");
                print_usage_and_exit();
            }
            (Some(d), None) => Mode::Depth(d),
            (None, Some(t)) => Mode::Time(t),
            (None, None) => {
                eprintln!("error: must pass --depth or --time-ms (or --determinism / --eval-only)");
                print_usage_and_exit();
            }
        }
    };

    Args {
        corpus_path,
        mode,
        runs,
        out_path,
        determinism,
        determinism_runs,
        eval_only,
        eval_iterations,
        eval_choice,
        report,
        fen,
        no_qs,
    }
}

/// Parse `corpus.txt`. Format:
///
/// ```text
/// # Comment lines start with '#'. Blank lines ignored.
/// # Columns (5, comma-separated): id, category, expected_best_move_depth_N, expected_score_range, fen
/// # Use '-' for any unused expectation field. Score range syntax: "lo..hi" inclusive.
/// # Expected-best-move encodings can be appended after the FEN, comma-separated.
/// # The FEN itself contains spaces - its commas are the only commas after the score range.
/// # So we split on the FIRST FOUR commas, treat the rest as "fen [;move1,move2,...]" where
/// # the optional best-move list is appended after a `;` separator.
/// opening-01, opening, -, -, <fen> ; <action_u32>[,<action_u32>...]
/// mate-in-3-01, tactical, 6, -32700..-32600, <fen> ; 12345678
/// ```
fn load_corpus(path: &PathBuf) -> Vec<CorpusEntry> {
    let raw = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("failed to read corpus {}: {}", path.display(), e);
        std::process::exit(2);
    });

    let mut out = Vec::new();
    for (line_no, raw_line) in raw.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split on first four commas only - the FEN may contain spaces but no
        // commas before the optional ';' best-move trailer.
        let mut parts = Vec::with_capacity(5);
        let mut rest = line;
        for _ in 0..4 {
            let (head, tail) = match rest.split_once(',') {
                Some(x) => x,
                None => {
                    eprintln!("corpus line {} malformed (expected 5 cols): {}", line_no + 1, line);
                    std::process::exit(2);
                }
            };
            parts.push(head.trim().to_string());
            rest = tail.trim_start();
        }
        parts.push(rest.trim().to_string());

        let id = parts[0].clone();
        let category = parts[1].clone();
        let depth_n = if parts[2] == "-" {
            None
        } else {
            Some(parts[2].parse::<u8>().unwrap_or_else(|_| {
                eprintln!("corpus line {}: bad depth_n '{}'", line_no + 1, parts[2]);
                std::process::exit(2);
            }))
        };
        let score_range = if parts[3] == "-" {
            None
        } else {
            let (lo, hi) = parts[3].split_once("..").unwrap_or_else(|| {
                eprintln!("corpus line {}: bad score_range '{}'", line_no + 1, parts[3]);
                std::process::exit(2);
            });
            Some((
                lo.trim().parse::<i32>().expect("score_range lo"),
                hi.trim().parse::<i32>().expect("score_range hi"),
            ))
        };

        // Split optional best-move trailer.
        let (fen_str, best_move_trailer) = match parts[4].rsplit_once(';') {
            Some((f, t)) => (f.trim().to_string(), t.trim().to_string()),
            None => (parts[4].clone(), String::new()),
        };
        let expected_best_moves: Vec<u32> = if best_move_trailer.is_empty() {
            Vec::new()
        } else {
            best_move_trailer
                .split(',')
                .map(|s| s.trim().parse::<u32>())
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|e| {
                    eprintln!("corpus line {}: bad best-move trailer: {}", line_no + 1, e);
                    std::process::exit(2);
                })
        };

        out.push(CorpusEntry {
            id,
            category,
            expected_best_move_depth_n: depth_n,
            expected_score_range: score_range,
            expected_best_moves,
            fen: fen_str,
        });
    }
    out
}

fn one_run(pos_template: &Position, mode: Mode, evaluator: &dyn Evaluator) -> (Measurement, TtStats) {
    let mut pos = pos_template.clone();
    let mut tt = TranspositionTable::with_capacity_mb(TT_MB);
    counters::reset();
    let t0 = Instant::now();
    let (time_ms_arg, max_depth_arg) = match mode {
        Mode::Depth(d) => (0u64, d),
        Mode::Time(t) => (t, 64u8),
    };
    let sr: SearchResult =
        find_best_with_evaluator(&mut pos, &mut tt, time_ms_arg, max_depth_arg, evaluator, None);
    let elapsed = t0.elapsed().as_secs_f64();
    let stats = tt.stats();
    let counter_snap = counters::snapshot();

    let nps = if elapsed > 0.0 { sr.nodes as f64 / elapsed } else { 0.0 };
    let ebf = if sr.depth > 0 {
        (sr.nodes as f64).powf(1.0 / sr.depth as f64)
    } else {
        0.0
    };
    let hit_rate = if stats.probes > 0 {
        stats.hits as f64 / stats.probes as f64
    } else {
        0.0
    };

    let m = Measurement {
        nodes: sr.nodes,
        depth: sr.depth,
        score: sr.score,
        best_move: sr.best,
        time_ms: elapsed * 1000.0,
        nodes_per_sec: nps,
        tt_probes: stats.probes,
        tt_hits: stats.hits,
        tt_hit_rate: hit_rate,
        ebf,
        counters: counter_snap,
    };
    (m, stats)
}

fn median_measurement(samples: &[Measurement]) -> Measurement {
    assert!(!samples.is_empty());
    let mut by_nodes = samples.to_vec();
    by_nodes.sort_by_key(|m| m.nodes);
    let mid = by_nodes.len() / 2;
    by_nodes[mid].clone()
}

fn geometric_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_ln: f64 = values
        .iter()
        .filter(|v| **v > 0.0)
        .map(|v| v.ln())
        .sum();
    let n = values.iter().filter(|v| **v > 0.0).count();
    if n == 0 {
        return 0.0;
    }
    (sum_ln / n as f64).exp()
}

fn action_brief(a: Option<Action>) -> String {
    match a {
        None      => "(none)".to_string(),
        Some(act) => core_engine::action_to_notation(act, None),
    }
}

fn run_corpus(
    entries: &[CorpusEntry],
    mode: Mode,
    runs: usize,
    evaluator: &dyn Evaluator,
) -> (Vec<(String, Measurement)>, Vec<String>) {
    let mut results = Vec::with_capacity(entries.len());
    let mut regressions: Vec<String> = Vec::new();

    for entry in entries {
        let template = match from_fen(&entry.fen) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FEN parse failed for {}: {:?}", entry.id, e);
                std::process::exit(2);
            }
        };

        let mut samples = Vec::with_capacity(runs);
        for _ in 0..runs {
            let (m, _stats) = one_run(&template, mode, evaluator);
            samples.push(m);
        }
        let med = median_measurement(&samples);

        // Correctness assertions (apply to median).
        if let Some(n) = entry.expected_best_move_depth_n {
            if med.depth >= n {
                if !entry.expected_best_moves.is_empty() {
                    let bm_raw = med.best_move.map(|a| a.0).unwrap_or(0);
                    if !entry.expected_best_moves.contains(&bm_raw) {
                        regressions.push(format!(
                            "{}: REGRESSION best-move at depth {} (got raw=0x{:08x}, expected one of {:?})",
                            entry.id, med.depth, bm_raw, entry.expected_best_moves
                        ));
                    }
                }
                if let Some((lo, hi)) = entry.expected_score_range {
                    if med.score < lo || med.score > hi {
                        regressions.push(format!(
                            "{}: REGRESSION score at depth {} (got {}, expected [{}, {}])",
                            entry.id, med.depth, med.score, lo, hi
                        ));
                    }
                }
            }
        }

        println!(
            "{:30}  cat={:20}  d={:>2}  nodes={:>10}  nps={:>10.0}  time={:>7.1}ms  tt_hit={:>5.1}%  ebf={:>5.2}  best={}",
            entry.id,
            entry.category,
            med.depth,
            med.nodes,
            med.nodes_per_sec,
            med.time_ms,
            med.tt_hit_rate * 100.0,
            med.ebf,
            action_brief(med.best_move),
        );
        let c = &med.counters;
        let leaf = c.ab_nodes + c.qs_nodes;
        let gate_total = c.maee_gate_pass + c.maee_gate_skip;
        let attackers_mean = if c.enumerate_attackers_calls > 0 {
            c.attackers_total() as f64 / c.enumerate_attackers_calls as f64
        } else {
            0.0
        };
        println!(
            "  counters:  eval={} ab={} qs={} leaf_ratio_qs={:.2}  maee_pass/skip={}/{} ({:.1}%)  skill_pass/skip={}/{}  act0_hit={}  maee_side={} maee_target={} enum_att={} att_mean={:.2}  skill_act={}  see_tables={} see_calls={} see_per_qs={:.2}",
            c.eval_calls,
            c.ab_nodes,
            c.qs_nodes,
            if leaf > 0 { c.qs_nodes as f64 / leaf as f64 } else { 0.0 },
            c.maee_gate_pass,
            c.maee_gate_skip,
            if gate_total > 0 { 100.0 * c.maee_gate_pass as f64 / gate_total as f64 } else { 0.0 },
            c.skill_gate_pass,
            c.skill_gate_skip,
            c.actions_zero_hit,
            c.maee_side_calls,
            c.maee_target_calls,
            c.enumerate_attackers_calls,
            attackers_mean,
            c.skill_activity_calls,
            c.see_table_builds,
            c.see_capture_calls,
            if c.qs_nodes > 0 { c.see_capture_calls as f64 / c.qs_nodes as f64 } else { 0.0 },
        );

        results.push((entry.id.clone(), med));
    }

    (results, regressions)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Inline-serialize a CounterSnapshot as a JSON object. `indent` is the
/// leading whitespace already at the start of the caller's line - used for
/// nested object formatting.
fn write_counter_snapshot(s: &mut String, c: &CounterSnapshot, indent: &str) {
    s.push_str("{\n");
    s.push_str(&format!("{}  \"eval_calls\": {},\n", indent, c.eval_calls));
    s.push_str(&format!("{}  \"maee_gate_pass\": {},\n", indent, c.maee_gate_pass));
    s.push_str(&format!("{}  \"maee_gate_skip\": {},\n", indent, c.maee_gate_skip));
    s.push_str(&format!("{}  \"skill_gate_pass\": {},\n", indent, c.skill_gate_pass));
    s.push_str(&format!("{}  \"skill_gate_skip\": {},\n", indent, c.skill_gate_skip));
    s.push_str(&format!("{}  \"actions_zero_hit\": {},\n", indent, c.actions_zero_hit));
    s.push_str(&format!("{}  \"maee_side_calls\": {},\n", indent, c.maee_side_calls));
    s.push_str(&format!("{}  \"maee_target_calls\": {},\n", indent, c.maee_target_calls));
    s.push_str(&format!("{}  \"enumerate_attackers_calls\": {},\n", indent, c.enumerate_attackers_calls));
    s.push_str(&format!("{}  \"see_table_builds\": {},\n", indent, c.see_table_builds));
    s.push_str(&format!("{}  \"see_capture_calls\": {},\n", indent, c.see_capture_calls));
    s.push_str(&format!("{}  \"skill_activity_calls\": {},\n", indent, c.skill_activity_calls));
    s.push_str(&format!("{}  \"ab_nodes\": {},\n", indent, c.ab_nodes));
    s.push_str(&format!("{}  \"qs_nodes\": {},\n", indent, c.qs_nodes));
    s.push_str(&format!("{}  \"attacker_list_hist\": [", indent));
    for (i, v) in c.attacker_list_hist.iter().enumerate() {
        if i > 0 { s.push_str(", "); }
        s.push_str(&format!("{}", v));
    }
    s.push_str("]\n");
    s.push_str(&format!("{}}}", indent));
}

fn write_json(
    out_path: &PathBuf,
    mode: Mode,
    runs: usize,
    results: &[(String, Measurement)],
    entries_by_id: &BTreeMap<String, CorpusEntry>,
) {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"mode\": \"{}\",\n", match mode {
        Mode::Depth(_) => "depth",
        Mode::Time(_) => "time",
    }));
    match mode {
        Mode::Depth(d) => s.push_str(&format!("  \"depth\": {},\n", d)),
        Mode::Time(t) => s.push_str(&format!("  \"time_ms\": {},\n", t)),
    }
    s.push_str(&format!("  \"runs_per_position\": {},\n", runs));

    let nps_values: Vec<f64> = results.iter().map(|(_, m)| m.nodes_per_sec).collect();
    let depth_values: Vec<u8> = results.iter().map(|(_, m)| m.depth).collect();
    let geo_nps = geometric_mean(&nps_values);
    let depth_min = depth_values.iter().copied().min().unwrap_or(0);
    let depth_max = depth_values.iter().copied().max().unwrap_or(0);

    // Sum counters across all positions so aggregate reflects "over the whole
    // corpus run" rather than a mean of per-position values.
    let mut agg = CounterSnapshot::default();
    for (_, m) in results {
        agg.eval_calls += m.counters.eval_calls;
        agg.maee_gate_pass += m.counters.maee_gate_pass;
        agg.maee_gate_skip += m.counters.maee_gate_skip;
        agg.skill_gate_pass += m.counters.skill_gate_pass;
        agg.skill_gate_skip += m.counters.skill_gate_skip;
        agg.actions_zero_hit += m.counters.actions_zero_hit;
        agg.maee_side_calls += m.counters.maee_side_calls;
        agg.maee_target_calls += m.counters.maee_target_calls;
        agg.enumerate_attackers_calls += m.counters.enumerate_attackers_calls;
        agg.see_table_builds += m.counters.see_table_builds;
        agg.see_capture_calls += m.counters.see_capture_calls;
        agg.skill_activity_calls += m.counters.skill_activity_calls;
        agg.ab_nodes += m.counters.ab_nodes;
        agg.qs_nodes += m.counters.qs_nodes;
        for i in 0..ATTACKER_LIST_HIST_BUCKETS {
            agg.attacker_list_hist[i] += m.counters.attacker_list_hist[i];
        }
    }

    s.push_str("  \"aggregate\": {\n");
    s.push_str(&format!("    \"geometric_mean_nps\": {:.2},\n", geo_nps));
    s.push_str(&format!("    \"depth_min\": {},\n", depth_min));
    s.push_str(&format!("    \"depth_max\": {},\n", depth_max));
    s.push_str(&format!("    \"positions\": {},\n", results.len()));
    s.push_str("    \"counters\": ");
    write_counter_snapshot(&mut s, &agg, "    ");
    s.push_str("\n  },\n");

    s.push_str("  \"positions\": [\n");
    for (i, (id, m)) in results.iter().enumerate() {
        let cat = entries_by_id.get(id).map(|e| e.category.as_str()).unwrap_or("?");
        s.push_str("    {\n");
        s.push_str(&format!("      \"id\": \"{}\",\n", json_escape(id)));
        s.push_str(&format!("      \"category\": \"{}\",\n", json_escape(cat)));
        s.push_str(&format!("      \"nodes\": {},\n", m.nodes));
        s.push_str(&format!("      \"depth\": {},\n", m.depth));
        s.push_str(&format!("      \"score\": {},\n", m.score));
        s.push_str(&format!("      \"best_move_raw\": {},\n",
            m.best_move.map(|a| a.0).unwrap_or(0)));
        s.push_str(&format!("      \"time_ms\": {:.3},\n", m.time_ms));
        s.push_str(&format!("      \"nodes_per_sec\": {:.2},\n", m.nodes_per_sec));
        s.push_str(&format!("      \"tt_probes\": {},\n", m.tt_probes));
        s.push_str(&format!("      \"tt_hits\": {},\n", m.tt_hits));
        s.push_str(&format!("      \"tt_hit_rate\": {:.4},\n", m.tt_hit_rate));
        s.push_str(&format!("      \"ebf\": {:.3},\n", m.ebf));
        s.push_str("      \"counters\": ");
        write_counter_snapshot(&mut s, &m.counters, "      ");
        s.push('\n');
        s.push_str("    }");
        if i + 1 < results.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push_str("  ]\n");
    s.push_str("}\n");

    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = fs::File::create(out_path).unwrap_or_else(|e| {
        eprintln!("failed to create {}: {}", out_path.display(), e);
        std::process::exit(2);
    });
    f.write_all(s.as_bytes()).expect("write json");
    eprintln!("wrote {}", out_path.display());
}

fn run_determinism(entries: &[CorpusEntry], depth: u8, runs: usize, evaluator: &dyn Evaluator) {
    eprintln!("Determinism check: {} positions x {} runs each at depth {}", entries.len(), runs, depth);
    let mut all_ok = true;
    for entry in entries {
        let template = match from_fen(&entry.fen) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}: FEN parse failed: {:?}", entry.id, e);
                all_ok = false;
                continue;
            }
        };
        let mut first: Option<(u64, i32, u32)> = None;
        let mut ok = true;
        for r in 0..runs {
            let (m, _) = one_run(&template, Mode::Depth(depth), evaluator);
            let bm = m.best_move.map(|a| a.0).unwrap_or(0);
            match first {
                None => first = Some((m.nodes, m.score, bm)),
                Some((n, s, b)) => {
                    if m.nodes != n || m.score != s || m.best_move.map(|a| a.0).unwrap_or(0) != b {
                        ok = false;
                        eprintln!(
                            "{}: NON-DETERMINISTIC at run {} (nodes {}→{}, score {}→{}, best 0x{:08x}→0x{:08x})",
                            entry.id, r, n, m.nodes, s, m.score, b, bm
                        );
                        break;
                    }
                }
            }
        }
        if ok {
            println!("{:30}  ok  (nodes={} score={})", entry.id, first.unwrap().0, first.unwrap().1);
        } else {
            all_ok = false;
        }
    }
    if !all_ok {
        std::process::exit(3);
    }
    eprintln!("All positions deterministic.");
}

fn run_eval_only(entries: &[CorpusEntry], iterations: u64, out_path: Option<&PathBuf>) {
    eprintln!(
        "Eval-only bench: {} positions x {} iterations each",
        entries.len(),
        iterations
    );

    // Per-position results: (id, total_ns, ns_per_eval, checksum).
    // Checksum is the accumulated total-score to prevent the compiler from
    // hoisting the call out of the loop or DCE'ing the whole body.
    let mut per_pos: Vec<(String, u128, f64, i64)> = Vec::with_capacity(entries.len());

    for entry in entries {
        let pos = match from_fen(&entry.fen) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FEN parse failed for {}: {:?}", entry.id, e);
                std::process::exit(2);
            }
        };

        // Warm-up: a small handful of calls to prime caches before timing.
        let mut checksum: i64 = 0;
        for _ in 0..64 {
            let r = evaluate_report(&pos, BreakdownDetail::Aggregate);
            checksum = checksum.wrapping_add(r.total as i64);
        }

        let t0 = Instant::now();
        for _ in 0..iterations {
            let r = evaluate_report(&pos, BreakdownDetail::Aggregate);
            checksum = checksum.wrapping_add(r.total as i64);
        }
        let elapsed_ns = t0.elapsed().as_nanos();
        let ns_per_eval = if iterations > 0 {
            elapsed_ns as f64 / iterations as f64
        } else {
            0.0
        };

        println!(
            "{:30}  cat={:20}  iters={:>10}  total={:>10.3}ms  ns/eval={:>8.1}  cs={}",
            entry.id,
            entry.category,
            iterations,
            elapsed_ns as f64 / 1_000_000.0,
            ns_per_eval,
            checksum,
        );

        per_pos.push((entry.id.clone(), elapsed_ns, ns_per_eval, checksum));
    }

    let ns_values: Vec<f64> = per_pos.iter().map(|(_, _, ns, _)| *ns).collect();
    let geo_ns = geometric_mean(&ns_values);
    let sum_ns: f64 = ns_values.iter().sum();
    let mean_ns = if !ns_values.is_empty() { sum_ns / ns_values.len() as f64 } else { 0.0 };
    let min_ns = ns_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ns = ns_values.iter().cloned().fold(0.0f64, f64::max);

    eprintln!("---");
    eprintln!(
        "aggregate: positions={} iters/pos={}  ns/eval  min={:.1} mean={:.1} geo={:.1} max={:.1}",
        entries.len(),
        iterations,
        min_ns,
        mean_ns,
        geo_ns,
        max_ns,
    );

    if let Some(path) = out_path {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str("  \"mode\": \"eval-only\",\n");
        s.push_str(&format!("  \"iterations_per_position\": {},\n", iterations));
        s.push_str("  \"aggregate\": {\n");
        s.push_str(&format!("    \"positions\": {},\n", entries.len()));
        s.push_str(&format!("    \"ns_per_eval_min\": {:.3},\n", min_ns));
        s.push_str(&format!("    \"ns_per_eval_mean\": {:.3},\n", mean_ns));
        s.push_str(&format!("    \"ns_per_eval_geo\": {:.3},\n", geo_ns));
        s.push_str(&format!("    \"ns_per_eval_max\": {:.3}\n", max_ns));
        s.push_str("  },\n");
        s.push_str("  \"positions\": [\n");
        for (i, (id, total_ns, ns_per_eval, checksum)) in per_pos.iter().enumerate() {
            s.push_str("    {\n");
            s.push_str(&format!("      \"id\": \"{}\",\n", json_escape(id)));
            s.push_str(&format!("      \"total_ns\": {},\n", total_ns));
            s.push_str(&format!("      \"ns_per_eval\": {:.3},\n", ns_per_eval));
            s.push_str(&format!("      \"checksum\": {}\n", checksum));
            s.push_str("    }");
            if i + 1 < per_pos.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ]\n");
        s.push_str("}\n");

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut f = fs::File::create(path).unwrap_or_else(|e| {
            eprintln!("failed to create {}: {}", path.display(), e);
            std::process::exit(2);
        });
        f.write_all(s.as_bytes()).expect("write json");
        eprintln!("wrote {}", path.display());
    }
}

/// Square index → algebraic name (a1..h8). File = `sq % 8` (a..h left-to-right),
/// rank = `sq / 8 + 1` (rank 1 = P1's back rank, matching the FEN convention).
fn sq_name(sq: u8) -> String {
    let file = (b'a' + (sq % 8)) as char;
    let rank = sq / 8 + 1;
    format!("{file}{rank}")
}

/// `PieceTermBreakdown.piece_kind` → label (1 = Guard, 2 = Champion, 3 = King).
fn kind_name(piece_kind: u8) -> &'static str {
    match piece_kind {
        3 => "King",
        2 => "Champ",
        1 => "Guard",
        _ => "?",
    }
}

/// Two skill ids → a readable `key1,key2` (0 = empty slot). `-` if both empty.
fn skills_str(id1: u8, id2: u8) -> String {
    let mut parts = Vec::new();
    for id in [id1, id2] {
        if let Some(s) = skill_from_id(id) {
            parts.push(skill_key(s));
        }
    }
    if parts.is_empty() { "-".to_string() } else { parts.join(",") }
}

/// Print the chosen evaluator's full per-piece + side-term breakdown for one
/// position — the designer's tuning view. Uses `BreakdownDetail::PerPiece` so the
/// per-square owner-signed values and every side term are exposed, and closes with
/// a CHECK line proving `Σ piece_total + Σ side.signed == total == evaluate()`.
fn run_report(fen: &str, pos: &Position, eval_label: &str, evaluator: &dyn Evaluator) {
    let report = evaluator.evaluate_report(pos, BreakdownDetail::PerPiece);
    let total = evaluator.evaluate(pos);

    println!("FEN: {fen}");
    println!(
        "evaluator: {eval_label}   total (P1-POV): {}   terminal: {}",
        report.total, report.terminal,
    );

    if report.terminal {
        println!("(terminal position — no term breakdown)");
        return;
    }

    // --- PIECES: owner-signed value per occupied square. ---
    println!();
    println!("PIECES  (owner-signed value; + favours P1, - favours P2)");
    println!("  {:<4} {:<6} {:<5} {:<7} {:<16} {:>7}",
        "sq", "piece", "owner", "hp/arm", "skills", "value");
    let mut piece_subtotal = 0i32;
    if let Some(pieces) = report.pieces.as_ref() {
        for pc in pieces {
            piece_subtotal += pc.piece_total;
            println!(
                "  {:<4} {:<6} {:<5} {:<7} {:<16} {:>+7}",
                sq_name(pc.sq),
                kind_name(pc.piece_kind),
                if pc.is_p1 { "P1" } else { "P2" },
                format!("{}/{}", pc.hp, pc.armor),
                skills_str(pc.skill1_id, pc.skill2_id),
                pc.piece_total,
            );
            // Per-factor breakdown for this piece (base × factor deltas + malus),
            // so tuning can see WHICH factor produced the value, not just the sum.
            if !pc.terms.is_empty() {
                let detail: Vec<String> = pc.terms.iter()
                    .map(|t| format!("{} {:+}", t.label, t.signed))
                    .collect();
                println!("        └ {}", detail.join(" · "));
            }
        }
    }
    println!("  -- piece subtotal: {:+}", piece_subtotal);

    // --- SIDE TERMS: per-side magnitudes + signed contribution. ---
    println!();
    println!("SIDE TERMS");
    println!("  {:<18} {:>6} {:>6} {:>8}", "name", "p1", "p2", "signed");
    let mut side_subtotal = 0i32;
    for t in &report.side_terms {
        side_subtotal += t.signed;
        println!("  {:<18} {:>6} {:>6} {:>+8}", t.name, t.p1, t.p2, t.signed);
    }
    println!("  -- side subtotal: {:+}", side_subtotal);

    // --- CHECK: the reconstruction invariant, made visible. ---
    println!();
    println!(
        "CHECK: piece_subtotal + side_subtotal = {:+}  (evaluate() = {:+})",
        piece_subtotal + side_subtotal,
        total,
    );
}

fn main() {
    let args = parse_args();

    // --no-qs: disable quiescence globally before any search runs. This is the
    // QS-on vs QS-off A/B lever (search-cliff investigation). Set once here.
    if args.no_qs {
        core_engine::search::alpha_beta::DISABLE_QS
            .store(true, std::sync::atomic::Ordering::Relaxed);
        eprintln!("QS disabled (--no-qs): quiescence search off at depth-0 boundary");
    }

    let evaluator = build_evaluator(&args.eval_choice);
    let eval_label = match &args.eval_choice {
        EvalChoice::Builtin(id) => id.clone(),
        EvalChoice::Nnue => "nnue".to_string(),
    };
    if args.eval_choice == EvalChoice::Nnue {
        eprintln!("evaluator: NNUE (quantized, refresh-per-call) - ns-50 A/B");
    }

    // --report: static per-piece + side-term breakdown, no search. A single
    // --fen skips the corpus entirely; otherwise every corpus position is dumped.
    if args.report {
        if let Some(fen) = &args.fen {
            let pos = from_fen(fen).unwrap_or_else(|e| {
                eprintln!("FEN parse failed: {:?}", e);
                std::process::exit(2);
            });
            run_report(fen, &pos, &eval_label, evaluator.as_ref());
        } else {
            let entries = load_corpus(&args.corpus_path);
            if entries.is_empty() {
                eprintln!("corpus is empty: {}", args.corpus_path.display());
                std::process::exit(2);
            }
            for entry in &entries {
                let pos = from_fen(&entry.fen).unwrap_or_else(|e| {
                    eprintln!("FEN parse failed for {}: {:?}", entry.id, e);
                    std::process::exit(2);
                });
                println!("=== {} ({}) ===", entry.id, entry.category);
                run_report(&entry.fen, &pos, &eval_label, evaluator.as_ref());
                println!();
            }
        }
        return;
    }

    let entries = load_corpus(&args.corpus_path);
    if entries.is_empty() {
        eprintln!("corpus is empty: {}", args.corpus_path.display());
        std::process::exit(2);
    }
    eprintln!("loaded {} positions from {}", entries.len(), args.corpus_path.display());

    if args.determinism {
        let depth = match args.mode {
            Mode::Depth(d) => d,
            Mode::Time(_) => DEFAULT_DEPTH,
        };
        run_determinism(&entries, depth, args.determinism_runs, evaluator.as_ref());
        return;
    }

    if args.eval_only {
        run_eval_only(&entries, args.eval_iterations, args.out_path.as_ref());
        return;
    }

    let entries_by_id: BTreeMap<String, CorpusEntry> =
        entries.iter().map(|e| (e.id.clone(), e.clone())).collect();
    let (results, regressions) = run_corpus(&entries, args.mode, args.runs, evaluator.as_ref());

    if let Some(path) = &args.out_path {
        write_json(path, args.mode, args.runs, &results, &entries_by_id);
    }

    if !regressions.is_empty() {
        eprintln!("---");
        for r in &regressions {
            eprintln!("{}", r);
        }
        std::process::exit(4);
    }
}
