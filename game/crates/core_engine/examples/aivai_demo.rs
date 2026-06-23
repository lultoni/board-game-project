//! L4 smoke test — AIvAI to completion.
//!
//! Builds `Config::local_aivai()`, optionally overrides per-side time/depth
//! budgets from CLI, then loops `step_ai` until the game ends or `max_plies`
//! is reached. Prints a compact move list and the final result.
//!
//! Usage:
//!   cargo run -p core_engine --example aivai_demo --release
//!   cargo run -p core_engine --example aivai_demo --release -- \
//!       --p1-ms 200 --p2-ms 200 --p1-depth 6 --p2-depth 6 --max-plies 500

use core_engine::game_logic::action::{Action, ActionKind};
use core_engine::{AiBudget, Config, Match, SeatKind};

const DEFAULT_MS:    u64   = 200;
const DEFAULT_DEPTH: u8    = 6;
const DEFAULT_PLIES: usize = 500;

struct Args {
    p1_ms: u64, p2_ms: u64,
    p1_depth: u8, p2_depth: u8,
    max_plies: usize,
}

fn parse_args() -> Args {
    let mut a = Args {
        p1_ms: DEFAULT_MS, p2_ms: DEFAULT_MS,
        p1_depth: DEFAULT_DEPTH, p2_depth: DEFAULT_DEPTH,
        max_plies: DEFAULT_PLIES,
    };
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        let key = raw[i].as_str();
        let val = || raw.get(i+1).cloned().unwrap_or_else(|| {
            eprintln!("missing value for {}", key); std::process::exit(2);
        });
        match key {
            "--p1-ms"     => { a.p1_ms     = val().parse().expect("u64"); i += 2; }
            "--p2-ms"     => { a.p2_ms     = val().parse().expect("u64"); i += 2; }
            "--p1-depth"  => { a.p1_depth  = val().parse().expect("u8");  i += 2; }
            "--p2-depth"  => { a.p2_depth  = val().parse().expect("u8");  i += 2; }
            "--max-plies" => { a.max_plies = val().parse().expect("usize"); i += 2; }
            other => { eprintln!("unknown arg: {}", other); std::process::exit(2); }
        }
    }
    a
}

fn fmt_action(a: Action) -> String {
    match a.kind() {
        ActionKind::Move     => format!("Move {}→{}",       a.src(), a.target()),
        ActionKind::Skill    => format!("Skill[{}] {}→{}",  a.skill_id(), a.src(), a.target()),
        ActionKind::EndPhase => "EndPhase".to_string(),
        ActionKind::EndTurn  => "EndTurn".to_string(),
    }
}

fn main() {
    let args = parse_args();

    let mut cfg = Config::local_aivai();
    cfg.p1_ai = AiBudget { time_limit_ms: args.p1_ms, max_depth: args.p1_depth };
    cfg.p2_ai = AiBudget { time_limit_ms: args.p2_ms, max_depth: args.p2_depth };

    let mut m = Match::new(cfg);
    println!("# AIvAI demo");
    println!("  P1: {:?}  budget: {} ms / depth {}", SeatKind::Ai, args.p1_ms, args.p1_depth);
    println!("  P2: {:?}  budget: {} ms / depth {}", SeatKind::Ai, args.p2_ms, args.p2_depth);
    println!();

    let t0 = std::time::Instant::now();
    let mut plies = 0usize;
    let mut last_score: i32 = 0;
    let mut total_nodes: u64 = 0;
    while plies < args.max_plies && m.game_result().is_none() {
        let r = m.step_ai().expect("AI step");
        last_score = r.score;
        total_nodes = total_nodes.saturating_add(r.nodes);
        if let Some(a) = r.best {
            println!("{:>4}. {}  (score={}, depth={}, nodes={})",
                     plies + 1, fmt_action(a), r.score, r.depth, r.nodes);
        }
        plies += 1;
    }
    let wall = t0.elapsed();

    println!();
    println!("plies: {}  wall: {:.2}s  cumulative nodes: {}  final score: {}",
             plies, wall.as_secs_f64(), total_nodes, last_score);
    match m.game_result() {
        Some(r) => println!("result: {:?}", r),
        None    => println!("result: cap hit (no winner within {} plies)", args.max_plies),
    }
}
