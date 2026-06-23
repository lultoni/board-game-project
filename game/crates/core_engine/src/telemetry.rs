//! Layer 5 — Telemetry & Analytics.
//!
//! Per ADR-005: every match is auto-logged with action history + per-move
//! timing + the full per-ply position trace. The "send to designer" upload
//! bundles N recent logs into a JSON blob the user transmits out-of-band.
//! No server endpoint required.
//!
//! This layer is pure data shaping. Persistence is the frontend's job
//! (browser localStorage / Tauri filesystem). The engine never writes files
//! or sleeps; the caller drives the clock.
//!
//! ## Capture scope
//!
//! L5 captures *maximum data* per ply (when `Config::auto_log` is set):
//! the action itself (raw u32 + decoded fields), seat, timing, full
//! pre/post-position fingerprints (FEN, Zobrist, static eval + breakdown),
//! phase/round/money/modifier state, tracked-table snapshots, and AI search
//! metadata when an AI played. Roughly 600 bytes per ply serialised.
//!
//! Schema is **forward-compatible**: optional fields can be added without
//! breaking previously-saved JSON (serde tolerates missing optional fields
//! and ignores unknown ones).

use serde::{Serialize, Deserialize};

use crate::game_logic::action::{Action, ActionKind};
use crate::game_logic::skills::skill_from_id;
use crate::search::evaluator::{evaluate_breakdown, EvalBreakdown, MATE_SCORE};
use crate::session::{Config, SeatKind};
use crate::state::Position;
use crate::state::position::{GameResult, Phase, Player, modifier_bits};

// === Search metadata =========================================================

/// AI search readout for a single ply. `raw_score` is what alpha-beta
/// returned; `was_mate / mate_in / score_cp` are the interpreted view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchMeta {
    pub depth:      u8,
    pub nodes:      u64,
    pub raw_score:  i32,
    pub was_mate:   bool,
    /// Plies-to-mate when `was_mate`; None otherwise.
    pub mate_in:    Option<i32>,
    /// "Centipawn"-style score when not a mate; None when `was_mate`.
    pub score_cp:   Option<i32>,
}

impl SearchMeta {
    /// Build from an alpha-beta SearchResult. Threshold matches L3's
    /// `MATE_THRESHOLD = MATE_SCORE - MAX_PLY` (128).
    pub fn from_search(depth: u8, nodes: u64, score: i32) -> Self {
        const MAX_PLY: i32 = 128;
        let threshold = MATE_SCORE - MAX_PLY;
        let was_mate = score.abs() > threshold;
        let (mate_in, score_cp) = if was_mate {
            (Some(MATE_SCORE - score.abs()), None)
        } else {
            (None, Some(score))
        };
        SearchMeta { depth, nodes, raw_score: score, was_mate, mate_in, score_cp }
    }
}

// === Action decode (denormalised for downstream tools) ======================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionDecoded {
    pub raw:        u32,
    pub kind:       String,         // "Move" | "Skill" | "EndPhase" | "EndTurn"
    pub src:        u8,
    pub target:     u8,
    pub skill_id:   u8,
    pub skill_name: Option<String>, // only when kind == "Skill" and id is known
}

impl ActionDecoded {
    pub fn from_action(a: Action) -> Self {
        let kind_str = match a.kind() {
            ActionKind::Move     => "Move",
            ActionKind::Skill    => "Skill",
            ActionKind::EndPhase => "EndPhase",
            ActionKind::EndTurn  => "EndTurn",
        };
        let skill_name = if matches!(a.kind(), ActionKind::Skill) {
            skill_from_id(a.skill_id()).map(|s| format!("{:?}", s))
        } else {
            None
        };
        ActionDecoded {
            raw:        a.0,
            kind:       kind_str.to_string(),
            src:        a.src(),
            target:     a.target(),
            skill_id:   a.skill_id(),
            skill_name,
        }
    }
}

// === Per-ply record =========================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlyRecord {
    pub ply_no:                u32,
    pub seat_player:           Player,
    pub seat_kind:             SeatKind,
    pub thought_ms:            u32,
    pub applied_at_unix_ms:    u64,
    pub action:                ActionDecoded,
    pub legal_count:           u32,
    // pre-action position fingerprint
    pub prev_zobrist:          u64,
    pub prev_fen:              String,
    pub prev_static_eval:      i32,
    pub prev_breakdown:        EvalBreakdown,
    // post-action position fingerprint
    pub post_zobrist:          u64,
    pub post_fen:              String,
    pub post_static_eval:      i32,
    pub post_breakdown:        EvalBreakdown,
    pub post_game_result:      Option<GameResult>,
    pub post_phase:            Phase,
    pub post_actions_remaining:u8,
    pub post_round:            u16,
    pub post_focus_pending:    bool,
    pub post_charge_pending:   bool,
    pub post_moved_this_phase: u64,
    pub post_p1_money:         u16,
    pub post_p2_money:         u16,
    pub post_tracked_enemies:  Vec<u8>,
    pub post_tracked_casters:  Vec<u8>,
    // AI metadata; None when human played
    pub ai:                    Option<SearchMeta>,
}

/// Snapshot a `Position` into the "post" half of a PlyRecord. Public so
/// `session::Match::try_apply_timed` can build records without leaking
/// internal layout.
pub fn snapshot_pre(pos: &Position) -> (u64, String, i32, EvalBreakdown) {
    let bd = evaluate_breakdown(pos);
    (pos.zobrist, pos.to_fen(), bd.total, bd)
}

#[allow(clippy::type_complexity)]
pub fn snapshot_post(pos: &Position)
    -> (u64, String, i32, EvalBreakdown, Option<GameResult>, Phase, u8, u16, bool, bool, u64, u16, u16, Vec<u8>, Vec<u8>)
{
    let bd = evaluate_breakdown(pos);
    let focus  = pos.pending_modifiers & modifier_bits::FOCUS  != 0;
    let charge = pos.pending_modifiers & modifier_bits::CHARGE != 0;
    let te = pos.tracked_enemies[..pos.tracked_enemies_len as usize].to_vec();
    let tc = pos.tracked_casters[..pos.tracked_casters_len as usize].to_vec();
    (
        pos.zobrist, pos.to_fen(), bd.total, bd,
        pos.game_result,
        pos.current_phase, pos.actions_remaining, pos.round_number,
        focus, charge,
        pos.moved_this_phase.0,
        pos.p1_money, pos.p2_money,
        te, tc,
    )
}

// === Match result ===========================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchResult { P1Win, P2Win, Draw, Aborted }

impl MatchResult {
    pub fn from_game_result(r: GameResult) -> Self {
        match r { GameResult::P1Wins => MatchResult::P1Win, GameResult::P2Wins => MatchResult::P2Win }
    }
}

// === Match log ==============================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchLog {
    pub engine_version:  String,
    pub started_at_unix: u64,
    pub config:          Config,
    pub config_hash:     u64,
    pub start_fen:       String,
    pub start_zobrist:   u64,
    pub plies:           Vec<PlyRecord>,
    pub final_result:    Option<MatchResult>,
    pub final_fen:       Option<String>,
    pub final_zobrist:   Option<u64>,
    pub total_plies:     u32,
    pub total_wall_ms:   u64,
    pub total_ai_nodes:  u64,
    pub notes:           Option<String>,
}

impl MatchLog {
    pub fn new(now_unix: u64, config: Config, start: &Position) -> Self {
        let hash = config_hash(&config);
        MatchLog {
            engine_version:  env!("CARGO_PKG_VERSION").to_string(),
            started_at_unix: now_unix,
            config,
            config_hash:     hash,
            start_fen:       start.to_fen(),
            start_zobrist:   start.zobrist,
            plies:           Vec::new(),
            final_result:    None,
            final_fen:       None,
            final_zobrist:   None,
            total_plies:     0,
            total_wall_ms:   0,
            total_ai_nodes:  0,
            notes:           None,
        }
    }

    pub fn record(&mut self, ev: PlyRecord) {
        self.total_wall_ms = self.total_wall_ms.saturating_add(ev.thought_ms as u64);
        if let Some(ai) = ev.ai { self.total_ai_nodes = self.total_ai_nodes.saturating_add(ai.nodes); }
        self.total_plies = self.total_plies.saturating_add(1);
        self.plies.push(ev);
    }

    pub fn finish(&mut self, _now_unix: u64, result: MatchResult, final_pos: &Position) {
        self.final_result  = Some(result);
        self.final_fen     = Some(final_pos.to_fen());
        self.final_zobrist = Some(final_pos.zobrist);
    }

    pub fn add_note<S: Into<String>>(&mut self, s: S) {
        let s = s.into();
        match self.notes.as_mut() {
            Some(existing) => { existing.push('\n'); existing.push_str(&s); }
            None           => { self.notes = Some(s); }
        }
    }
}

// === Bundle for multi-match export =========================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub exported_at_unix: u64,
    pub engine_version:   String,
    pub logs:             Vec<MatchLog>,
}

impl Bundle {
    pub fn new(now_unix: u64, logs: Vec<MatchLog>) -> Self {
        Bundle {
            exported_at_unix: now_unix,
            engine_version:   env!("CARGO_PKG_VERSION").to_string(),
            logs,
        }
    }
}

// === JSON helpers ===========================================================

pub fn to_json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).expect("serialise failed (should not happen for owned values)")
}

pub fn to_json_pretty<T: Serialize>(v: &T) -> String {
    serde_json::to_string_pretty(v).expect("serialise failed")
}

pub fn from_json<T: for<'a> Deserialize<'a>>(s: &str) -> serde_json::Result<T> {
    serde_json::from_str(s)
}

// === Config hash ============================================================

/// Deterministic 64-bit fingerprint of a Config — stable across processes
/// (DefaultHasher is randomly-seeded; we hand-roll an FxHash-style mixer so
/// the designer can group matches by config without leaking the full struct).
pub fn config_hash(c: &Config) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325; // FNV offset basis
    let mix = |h: u64, x: u64| -> u64 { (h ^ x).wrapping_mul(0x517C_C1B7_2722_0A95) };
    let seat = |s: SeatKind| -> u64 { match s { SeatKind::Human => 1, SeatKind::Ai => 2 } };
    h = mix(h, seat(c.p1));
    h = mix(h, seat(c.p2));
    h = mix(h, c.p1_ai.time_limit_ms);
    h = mix(h, c.p1_ai.max_depth as u64);
    h = mix(h, c.p2_ai.time_limit_ms);
    h = mix(h, c.p2_ai.max_depth as u64);
    h = mix(h, c.aivai_step_delay.as_millis() as u64);
    h = mix(h, c.allow_undo as u64);
    h = mix(h, c.auto_log as u64);
    h
}

// === Notation (PGN-like) ====================================================

pub mod notation {
    use super::*;

    /// Square index → "a1".."h8".
    fn sq_name(sq: u8) -> String {
        if sq >= 64 { return format!("?{}", sq); }
        let file = (b'a' + (sq % 8)) as char;
        let rank = (sq / 8) + 1;
        format!("{}{}", file, rank)
    }

    fn fmt_action(d: &ActionDecoded) -> String {
        match d.kind.as_str() {
            "Move"     => format!("Move {}→{}", sq_name(d.src), sq_name(d.target)),
            "Skill"    => {
                let name = d.skill_name.clone().unwrap_or_else(|| format!("Skill#{}", d.skill_id));
                format!("{} {}→{}", name, sq_name(d.src), sq_name(d.target))
            }
            "EndPhase" => "EndPhase".to_string(),
            "EndTurn"  => "EndTurn".to_string(),
            other      => other.to_string(),
        }
    }

    fn fmt_swing(prev: i32, post: i32) -> String {
        let d = post - prev;
        if d.abs() < 200 { return "      ".to_string(); }
        if d > 0 { format!("▲+{:<4}", d) } else { format!("▼{:<5}", d) }
    }

    fn fmt_ai(ai: &SearchMeta) -> String {
        let core = format!("d={} nodes={}", ai.depth, fmt_nodes(ai.nodes));
        if ai.was_mate {
            format!("[{} mate_in={}]", core, ai.mate_in.unwrap_or(0))
        } else {
            format!("[{} score={}]", core, ai.score_cp.unwrap_or(ai.raw_score))
        }
    }

    fn fmt_nodes(n: u64) -> String {
        if n < 1_000 { format!("{}", n) }
        else if n < 1_000_000 { format!("{:.1}k", n as f64 / 1_000.0) }
        else { format!("{:.1}M", n as f64 / 1_000_000.0) }
    }

    pub fn to_text(log: &MatchLog) -> String {
        let mut s = String::with_capacity(4096);
        s.push_str(&format!(
            "# Match started_unix={} engine={} config_hash=0x{:016X}\n",
            log.started_at_unix, log.engine_version, log.config_hash,
        ));
        s.push_str(&format!("# start_fen: \"{}\"\n", log.start_fen));
        s.push_str(&format!("# start_zobrist: 0x{:016X}\n", log.start_zobrist));
        for (i, p) in log.plies.iter().enumerate() {
            let seat = match p.seat_player { Player::P1 => "P1", Player::P2 => "P2" };
            let kind = match p.seat_kind   { SeatKind::Human => "Human", SeatKind::Ai => "Ai" };
            let act  = fmt_action(&p.action);
            let swing = fmt_swing(p.prev_static_eval, p.post_static_eval);
            let ai_tag = p.ai.as_ref().map(fmt_ai).unwrap_or_default();
            s.push_str(&format!(
                "{:>4}. {} ({}) {:<28} [{:>5}ms] {}  {}\n",
                i + 1, seat, kind, act, p.thought_ms, swing, ai_tag,
            ));
        }
        match log.final_result {
            Some(r) => s.push_str(&format!(
                "# result: {:?}  plies={}  wall_ms={}  ai_nodes={}\n",
                r, log.total_plies, log.total_wall_ms, log.total_ai_nodes,
            )),
            None => s.push_str(&format!(
                "# result: unfinished  plies={}  wall_ms={}\n",
                log.total_plies, log.total_wall_ms,
            )),
        }
        s
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{AiBudget, Match};
    use crate::game_logic::generator;
    use crate::state::Position;

    fn cfg_aivai_short() -> Config {
        let mut c = Config::local_aivai();
        c.p1_ai = AiBudget { time_limit_ms: 0, max_depth: 3 };
        c.p2_ai = AiBudget { time_limit_ms: 0, max_depth: 3 };
        c.auto_log = true;
        c
    }

    #[test]
    fn match_log_off_by_default() {
        let m = Match::new(Config::local_hvh());
        assert!(m.match_log().is_none());
    }

    #[test]
    fn match_log_on_when_flag_set() {
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let m = Match::new(cfg);
        assert!(m.match_log().is_some());
        assert_eq!(m.match_log().unwrap().plies.len(), 0);
    }

    #[test]
    fn try_apply_records_ply_when_logging() {
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        let first = m.legal_actions()[0];
        m.try_apply(first).unwrap();
        let log = m.match_log().unwrap();
        assert_eq!(log.plies.len(), 1);
        assert_eq!(log.plies[0].action.raw, first.0);
        assert_eq!(log.plies[0].post_zobrist, m.position().zobrist);
        assert!(log.plies[0].ai.is_none()); // human-applied
    }

    #[test]
    fn ply_pre_post_fingerprints_are_consistent() {
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        for _ in 0..6 {
            let acts = m.legal_actions();
            if acts.is_empty() { break; }
            m.try_apply(acts[0]).unwrap();
        }
        let log = m.match_log().unwrap();
        for p in &log.plies {
            let prev = Position::from_fen(&p.prev_fen).expect("prev_fen parses");
            assert_eq!(prev.zobrist, p.prev_zobrist, "prev_fen → zobrist mismatch");
            let post = Position::from_fen(&p.post_fen).expect("post_fen parses");
            assert_eq!(post.zobrist, p.post_zobrist, "post_fen → zobrist mismatch");
        }
    }

    #[test]
    fn eval_breakdown_total_equals_evaluate() {
        use crate::search::evaluator::evaluate;
        let mut m = Match::new(Config::local_hvh());
        for _ in 0..10 {
            let acts = m.legal_actions();
            if acts.is_empty() { break; }
            m.try_apply(acts[0]).unwrap();
            let bd = evaluate_breakdown(m.position());
            assert_eq!(bd.total, evaluate(m.position()));
        }
    }

    #[test]
    fn legal_count_matches_generator() {
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        for _ in 0..4 {
            let acts = m.legal_actions();
            if acts.is_empty() { break; }
            m.try_apply(acts[0]).unwrap();
        }
        let log = m.match_log().unwrap();
        for p in &log.plies {
            let prev = Position::from_fen(&p.prev_fen).unwrap();
            let n = generator::generate(&prev).len() as u32;
            assert_eq!(p.legal_count, n);
        }
    }

    #[test]
    fn step_ai_records_searchmeta() {
        let mut m = Match::new(cfg_aivai_short());
        for _ in 0..5 {
            if m.game_result().is_some() { break; }
            m.step_ai().unwrap();
        }
        let log = m.match_log().unwrap();
        assert!(!log.plies.is_empty());
        for p in &log.plies {
            let ai = p.ai.as_ref().expect("AI ply must carry SearchMeta");
            assert!(ai.depth > 0);
            assert!(ai.nodes > 0);
        }
    }

    #[test]
    fn searchmeta_flags_mate_at_terminal() {
        let mut m = Match::new(cfg_aivai_short());
        let mut steps = 0usize;
        while m.game_result().is_none() && steps < 400 {
            m.step_ai().unwrap();
            steps += 1;
        }
        assert!(m.game_result().is_some(), "AIvAI should terminate within 400 plies");
        let log = m.match_log().unwrap();
        let last = log.plies.last().unwrap();
        let ai = last.ai.as_ref().unwrap();
        assert!(ai.was_mate, "terminating ply should be mate");
        assert!(ai.mate_in.is_some());
    }

    #[test]
    fn finalise_sets_fields() {
        let mut m = Match::new(cfg_aivai_short());
        while m.game_result().is_none() {
            m.step_ai().unwrap();
        }
        let final_result = MatchResult::from_game_result(m.game_result().unwrap());
        m.finalise_log(0, final_result);
        let log = m.match_log().unwrap();
        assert_eq!(log.final_result, Some(final_result));
        assert!(log.final_fen.is_some());
        assert!(log.final_zobrist.is_some());
        assert!(log.total_plies > 0);
    }

    #[test]
    fn notation_has_header_and_per_ply_lines() {
        let mut m = Match::new(cfg_aivai_short());
        for _ in 0..4 {
            if m.game_result().is_some() { break; }
            m.step_ai().unwrap();
        }
        let text = notation::to_text(m.match_log().unwrap());
        let lines: Vec<&str> = text.lines().collect();
        // 3 header lines + N move lines + 1 result line
        assert!(lines.len() >= m.match_log().unwrap().plies.len() + 4);
        // first ply line contains "P1" or "P2" and a "ms" tag
        let first_move_line = lines.iter().find(|l| !l.starts_with('#')).unwrap();
        assert!(first_move_line.contains("ms"));
    }

    #[test]
    fn notation_marks_eval_swing() {
        // Synthetic record with a big swing
        let mut log = MatchLog::new(0, Config::local_hvh(), &Position::setup_stack_m());
        log.plies.push(PlyRecord {
            ply_no: 1,
            seat_player: Player::P1,
            seat_kind: SeatKind::Human,
            thought_ms: 0,
            applied_at_unix_ms: 0,
            action: ActionDecoded::from_action(Action::default()),
            legal_count: 0,
            prev_zobrist: 0, prev_fen: "x".into(), prev_static_eval: 0, prev_breakdown: EvalBreakdown::default(),
            post_zobrist: 0, post_fen: "x".into(), post_static_eval: 500, post_breakdown: EvalBreakdown::default(),
            post_game_result: None, post_phase: Phase::Move,
            post_actions_remaining: 2, post_round: 1,
            post_focus_pending: false, post_charge_pending: false,
            post_moved_this_phase: 0, post_p1_money: 0, post_p2_money: 0,
            post_tracked_enemies: vec![], post_tracked_casters: vec![],
            ai: None,
        });
        log.total_plies = 1;
        let text = notation::to_text(&log);
        assert!(text.contains("▲"), "expected up-arrow for +500 swing, got:\n{}", text);
    }

    #[test]
    fn json_action_is_transparent() {
        let s = to_json(&Action(0xDEAD_BEEF));
        // serde_json renders u32 as a bare integer with no struct wrapping
        assert_eq!(s, "3735928559");
        let back: Action = from_json(&s).unwrap();
        assert_eq!(back.0, 0xDEAD_BEEF);
    }

    #[test]
    fn json_match_log_round_trip() {
        let mut m = Match::new(cfg_aivai_short());
        for _ in 0..6 { if m.game_result().is_some() { break; } m.step_ai().unwrap(); }
        let log = m.match_log().unwrap().clone();
        let s = to_json(&log);
        let back: MatchLog = from_json(&s).unwrap();
        assert_eq!(back, log);
    }

    #[test]
    fn json_bundle_round_trip() {
        let logs: Vec<MatchLog> = (0..3).map(|i| {
            let mut m = Match::new(cfg_aivai_short());
            for _ in 0..(2 + i) { if m.game_result().is_some() { break; } m.step_ai().unwrap(); }
            m.match_log().unwrap().clone()
        }).collect();
        let b = Bundle::new(42, logs);
        let s = to_json(&b);
        let back: Bundle = from_json(&s).unwrap();
        assert_eq!(back, b);
    }

    #[test]
    fn json_rejects_malformed() {
        let r: serde_json::Result<MatchLog> = from_json("{not json");
        assert!(r.is_err());
    }

    #[test]
    fn config_hash_stable_and_distinct() {
        let c1 = Config::local_hvh();
        let c2 = Config::local_hvh();
        assert_eq!(config_hash(&c1), config_hash(&c2));
        let mut c3 = Config::local_hvh();
        c3.auto_log = true;
        assert_ne!(config_hash(&c1), config_hash(&c3));
    }

    #[test]
    fn engine_version_matches_cargo() {
        let m = Match::new({ let mut c = Config::local_hvh(); c.auto_log = true; c });
        assert_eq!(m.match_log().unwrap().engine_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn pre_eval_matches_static_eval_of_prev_fen() {
        use crate::search::evaluator::evaluate;
        let mut cfg = Config::local_hvh();
        cfg.auto_log = true;
        let mut m = Match::new(cfg);
        for _ in 0..4 {
            let acts = m.legal_actions();
            if acts.is_empty() { break; }
            m.try_apply(acts[0]).unwrap();
        }
        let log = m.match_log().unwrap();
        for p in &log.plies {
            let prev = Position::from_fen(&p.prev_fen).unwrap();
            assert_eq!(p.prev_static_eval, evaluate(&prev));
        }
    }
}
