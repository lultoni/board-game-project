//! Compact human-readable notation for [`Action`] values.
//!
//! Two public functions:
//! - [`action_to_notation`] — encode an `Action` to a notation string.
//! - [`notation_to_action`] — parse a notation string back to an `Action`.
//!
//! # Bodyguard redirect encoding
//!
//! `action_to_notation` takes `Option<&PendingBodyguard>`. Pass
//! `self.position.pending_bodyguard.as_ref()` at the one live call site in
//! `session.rs::try_apply_timed` (before `make()` clears the pending state).
//! All other call sites pass `None` and receive a `bg<N>` (numeric index)
//! fallback instead of the canonical `bg<sq>` form. This only affects
//! BodyguardChoice redirect actions; decline (`bgX`) and all other action
//! families are unaffected.

use crate::game_logic::action::{Action, ActionKind};
use crate::state::position::PendingBodyguard;

// ---- NotationError --------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotationError {
    EmptyInput,
    BadSquare(String),
    UnknownSkill(String),
    UnknownDirection(String),
    NoPendingBodyguard,
    BadBodyguardSquare(String),
    TrailingInput(String),
    UnexpectedChar { pos: usize, ch: char },
}

impl std::fmt::Display for NotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput              => write!(f, "empty input"),
            Self::BadSquare(s)            => write!(f, "bad square: {s:?}"),
            Self::UnknownSkill(s)         => write!(f, "unknown skill: {s:?}"),
            Self::UnknownDirection(s)     => write!(f, "unknown direction: {s:?}"),
            Self::NoPendingBodyguard      => write!(f, "no pending bodyguard state"),
            Self::BadBodyguardSquare(s)   => write!(f, "square not in eligible list: {s:?}"),
            Self::TrailingInput(s)        => write!(f, "trailing input: {s:?}"),
            Self::UnexpectedChar { pos, ch } => write!(f, "unexpected char {ch:?} at pos {pos}"),
        }
    }
}

impl std::error::Error for NotationError {}

// ---- Skill ID <-> name table ----------------------------------------------

fn skill_name(id: u8) -> Option<&'static str> {
    Some(match id {
        1  => "Lance",   2  => "Hook",    3  => "Break",
        4  => "Steal",   5  => "Tempest", 6  => "Shield",
        7  => "Heal",    8  => "Plate",   9  => "Dash",
        10 => "Blast",   11 => "Shove",   12 => "Swap",
        13 => "Retreat", 14 => "Focus",   15 => "Charge",
        _  => return None,
    })
}

fn skill_id_from_name(s: &str) -> Option<u8> {
    Some(match s {
        "Lance"   => 1,  "Hook"    => 2,  "Break"   => 3,
        "Steal"   => 4,  "Tempest" => 5,  "Shield"  => 6,
        "Heal"    => 7,  "Plate"   => 8,  "Dash"    => 9,
        "Blast"   => 10, "Shove"   => 11, "Swap"    => 12,
        "Retreat" => 13, "Focus"   => 14, "Charge"  => 15,
        _         => return None,
    })
}

const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

// ---- Square helpers -------------------------------------------------------

pub fn sq_to_notation(sq: u8) -> String {
    debug_assert!(sq < 64);
    let file = (b'a' + sq % 8) as char;
    let rank = sq / 8 + 1;
    format!("{}{}", file, rank)
}

pub fn notation_to_sq(s: &str) -> Result<u8, NotationError> {
    let b = s.as_bytes();
    if b.len() != 2
        || !(b'a'..=b'h').contains(&b[0])
        || !(b'1'..=b'8').contains(&b[1])
    {
        return Err(NotationError::BadSquare(s.to_string()));
    }
    Ok((b[0] - b'a') + (b[1] - b'1') * 8)
}

// ---- action_to_notation ---------------------------------------------------

pub fn action_to_notation(action: Action, pending: Option<&PendingBodyguard>) -> String {
    // 1. BodyguardChoice (bit 31 set)
    if action.is_bodyguard_choice() {
        let idx = action.bg_guard_idx();
        if idx == 0 {
            return "bgX".to_string();
        }
        if let Some(pb) = pending {
            let i = (idx - 1) as usize;
            if i < pb.eligible_len as usize {
                return format!("bg{}", sq_to_notation(pb.eligible[i]));
            }
        }
        // Fall back to numeric index when pending context is unavailable.
        return format!("bg{}", idx);
    }

    // 2. DraftTurn (bit 30 set)
    if action.is_draft_turn() {
        let (s1, sq1, slot1) = action.draft_pick1();
        let (s2, sq2, slot2) = action.draft_pick2();
        return format!(
            "draft {}@{}:{}+{}@{}:{}",
            skill_name(s1).unwrap_or("?"), sq_to_notation(sq1), slot1 + 1,
            skill_name(s2).unwrap_or("?"), sq_to_notation(sq2), slot2 + 1,
        );
    }

    // 3. Regular action
    match action.kind() {
        ActionKind::EndPhase => "endphase".to_string(),
        ActionKind::EndTurn  => "endturn".to_string(),

        ActionKind::Move => {
            let src = action.src();
            let tgt = action.target();
            if !action.has_approach() {
                format!("{}-{}", sq_to_notation(src), sq_to_notation(tgt))
            } else {
                let mut s = format!("{}x{}", sq_to_notation(src), sq_to_notation(tgt));
                let approach = action.approach_sq();
                if approach != src {
                    s.push('@');
                    s.push_str(&sq_to_notation(approach));
                }
                s
            }
        }

        ActionKind::Skill => {
            let src    = action.src();
            let tgt    = action.target();
            let sid    = action.skill_id();
            let mut s  = format!(
                "{}*{}:{}",
                sq_to_notation(src),
                sq_to_notation(tgt),
                skill_name(sid).unwrap_or("?"),
            );
            // Suffix order: ~ then > then :DIR
            if action.focus_effect_mode() {
                s.push('~');
            }
            if action.has_aux() {
                s.push('>');
                let aux = action.aux_sq();
                if aux != tgt {
                    s.push_str(&sq_to_notation(aux));
                }
            }
            if sid == 11 {
                // Shove: append cardinal direction
                let dir_idx = action.choice_idx() as usize;
                let dir = DIRS.get(dir_idx).copied().unwrap_or("?");
                s.push(':');
                s.push_str(dir);
            }
            s
        }
    }
}

// ---- notation_to_action ---------------------------------------------------

pub fn notation_to_action(s: &str, pos: &crate::state::position::Position) -> Result<Action, NotationError> {
    let s = s.trim();

    if s.is_empty() {
        return Err(NotationError::EmptyInput);
    }

    // endphase / endturn (exact match — no trailing input allowed)
    if s.starts_with("endphase") {
        if s.len() > "endphase".len() {
            return Err(NotationError::TrailingInput(s["endphase".len()..].to_string()));
        }
        return Ok(Action::encode(0, 0, ActionKind::EndPhase, 0, 0));
    }
    if s.starts_with("endturn") {
        if s.len() > "endturn".len() {
            return Err(NotationError::TrailingInput(s["endturn".len()..].to_string()));
        }
        return Ok(Action::encode(0, 0, ActionKind::EndTurn, 0, 0));
    }

    // Bodyguard actions
    if s.starts_with("bg") {
        let rest = &s[2..];
        if rest == "X" {
            return Ok(Action::encode_bodyguard_choice(0));
        }
        // bg<letter><digit> → square form
        let bytes = rest.as_bytes();
        if bytes.len() >= 2 && (b'a'..=b'h').contains(&bytes[0]) {
            let sq = notation_to_sq(rest)?;
            let pb = pos.pending_bodyguard.as_ref().ok_or(NotationError::NoPendingBodyguard)?;
            for k in 0..pb.eligible_len as usize {
                if pb.eligible[k] == sq {
                    return Ok(Action::encode_bodyguard_choice(k as u8 + 1));
                }
            }
            return Err(NotationError::BadBodyguardSquare(rest.to_string()));
        }
        // bg<digit(s)> → raw index form
        let idx: u8 = rest.parse().map_err(|_| NotationError::UnexpectedChar { pos: 2, ch: rest.chars().next().unwrap_or('?') })?;
        if idx > Action::BG_CHOICE_MAX_IDX {
            return Err(NotationError::UnexpectedChar { pos: 2, ch: '?' });
        }
        return Ok(Action::encode_bodyguard_choice(idx));
    }

    // Draft turn
    if let Some(rest) = s.strip_prefix("draft ") {
        // Split on '+' into two pick strings
        let plus_pos = rest.find('+').ok_or_else(|| NotationError::UnexpectedChar { pos: 6, ch: '?' })?;
        let pick1_str = &rest[..plus_pos];
        let pick2_str = &rest[plus_pos + 1..];

        let (sk1, sq1, sl1) = parse_draft_pick(pick1_str)?;
        let (sk2, sq2, sl2) = parse_draft_pick(pick2_str)?;
        return Ok(Action::encode_draft_turn(sk1, sq1, sl1, sk2, sq2, sl2));
    }

    // Skill (contains '*')
    if let Some(star_pos) = s.find('*') {
        let src_str  = &s[..star_pos];
        let after    = &s[star_pos + 1..];
        let src = notation_to_sq(src_str)?;

        // Split on first ':' to separate tgt from suffix
        let colon_pos = after.find(':').ok_or_else(|| NotationError::UnexpectedChar { pos: star_pos + 1, ch: '?' })?;
        let tgt_str   = &after[..colon_pos];
        let suffix    = &after[colon_pos + 1..];
        let tgt = notation_to_sq(tgt_str)?;

        return parse_skill_suffix(src, tgt, suffix);
    }

    // Move-Attack (contains 'x')
    if let Some(x_pos) = s.find('x') {
        let src_str   = &s[..x_pos];
        let right     = &s[x_pos + 1..];
        let src = notation_to_sq(src_str)?;

        let (tgt, approach) = if let Some(at_pos) = right.find('@') {
            let tgt     = notation_to_sq(&right[..at_pos])?;
            let approach = notation_to_sq(&right[at_pos + 1..])?;
            (tgt, approach)
        } else {
            let tgt = notation_to_sq(right)?;
            (tgt, src)
        };
        return Ok(Action::encode_move_attack(src, tgt, 0, approach));
    }

    // Plain move (contains '-')
    if let Some(dash_pos) = s.find('-') {
        let src_str = &s[..dash_pos];
        let tgt_str = &s[dash_pos + 1..];
        let src = notation_to_sq(src_str)?;
        let tgt = notation_to_sq(tgt_str)?;
        return Ok(Action::encode(src, tgt, ActionKind::Move, 0, 0));
    }

    // Nothing matched
    let ch = s.chars().next().unwrap_or('?');
    Err(NotationError::UnexpectedChar { pos: 0, ch })
}

// Parse "<SkillName>@<sq>:<slot>" for draft turns.
fn parse_draft_pick(s: &str) -> Result<(u8, u8, u8), NotationError> {
    let at_pos = s.find('@').ok_or_else(|| NotationError::UnexpectedChar { pos: 0, ch: '?' })?;
    let skill_str = &s[..at_pos];
    let rest      = &s[at_pos + 1..];

    let skill_id = skill_id_from_name(skill_str)
        .ok_or_else(|| NotationError::UnknownSkill(skill_str.to_string()))?;

    let colon_pos = rest.find(':').ok_or_else(|| NotationError::UnexpectedChar { pos: 0, ch: '?' })?;
    let sq_str   = &rest[..colon_pos];
    let slot_ch  = &rest[colon_pos + 1..];

    let sq = notation_to_sq(sq_str)?;
    let slot = match slot_ch {
        "1" => 0u8,
        "2" => 1u8,
        _   => {
            let ch = slot_ch.chars().next().unwrap_or('?');
            return Err(NotationError::UnexpectedChar { pos: colon_pos + 1, ch });
        }
    };

    Ok((skill_id, sq, slot))
}

// Parse the suffix of a skill action (everything after "src*tgt:") and build the Action.
fn parse_skill_suffix(src: u8, tgt: u8, suffix: &str) -> Result<Action, NotationError> {
    let mut rest = suffix;

    // 1. Read skill name up to '~', '>', ':', or end.
    let name_end = rest.find(|c| c == '~' || c == '>' || c == ':').unwrap_or(rest.len());
    let skill_str = &rest[..name_end];
    let skill_id  = skill_id_from_name(skill_str)
        .ok_or_else(|| NotationError::UnknownSkill(skill_str.to_string()))?;
    rest = &rest[name_end..];

    // 2. Optional '~' (focus_effect)
    let mut focus_effect = false;
    if rest.starts_with('~') {
        focus_effect = true;
        rest = &rest[1..];
    }

    // 3. Optional '>' (has_aux)
    let mut has_aux = false;
    let mut aux_sq  = tgt;
    if rest.starts_with('>') {
        has_aux = true;
        rest    = &rest[1..];
        // Peek next 2 chars: if [a-h][1-8] consume as aux_sq
        let bytes = rest.as_bytes();
        if bytes.len() >= 2
            && (b'a'..=b'h').contains(&bytes[0])
            && (b'1'..=b'8').contains(&bytes[1])
        {
            aux_sq = notation_to_sq(&rest[..2])?;
            rest   = &rest[2..];
        }
        // else aux_sq stays == tgt (bare '>')
    }

    // 4. Optional ':DIR' (Shove direction)
    let mut choice_idx: u8 = 0;
    if rest.starts_with(':') {
        let dir_str = &rest[1..];
        let idx = DIRS.iter().position(|&d| d == dir_str)
            .ok_or_else(|| NotationError::UnknownDirection(dir_str.to_string()))?;
        choice_idx = idx as u8;
        rest = &rest[1 + dir_str.len()..];
    }

    // 5. Nothing should remain
    if !rest.is_empty() {
        return Err(NotationError::TrailingInput(rest.to_string()));
    }

    // Construct action
    let action = match (focus_effect, has_aux) {
        (false, false) => Action::encode(src, tgt, ActionKind::Skill, skill_id, choice_idx),
        (false, true)  => Action::encode_with_aux(src, tgt, ActionKind::Skill, skill_id, choice_idx, aux_sq),
        (true,  false) => Action::encode_focus_effect(src, tgt, ActionKind::Skill, skill_id, choice_idx),
        (true,  true)  => {
            let mut a = Action::encode_with_aux(src, tgt, ActionKind::Skill, skill_id, choice_idx, aux_sq);
            a.0 |= 1 << 22;
            a
        }
    };
    Ok(action)
}

// ---- Tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::position::{PendingBodyguard, Position};

    fn minimal_pos() -> Position {
        Position::empty()
    }

    fn pos_with_bg(eligible: &[u8]) -> Position {
        let mut pos = Position::empty();
        let mut arr = [0u8; crate::state::position::MAX_BODYGUARD_ELIGIBLE];
        for (i, &sq) in eligible.iter().enumerate().take(crate::state::position::MAX_BODYGUARD_ELIGIBLE) {
            arr[i] = sq;
        }
        pos.pending_bodyguard = Some(PendingBodyguard {
            attacker_src: 0,
            attacker_now: 0,
            target_sq: 0,
            eligible: arr,
            eligible_len: eligible.len() as u8,
        });
        pos
    }

    // A5-style: encode → string, then check the string
    fn enc(action: Action) -> String {
        action_to_notation(action, None)
    }

    // A6-style: string → action
    fn dec(s: &str) -> Action {
        notation_to_action(s, &minimal_pos()).expect(s)
    }

    fn dec_with_bg(s: &str, eligible: &[u8]) -> Action {
        notation_to_action(s, &pos_with_bg(eligible)).expect(s)
    }

    // --- 1: sq_roundtrip_all_64 ---
    #[test]
    fn sq_roundtrip_all_64() {
        for sq in 0u8..64 {
            let s = sq_to_notation(sq);
            let back = notation_to_sq(&s).unwrap();
            assert_eq!(back, sq, "roundtrip failed for sq={sq}");
        }
    }

    // --- 2: sq_corners ---
    #[test]
    fn sq_corners() {
        assert_eq!(sq_to_notation(0),  "a1");
        assert_eq!(sq_to_notation(63), "h8");
    }

    // --- 3: sq_bad_input ---
    #[test]
    fn sq_bad_input() {
        for bad in &["z9", "a9", "i1", ""] {
            assert!(notation_to_sq(bad).is_err(), "expected error for {bad:?}");
        }
    }

    // --- 4: plain_move_roundtrip ---
    #[test]
    fn plain_move_roundtrip() {
        let a = Action::encode(0, 9, ActionKind::Move, 0, 0); // a1-b2
        let s = enc(a);
        assert_eq!(s, "a1-b2");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 5: move_attack_speed1 ---
    #[test]
    fn move_attack_speed1() {
        // c3 = sq 18, d5 = sq 35. approach = src (speed-1), no '@'
        let src = notation_to_sq("c3").unwrap();
        let tgt = notation_to_sq("d5").unwrap();
        let a = Action::encode_move_attack(src, tgt, 0, src);
        let s = enc(a);
        assert_eq!(s, "c3xd5");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 6: move_attack_speed2 ---
    #[test]
    fn move_attack_speed2() {
        let src      = notation_to_sq("c3").unwrap();
        let tgt      = notation_to_sq("d5").unwrap();
        let approach = notation_to_sq("c4").unwrap();
        let a = Action::encode_move_attack(src, tgt, 0, approach);
        let s = enc(a);
        assert_eq!(s, "c3xd5@c4");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 7: skill_basic ---
    #[test]
    fn skill_basic() {
        let src = notation_to_sq("b2").unwrap();
        let tgt = notation_to_sq("d4").unwrap();
        let a = Action::encode(src, tgt, ActionKind::Skill, 5, 0); // Tempest
        let s = enc(a);
        assert_eq!(s, "b2*d4:Tempest");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 8: skill_focus_effect_mode ---
    #[test]
    fn skill_focus_effect_mode() {
        let src = notation_to_sq("b2").unwrap();
        let tgt = notation_to_sq("d4").unwrap();
        let a = Action::encode_focus_effect(src, tgt, ActionKind::Skill, 10, 0); // Blast~
        assert!(a.focus_effect_mode());
        let s = enc(a);
        assert_eq!(s, "b2*d4:Blast~");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 9: skill_focus_retarget_aux_eq_target ---
    #[test]
    fn skill_focus_retarget_aux_eq_target() {
        let src = notation_to_sq("b2").unwrap();
        let tgt = notation_to_sq("c3").unwrap();
        // Shield (id=6), aux == tgt → ">"
        let a = Action::encode_with_aux(src, tgt, ActionKind::Skill, 6, 0, tgt);
        let s = enc(a);
        assert_eq!(s, "b2*c3:Shield>");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 10: skill_focus_retarget_aux_ne_target ---
    #[test]
    fn skill_focus_retarget_aux_ne_target() {
        let src = notation_to_sq("b2").unwrap();
        let tgt = notation_to_sq("d4").unwrap();
        let aux = notation_to_sq("c3").unwrap();
        // Dash (id=9), aux != tgt → ">c3"
        let a = Action::encode_with_aux(src, tgt, ActionKind::Skill, 9, 0, aux);
        let s = enc(a);
        assert_eq!(s, "b2*d4:Dash>c3");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 11: skill_focus_effect_and_aux_combined ---
    #[test]
    fn skill_focus_effect_and_aux_combined() {
        let src = notation_to_sq("b2").unwrap();
        let tgt = notation_to_sq("d4").unwrap();
        let aux = notation_to_sq("c3").unwrap();
        // Both bits: encode_with_aux then set bit 22
        let mut a = Action::encode_with_aux(src, tgt, ActionKind::Skill, 9, 0, aux);
        a.0 |= 1 << 22;
        assert!(a.focus_effect_mode());
        assert!(a.has_aux());
        let s = enc(a);
        assert_eq!(s, "b2*d4:Dash~>c3");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 12: skill_shove_all_8_directions ---
    #[test]
    fn skill_shove_all_8_directions() {
        let src = notation_to_sq("d4").unwrap();
        let tgt = notation_to_sq("e5").unwrap();
        for (i, dir) in DIRS.iter().enumerate() {
            let a = Action::encode(src, tgt, ActionKind::Skill, 11, i as u8);
            let s = enc(a);
            assert_eq!(s, format!("d4*e5:Shove:{dir}"));
            let back = dec(&s);
            assert_eq!(back, a, "direction {dir}");
        }
    }

    // --- 13: endphase_roundtrip ---
    #[test]
    fn endphase_roundtrip() {
        let a = Action::encode(0, 0, ActionKind::EndPhase, 0, 0);
        assert_eq!(enc(a), "endphase");
        assert_eq!(dec("endphase"), a);
    }

    // --- 14: endturn_roundtrip ---
    #[test]
    fn endturn_roundtrip() {
        let a = Action::encode(0, 0, ActionKind::EndTurn, 0, 0);
        assert_eq!(enc(a), "endturn");
        assert_eq!(dec("endturn"), a);
    }

    // --- 15: draft_roundtrip ---
    #[test]
    fn draft_roundtrip() {
        // draft Lance@a1:1+Shield@b2:2
        let a = Action::encode_draft_turn(1, 0, 0, 6, 9, 1);
        let s = enc(a);
        assert_eq!(s, "draft Lance@a1:1+Shield@b2:2");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 16: draft_slot2 ---
    #[test]
    fn draft_slot2() {
        // Both picks use slot 2 (slot index 1)
        let a = Action::encode_draft_turn(2, 8, 1, 3, 16, 1);
        let s = enc(a);
        // slot2 encodes as "2" in notation
        assert!(s.contains(":2+"), "expected ':2+' in {s:?}");
        assert!(s.ends_with(":2"),  "expected trailing ':2' in {s:?}");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 17: draft_extremal ---
    #[test]
    fn draft_extremal() {
        // Charge (15), h8 (sq=63), slot2 (index=1) for both picks
        let a = Action::encode_draft_turn(15, 63, 1, 15, 63, 1);
        let s = enc(a);
        assert_eq!(s, "draft Charge@h8:2+Charge@h8:2");
        let back = dec(&s);
        assert_eq!(back, a);
    }

    // --- 18: bodyguard_decline ---
    #[test]
    fn bodyguard_decline() {
        let a = Action::encode_bodyguard_choice(0);
        assert_eq!(enc(a), "bgX");
        assert_eq!(dec("bgX"), a);
    }

    // --- 19: bodyguard_redirect_by_square ---
    #[test]
    fn bodyguard_redirect_by_square() {
        // a5 = sq 32
        let sq_a5 = notation_to_sq("a5").unwrap();
        let eligible = [sq_a5, 0, 0, 0];
        let pb = PendingBodyguard {
            attacker_src: 0, attacker_now: 0, target_sq: 0,
            eligible, eligible_len: 1,
        };
        let a = Action::encode_bodyguard_choice(1); // index 1 → eligible[0] = a5
        // Encoding with pending resolves to "bga5"
        let s = action_to_notation(a, Some(&pb));
        assert_eq!(s, "bga5");
        // Decoding "bga5" with matching pending yields the same action
        let back = dec_with_bg("bga5", &[sq_a5]);
        assert_eq!(back, a);
    }

    // --- 20: bodyguard_redirect_by_index ---
    #[test]
    fn bodyguard_redirect_by_index() {
        let a = Action::encode_bodyguard_choice(2);
        let back = dec("bg2");
        assert_eq!(back, a);
    }

    // --- 21: bodyguard_encode_without_pending ---
    #[test]
    fn bodyguard_encode_without_pending() {
        let a = Action::encode_bodyguard_choice(1);
        // With pending=None, falls back to "bg1"
        let s = action_to_notation(a, None);
        assert_eq!(s, "bg1");
    }

    // --- 22: bodyguard_no_pending ---
    #[test]
    fn bodyguard_no_pending() {
        let err = notation_to_action("bga5", &minimal_pos()).unwrap_err();
        assert_eq!(err, NotationError::NoPendingBodyguard);
    }

    // --- 23: bodyguard_square_not_in_eligible ---
    #[test]
    fn bodyguard_square_not_in_eligible() {
        // eligible contains b3, but we ask for a5
        let sq_b3 = notation_to_sq("b3").unwrap();
        let err = notation_to_action("bga5", &pos_with_bg(&[sq_b3])).unwrap_err();
        assert!(matches!(err, NotationError::BadBodyguardSquare(_)));
    }

    // --- 24: error_unknown_skill ---
    #[test]
    fn error_unknown_skill() {
        let err = notation_to_action("a1*b2:Nuke", &minimal_pos()).unwrap_err();
        assert_eq!(err, NotationError::UnknownSkill("Nuke".to_string()));
    }

    // --- 25: error_bad_square ---
    #[test]
    fn error_bad_square() {
        let err = notation_to_action("z9-a1", &minimal_pos()).unwrap_err();
        assert!(matches!(err, NotationError::BadSquare(_)));
    }

    // --- 26: error_trailing_input ---
    #[test]
    fn error_trailing_input() {
        let err = notation_to_action("endphaseXXX", &minimal_pos()).unwrap_err();
        assert_eq!(err, NotationError::TrailingInput("XXX".to_string()));
    }

    // --- 27: error_empty_input ---
    #[test]
    fn error_empty_input() {
        let err = notation_to_action("", &minimal_pos()).unwrap_err();
        assert_eq!(err, NotationError::EmptyInput);
    }

    // --- 28: error_unknown_direction ---
    #[test]
    fn error_unknown_direction() {
        let err = notation_to_action("a1*b2:Shove:XX", &minimal_pos()).unwrap_err();
        assert_eq!(err, NotationError::UnknownDirection("XX".to_string()));
    }
}
