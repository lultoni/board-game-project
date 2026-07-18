//! Branching-factor profiler for Stack M - Layer-1 data for `oq-81`.
//!
//! Plays N uniform-random games from `Position::setup_stack_m()`, records the
//! legal-action count at every ply, and prints three markdown tables:
//! Phase × round bucket, Phase overall, Skill-Phase × pending modifiers.
//!
//! Usage:
//!   cargo run -p core_engine --example branching_profile --release
//!   cargo run -p core_engine --example branching_profile --release -- \
//!       --games 1000 --seed 42 --csv /tmp/samples.csv

use core_engine::game_logic::{generator, make_unmake};
use core_engine::state::Position;
use core_engine::state::position::{Phase, modifier_bits};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use std::fs::File;
use std::io::{BufWriter, Write};

const MAX_PLIES: usize = 5_000;
const DEFAULT_GAMES: usize = 500;
const DEFAULT_SEED:  u64   = 0x426F_6172_6447_616D; // "BoardGam" - Zobrist seed

#[derive(Clone, Copy)]
struct Sample {
    phase: Phase,
    round: u16,
    legal: u32,
    focus:  bool,
    charge: bool,
}

/// Play one game to terminal (or to `MAX_PLIES`), appending one Sample per
/// ply. Returns `(reached_terminal, plies)`.
fn play_one(seed: u64, samples: &mut Vec<Sample>) -> (bool, usize) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut pos = Position::setup_stack_m();
    let mut plies = 0usize;

    while plies < MAX_PLIES {
        if pos.game_result.is_some() { return (true, plies); }

        let moves = generator::generate(&pos);
        if moves.is_empty() {
            // Non-terminal with no actions = generator bug (should never fire).
            // The Move-Phase generator always emits at least EndPhase.
            eprintln!("WARN: empty action set at non-terminal ply {} (round {} phase {:?})",
                      plies, pos.round_number, pos.current_phase);
            return (false, plies);
        }

        samples.push(Sample {
            phase: pos.current_phase,
            round: pos.round_number,
            legal: moves.len() as u32,
            focus:  pos.pending_modifiers & modifier_bits::FOCUS  != 0,
            charge: pos.pending_modifiers & modifier_bits::CHARGE != 0,
        });

        let pick = moves.choose(&mut rng).copied().unwrap();
        let _undo = make_unmake::make(&mut pos, pick);
        plies += 1;
    }

    (false, plies)
}

fn percentile(sorted: &[u32], q: f64) -> u32 {
    if sorted.is_empty() { return 0; }
    // nearest-rank, 1-indexed: ceil(q * n)
    let n = sorted.len();
    let k = ((q * n as f64).ceil() as usize).max(1);
    sorted[k.min(n) - 1]
}

fn stats(values: &[u32]) -> (usize, f64, u32, u32, u32) {
    if values.is_empty() { return (0, 0.0, 0, 0, 0); }
    let n = values.len();
    let sum: u64 = values.iter().map(|&v| v as u64).sum();
    let mean = sum as f64 / n as f64;
    let mut sorted: Vec<u32> = values.to_vec();
    sorted.sort_unstable();
    let p50 = percentile(&sorted, 0.50);
    let p95 = percentile(&sorted, 0.95);
    let max = *sorted.last().unwrap();
    (n, mean, p50, p95, max)
}

fn round_bucket(r: u16) -> &'static str {
    match r {
        0..=5   => "1–5",
        6..=10  => "6–10",
        11..=20 => "11–20",
        21..=40 => "21–40",
        _       => "41+",
    }
}

const BUCKETS: [&str; 5] = ["1–5", "6–10", "11–20", "21–40", "41+"];

fn print_phase_round_table(samples: &[Sample]) {
    println!("### Phase × Round bucket");
    println!("| Phase | Round    |       n |   mean |  p50 |  p95 |  max |");
    println!("|-------|----------|---------|--------|------|------|------|");
    for phase in [Phase::Move, Phase::Skill] {
        for &bucket in &BUCKETS {
            let vals: Vec<u32> = samples.iter()
                .filter(|s| s.phase == phase && round_bucket(s.round) == bucket)
                .map(|s| s.legal)
                .collect();
            let (n, mean, p50, p95, max) = stats(&vals);
            println!("| {:<5} | {:<8} | {:>7} | {:>6.2} | {:>4} | {:>4} | {:>4} |",
                     phase_str(phase), bucket, n, mean, p50, p95, max);
        }
    }
    println!();
}

fn print_phase_overall_table(samples: &[Sample]) {
    println!("### Phase overall");
    println!("| Phase |       n |   mean |  p50 |  p95 |  max |");
    println!("|-------|---------|--------|------|------|------|");
    for phase in [Phase::Move, Phase::Skill] {
        let vals: Vec<u32> = samples.iter()
            .filter(|s| s.phase == phase)
            .map(|s| s.legal)
            .collect();
        let (n, mean, p50, p95, max) = stats(&vals);
        println!("| {:<5} | {:>7} | {:>6.2} | {:>4} | {:>4} | {:>4} |",
                 phase_str(phase), n, mean, p50, p95, max);
    }
    println!();
}

fn print_modifier_table(samples: &[Sample]) {
    println!("### Skill Phase × pending modifiers");
    println!("| Focus | Charge |       n |   mean |  p50 |  p95 |  max |");
    println!("|-------|--------|---------|--------|------|------|------|");
    for &focus in &[false, true] {
        for &charge in &[false, true] {
            let vals: Vec<u32> = samples.iter()
                .filter(|s| s.phase == Phase::Skill && s.focus == focus && s.charge == charge)
                .map(|s| s.legal)
                .collect();
            let (n, mean, p50, p95, max) = stats(&vals);
            println!("| {:^5} | {:^6} | {:>7} | {:>6.2} | {:>4} | {:>4} | {:>4} |",
                     yn(focus), yn(charge), n, mean, p50, p95, max);
        }
    }
    println!();
}

fn phase_str(p: Phase) -> &'static str {
    match p { Phase::Move => "Move", Phase::Skill => "Skill", Phase::Draft => "Draft" }
}
fn yn(b: bool) -> &'static str { if b { "yes" } else { "no" } }

fn write_csv(path: &str, samples: &[Sample]) -> std::io::Result<()> {
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);
    writeln!(w, "phase,round,legal,focus,charge")?;
    for s in samples {
        writeln!(w, "{},{},{},{},{}",
                 phase_str(s.phase), s.round, s.legal,
                 if s.focus  { 1 } else { 0 },
                 if s.charge { 1 } else { 0 })?;
    }
    Ok(())
}

fn parse_args() -> (usize, u64, Option<String>) {
    let mut games = DEFAULT_GAMES;
    let mut seed  = DEFAULT_SEED;
    let mut csv: Option<String> = None;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--games" => {
                games = args.get(i+1).and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| { eprintln!("--games needs a number"); std::process::exit(2); });
                i += 2;
            }
            "--seed" => {
                let s = args.get(i+1).cloned()
                    .unwrap_or_else(|| { eprintln!("--seed needs a value"); std::process::exit(2); });
                seed = if let Some(hex) = s.strip_prefix("0x") {
                    u64::from_str_radix(hex, 16).expect("invalid hex seed")
                } else {
                    s.parse().expect("invalid decimal seed")
                };
                i += 2;
            }
            "--csv" => {
                csv = Some(args.get(i+1).cloned()
                    .unwrap_or_else(|| { eprintln!("--csv needs a path"); std::process::exit(2); }));
                i += 2;
            }
            other => { eprintln!("unknown arg: {}", other); std::process::exit(2); }
        }
    }
    (games, seed, csv)
}

fn main() {
    let (n_games, seed, csv) = parse_args();

    let t0 = std::time::Instant::now();
    let mut samples: Vec<Sample> = Vec::with_capacity(n_games * 400);
    let mut terminated = 0usize;
    let mut cap_hits = 0usize;
    let mut total_plies = 0usize;

    for g in 0..n_games {
        let (term, plies) = play_one(seed.wrapping_add(g as u64), &mut samples);
        if term { terminated += 1; } else { cap_hits += 1; }
        total_plies += plies;
    }
    let elapsed = t0.elapsed();

    println!("## Stack M branching factor - N={} games, seed=0x{:016X}", n_games, seed);
    println!("games: {}  terminal: {}  cap-hits: {}  samples: {}  total-plies: {}  wall: {:.2}s",
             n_games, terminated, cap_hits, samples.len(), total_plies, elapsed.as_secs_f64());
    if cap_hits as f64 / n_games as f64 > 0.01 {
        eprintln!("WARN: cap-hit rate {:.2}% > 1% - investigate generator/economy",
                  100.0 * cap_hits as f64 / n_games as f64);
    }
    println!();

    print_phase_round_table(&samples);
    print_phase_overall_table(&samples);
    print_modifier_table(&samples);

    if let Some(path) = csv {
        match write_csv(&path, &samples) {
            Ok(()) => eprintln!("wrote {} samples to {}", samples.len(), path),
            Err(e) => eprintln!("csv write failed: {}", e),
        }
    }
}
