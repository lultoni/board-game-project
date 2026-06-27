//! Head-to-head play-strength match: QS-engine vs baseline (no-QS).
//!
//! Toggles `alpha_beta::DISABLE_QS` per side at every search call. Reports
//! win-count after N games. Swaps sides every game to net out first-mover
//! bias.
//!
//! Usage:
//!   cargo run -p core_engine --example qs_match --release -- \
//!       --games 100 --time-ms 1000

use core_engine::search::alpha_beta::DISABLE_QS;
use core_engine::state::position::{GameResult, Player};
use core_engine::{AiBudget, Config, Match};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::time::{SystemTime, UNIX_EPOCH};

struct Args {
    games:    usize,
    time_ms:  u64,
    depth:    u8,
    max_plies: usize,
}

fn parse() -> Args {
    let mut a = Args { games: 20, time_ms: 1000, depth: 64, max_plies: 500 };
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        let key = raw[i].as_str();
        let val = || raw.get(i+1).cloned().expect("missing value");
        match key {
            "--games"      => { a.games = val().parse().unwrap(); i += 2; }
            "--time-ms"    => { a.time_ms = val().parse().unwrap(); i += 2; }
            "--depth"      => { a.depth = val().parse().unwrap(); i += 2; }
            "--max-plies"  => { a.max_plies = val().parse().unwrap(); i += 2; }
            other          => { eprintln!("unknown arg {}", other); std::process::exit(2); }
        }
    }
    a
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Play one game. `qs_side` plays with QS enabled; the other side plays with
/// QS disabled. Returns `Some(GameResult)` if a side won, or `None` for cap.
fn play_one(qs_side: Player, time_ms: u64, depth: u8, max_plies: usize) -> Option<GameResult> {
    let mut cfg = Config::local_aivai();
    cfg.p1_ai = AiBudget { time_limit_ms: time_ms, max_depth: depth };
    cfg.p2_ai = AiBudget { time_limit_ms: time_ms, max_depth: depth };
    cfg.auto_log = false;
    let mut m = Match::new_with_clock(cfg, now_ms());
    let mut plies = 0usize;
    let mut n_draft = 0usize;
    let mut n_move = 0usize;
    let mut n_skill = 0usize;
    let mut n_endphase = 0usize;
    let mut n_endturn = 0usize;
    let mut n_other = 0usize;
    while plies < max_plies && m.game_result().is_none() {
        // Toggle DISABLE_QS *before* the search runs.
        let mover = m.position().to_move;
        let qs_on = mover == qs_side;
        DISABLE_QS.store(!qs_on, AtomicOrdering::Relaxed);
        let r = m.step_ai().expect("ai step");
        if let Some(a) = r.best {
            if a.is_draft_turn() { n_draft += 1; }
            else { match a.kind() {
                core_engine::game_logic::action::ActionKind::Move     => n_move += 1,
                core_engine::game_logic::action::ActionKind::Skill    => n_skill += 1,
                core_engine::game_logic::action::ActionKind::EndPhase => n_endphase += 1,
                core_engine::game_logic::action::ActionKind::EndTurn  => n_endturn += 1,
            }}
        } else {
            n_other += 1;
        }
        plies += 1;
    }
    let round = m.position().round_number;
    println!("    plies: draft={} move={} skill={} endphase={} endturn={} other={} | final_round={}",
             n_draft, n_move, n_skill, n_endphase, n_endturn, n_other, round);
    DISABLE_QS.store(false, AtomicOrdering::Relaxed);
    m.game_result()
}

fn main() {
    let args = parse();
    println!("# QS vs no-QS match");
    println!("  games={}  time-ms={}  depth-cap={}  max-plies={}",
             args.games, args.time_ms, args.depth, args.max_plies);
    println!();

    let mut qs_wins   = 0;
    let mut base_wins = 0;
    let mut draws_or_caps = 0;

    for g in 0..args.games {
        // Alternate: even games QS=P1, odd games QS=P2.
        let qs_side = if g % 2 == 0 { Player::P1 } else { Player::P2 };
        let r = play_one(qs_side, args.time_ms, args.depth, args.max_plies);
        let outcome = match (r, qs_side) {
            (Some(GameResult::P1Wins), Player::P1) => { qs_wins += 1; "QS wins (as P1)" }
            (Some(GameResult::P2Wins), Player::P2) => { qs_wins += 1; "QS wins (as P2)" }
            (Some(GameResult::P1Wins), Player::P2) => { base_wins += 1; "base wins (as P1)" }
            (Some(GameResult::P2Wins), Player::P1) => { base_wins += 1; "base wins (as P2)" }
            (None, _) => { draws_or_caps += 1; "cap" }
        };
        println!("  game {:>3}/{}: QS={:?}  -> {}",
                 g + 1, args.games,
                 qs_side, outcome);
    }

    let decisive = qs_wins + base_wins;
    let pct_qs = if decisive > 0 { 100.0 * qs_wins as f64 / decisive as f64 } else { 0.0 };
    println!();
    println!("Summary: QS={} base={} caps={}", qs_wins, base_wins, draws_or_caps);
    if decisive > 0 {
        println!("Decisive win-rate (QS): {:.1}% ({}/{})", pct_qs, qs_wins, decisive);
    }
}
