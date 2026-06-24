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
//!
//! Watch mode:
//!   cargo run -p core_engine --example aivai_demo --release -- \
//!       --show-board --step              # press Enter to advance each ply
//!   cargo run -p core_engine --example aivai_demo --release -- \
//!       --show-board --step-delay-ms 400 # auto-advance every 400 ms
//!
//! Flags:
//!   --show-board        ASCII 8×8 board + full per-piece state after every action
//!   --show-fen          one-line FEN after every action
//!   --no-color          disable ANSI color
//!   --step              wait for Enter between plies (overrides --step-delay-ms)
//!   --step-delay-ms N   sleep N ms between plies

use core_engine::game_logic::action::{Action, ActionKind};
use core_engine::game_logic::skills::skill_from_id;
use core_engine::state::Position;
use core_engine::state::fen::to_fen;
use core_engine::state::position::{Phase, Player, modifier_bits};
use core_engine::telemetry::{notation, to_json, to_json_pretty, MatchResult};
use core_engine::{AiBudget, Config, Match, SeatKind};

use std::io::{BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_MS:    u64   = 200;
const DEFAULT_DEPTH: u8    = 6;
const DEFAULT_PLIES: usize = 500;

// ANSI escapes
const RESET:  &str = "\x1b[0m";
const BOLD:   &str = "\x1b[1m";
const DIM:    &str = "\x1b[2m";
const RED:    &str = "\x1b[31m";   // P1
const BLUE:   &str = "\x1b[34m";   // P2
const YELLOW: &str = "\x1b[33m";
const CYAN:   &str = "\x1b[36m";

struct Args {
    p1_ms: u64, p2_ms: u64,
    p1_depth: u8, p2_depth: u8,
    max_plies: usize,
    show_board: bool,
    show_fen: bool,
    color: bool,
    step: bool,
    step_delay_ms: u64,
    export_json: Option<String>,
    export_json_pretty: Option<String>,
    export_notation: Option<String>,
    note: Option<String>,
}

fn parse_args() -> Args {
    let mut a = Args {
        p1_ms: DEFAULT_MS, p2_ms: DEFAULT_MS,
        p1_depth: DEFAULT_DEPTH, p2_depth: DEFAULT_DEPTH,
        max_plies: DEFAULT_PLIES,
        show_board: false,
        show_fen: false,
        color: true,
        step: false,
        step_delay_ms: 0,
        export_json: None,
        export_json_pretty: None,
        export_notation: None,
        note: None,
    };
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        let key = raw[i].as_str();
        let val = || raw.get(i+1).cloned().unwrap_or_else(|| {
            eprintln!("missing value for {}", key); std::process::exit(2);
        });
        match key {
            "--p1-ms"         => { a.p1_ms     = val().parse().expect("u64"); i += 2; }
            "--p2-ms"         => { a.p2_ms     = val().parse().expect("u64"); i += 2; }
            "--p1-depth"      => { a.p1_depth  = val().parse().expect("u8");  i += 2; }
            "--p2-depth"      => { a.p2_depth  = val().parse().expect("u8");  i += 2; }
            "--max-plies"     => { a.max_plies = val().parse().expect("usize"); i += 2; }
            "--show-board"    => { a.show_board = true; i += 1; }
            "--show-fen"      => { a.show_fen = true; i += 1; }
            "--no-color"      => { a.color = false; i += 1; }
            "--step"          => { a.step = true; i += 1; }
            "--step-delay-ms" => { a.step_delay_ms = val().parse().expect("u64"); i += 2; }
            "--export-json"        => { a.export_json        = Some(val()); i += 2; }
            "--export-json-pretty" => { a.export_json_pretty = Some(val()); i += 2; }
            "--export-notation"    => { a.export_notation    = Some(val()); i += 2; }
            "--note"               => { a.note               = Some(val()); i += 2; }
            other => { eprintln!("unknown arg: {}", other); std::process::exit(2); }
        }
    }
    a
}

fn paint(s: &str, code: &str, color: bool) -> String {
    if color { format!("{}{}{}", code, s, RESET) } else { s.to_string() }
}

fn fmt_action(a: Action, color: bool) -> String {
    let body = match a.kind() {
        ActionKind::Move     => format!("Move {}→{}",       sq_name(a.src()), sq_name(a.target())),
        ActionKind::Skill    => {
            let name = skill_from_id(a.skill_id())
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| format!("Skill#{}", a.skill_id()));
            format!("{} {}→{}", name, sq_name(a.src()), sq_name(a.target()))
        }
        ActionKind::EndPhase => "EndPhase".to_string(),
        ActionKind::EndTurn  => "EndTurn".to_string(),
    };
    paint(&body, BOLD, color)
}

fn sq_name(sq: u8) -> String {
    if sq >= 64 { return format!("?{}", sq); }
    let file = (b'a' + (sq % 8)) as char;
    let rank = (sq / 8) + 1;
    format!("{}{}", file, rank)
}

fn skill_short(id: u8) -> String {
    if id == 0 { return "--".to_string(); }
    match skill_from_id(id) {
        Some(s) => {
            let n = format!("{:?}", s);
            n.chars().take(4).collect()
        }
        None => format!("?{}", id),
    }
}

/// Per-square colored glyph for the board grid: e.g. "K4" (King with 4 HP).
/// Width is exactly 4 chars of *visible* content (incl. trailing space).
fn cell_glyph(pos: &Position, sq: u8, color: bool) -> String {
    if !pos.is_occupied(sq) {
        return " .  ".to_string();
    }
    let is_p1 = pos.p1_pieces.contains(sq);
    let base = if pos.kings.contains(sq) {
        if is_p1 { 'K' } else { 'k' }
    } else if pos.champions.contains(sq) {
        if is_p1 { 'C' } else { 'c' }
    } else {
        if is_p1 { 'G' } else { 'g' }
    };
    let hp = pos.mailbox[sq as usize].hp();
    let raw = format!(" {}{} ", base, hp);
    if color {
        let c = if is_p1 { RED } else { BLUE };
        format!("{}{}{}{}", BOLD, c, raw, RESET)
    } else {
        raw
    }
}

fn render_board(pos: &Position, color: bool) -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("     a    b    c    d    e    f    g    h\n");
    out.push_str("   +----+----+----+----+----+----+----+----+\n");
    for rank_top in 0..8u8 {
        let rank_idx = 7 - rank_top;
        out.push_str(&format!(" {} |", rank_idx + 1));
        for file in 0..8u8 {
            let sq = rank_idx * 8 + file;
            out.push_str(&cell_glyph(pos, sq, color));
            out.push('|');
        }
        out.push_str(&format!(" {}\n", rank_idx + 1));
        out.push_str("   +----+----+----+----+----+----+----+----+\n");
    }
    out.push_str("     a    b    c    d    e    f    g    h\n");
    out
}

fn render_piece_table(pos: &Position, color: bool) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str("\n  Pieces (Sq | Owner | Type | HP | Armor | Combo | Skill1 | Skill2):\n");
    // Walk both players, then within each: King, Champions, Guards.
    for player in [Player::P1, Player::P2] {
        let label = match player { Player::P1 => "P1", Player::P2 => "P2" };
        let code  = if matches!(player, Player::P1) { RED } else { BLUE };
        for sq in 0..64u8 {
            if !pos.is_occupied(sq) { continue; }
            let is_p1 = pos.p1_pieces.contains(sq);
            let owner_matches = matches!(player, Player::P1) == is_p1;
            if !owner_matches { continue; }
            let kind = if pos.kings.contains(sq) {
                "King"
            } else if pos.champions.contains(sq) {
                "Champ"
            } else {
                "Guard"
            };
            let e = pos.mailbox[sq as usize];
            let line = format!(
                "    {:>3} | {} | {:<5} | {}  |   {}   |   {}   |  {:<4}  |  {:<4}\n",
                sq_name(sq),
                paint(label, code, color),
                kind,
                e.hp(),
                e.armor(),
                e.combo(),
                skill_short(e.skill1()),
                skill_short(e.skill2()),
            );
            out.push_str(&line);
        }
    }
    out
}

fn render_state_bar(pos: &Position, color: bool) -> String {
    let to_move = match pos.to_move {
        Player::P1 => paint("P1", RED,  color),
        Player::P2 => paint("P2", BLUE, color),
    };
    let phase = match pos.current_phase {
        Phase::Move  => "Move",
        Phase::Skill => "Skill",
        Phase::Draft => "Draft",
    };
    let focus  = pos.pending_modifiers & modifier_bits::FOCUS  != 0;
    let charge = pos.pending_modifiers & modifier_bits::CHARGE != 0;
    let mods = match (focus, charge) {
        (false, false) => "none".to_string(),
        (true,  false) => paint("Focus", YELLOW, color),
        (false, true ) => paint("Charge", YELLOW, color),
        (true,  true ) => paint("Focus+Charge", YELLOW, color),
    };
    format!(
        "  to_move: {}  phase: {}  actions_left: {}  round: {}  money: P1={} P2={}  pending: {}  moved_this_phase: 0x{:016x}\n",
        to_move, paint(phase, CYAN, color),
        pos.actions_remaining,
        pos.round_number,
        pos.p1_money, pos.p2_money,
        mods,
        pos.moved_this_phase.0,
    )
}

fn render_full(pos: &Position, color: bool) -> String {
    let mut s = render_board(pos, color);
    s.push_str(&render_state_bar(pos, color));
    s.push_str(&render_piece_table(pos, color));
    s
}

fn wait_for_enter() {
    print!("  {}[press Enter for next ply, Ctrl-C to quit]{} ",
           DIM, RESET);
    let _ = std::io::stdout().flush();
    let stdin = std::io::stdin();
    let mut buf = String::new();
    let _ = stdin.lock().read_line(&mut buf);
}

fn main() {
    let args = parse_args();

    let mut cfg = Config::local_aivai();
    cfg.p1_ai = AiBudget { time_limit_ms: args.p1_ms, max_depth: args.p1_depth };
    cfg.p2_ai = AiBudget { time_limit_ms: args.p2_ms, max_depth: args.p2_depth };

    let want_export = args.export_json.is_some()
                   || args.export_json_pretty.is_some()
                   || args.export_notation.is_some();
    cfg.auto_log = want_export;

    let now_ms = || SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut m = Match::new_with_clock(cfg, now_ms());
    if let (Some(note), Some(log)) = (args.note.as_ref(), m.match_log_mut()) {
        log.add_note(note);
    }
    println!("# AIvAI demo");
    println!("  P1: {}  budget: {} ms / depth {}",
             paint("Ai", RED, args.color), args.p1_ms, args.p1_depth);
    println!("  P2: {}  budget: {} ms / depth {}",
             paint("Ai", BLUE, args.color), args.p2_ms, args.p2_depth);
    println!();
    let _ = SeatKind::Ai; // keep the import live

    if args.show_board {
        println!("--- {} ---", paint("initial position", BOLD, args.color));
        print!("{}", render_full(m.position(), args.color));
        if args.show_fen { println!("  FEN: {}", to_fen(m.position())); }
        println!();
        if args.step { wait_for_enter(); }
    }

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
                     plies + 1, fmt_action(a, args.color), r.score, r.depth, r.nodes);
        }
        if args.show_board {
            print!("{}", render_full(m.position(), args.color));
        }
        if args.show_fen {
            println!("  FEN: {}", to_fen(m.position()));
        }
        if args.show_board || args.show_fen {
            println!();
        }
        if args.step && m.game_result().is_none() && plies + 1 < args.max_plies {
            wait_for_enter();
        } else if args.step_delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(args.step_delay_ms));
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

    if want_export {
        use core_engine::state::position::GameResult;
        let final_result = match m.game_result() {
            Some(GameResult::P1Wins) => MatchResult::P1Win,
            Some(GameResult::P2Wins) => MatchResult::P2Win,
            None                     => MatchResult::Aborted,
        };
        m.finalise_log(now_ms(), final_result);

        let log = m.match_log().expect("auto_log enabled");
        if let Some(p) = &args.export_json {
            let s = to_json(log);
            std::fs::write(p, s).expect("write export-json");
            println!("wrote JSON to {} ({} plies)", p, log.total_plies);
        }
        if let Some(p) = &args.export_json_pretty {
            let s = to_json_pretty(log);
            std::fs::write(p, s).expect("write export-json-pretty");
            println!("wrote pretty JSON to {} ({} plies)", p, log.total_plies);
        }
        if let Some(p) = &args.export_notation {
            let s = notation::to_text(log);
            std::fs::write(p, s).expect("write export-notation");
            println!("wrote notation to {}", p);
        }
    }
}
