// Centralised human-readable Action labels. All three places that render
// action strings (inspector legal-action picker, inspector tree edges via
// MoveListItem, AiHintBanner, and the match HUD if it ever needs labels)
// should call `formatAction`. This is the single source of truth for
// disambiguating variants of the same (src, target, skillId) tuple: approach
// squares for Move-Attacks, push directions for Shove, Bodyguard redirects,
// Focus-retargeted recipients, and Focus-effect mode for Blast/Shove.

import { ActionKind, decodeAction } from "./action";
import { skillById } from "./skills";

export function formatSquare(sq: number): string {
  const file = String.fromCharCode("a".charCodeAt(0) + (sq % 8));
  const rank = Math.floor(sq / 8) + 1;
  return `${file}${rank}`;
}

// 8-direction compass for Shove. Order matches `magic::neighbour_in_dir` in
// the engine (N, NE, E, SE, S, SW, W, NW).
const DIR_ARROWS = ["↑", "↗", "→", "↘", "↓", "↙", "←", "↖"];

// Skills whose Focus interpretation retargets onto an adjacent ally (the
// recipient is encoded in `auxSq`). Matches the generator's emit_focus_*
// helpers in core_engine/src/game_logic/generator.rs.
const FOCUS_RETARGET_SKILLS = new Set<number>([
  6,  // shield
  9,  // dash
  13, // retreat
]);

export function formatAction(raw: number): string {
  const d = decodeAction(raw);
  if (d.kind === ActionKind.EndPhase) return "End phase";
  if (d.kind === ActionKind.EndTurn) return "End turn";

  if (d.kind === ActionKind.Move) {
    // Plain Move: src→target. Move-Attack: hasAux + auxSq = approach square,
    // optionally choiceIdx > 0 = Bodyguard redirect onto the (choiceIdx-1)-th
    // adjacent friendly Guard.
    if (!d.hasAux) {
      return `Move ${formatSquare(d.src)}→${formatSquare(d.target)}`;
    }
    let s = `Atk ${formatSquare(d.src)}→${formatSquare(d.target)} via ${formatSquare(d.auxSq)}`;
    if (d.choiceIdx > 0) {
      s += ` (BG #${d.choiceIdx})`;
    }
    return s;
  }

  if (d.kind === ActionKind.Skill) {
    const info = skillById(d.skillId);
    const name = info?.key ?? `s${d.skillId}`;
    const srcSq = formatSquare(d.src);
    const tgtSq = formatSquare(d.target);

    // Shove (id 11): choiceIdx is the push direction 0..7.
    if (d.skillId === 11) {
      const arrow = DIR_ARROWS[d.choiceIdx] ?? "?";
      const tag = d.focusMode ? " [focus-effect 2]" : "";
      return `${name} ${srcSq}→${tgtSq} ${arrow}${tag}`;
    }

    // Focus-retarget skills (Shield/Dash/Retreat): hasAux + auxSq = ally
    // recipient. The engine treats `target` as the destination square (Dash/
    // Retreat) or mirrors auxSq (Shield); always show the recipient.
    if (d.hasAux && FOCUS_RETARGET_SKILLS.has(d.skillId)) {
      return `${name} ${srcSq}→${tgtSq} via ${formatSquare(d.auxSq)} [focus-retarget]`;
    }

    // Blast (id 10) with focus_effect_mode = 2-tile push variant.
    if (d.skillId === 10 && d.focusMode) {
      return `${name} ${srcSq}→${tgtSq} [focus-effect 2]`;
    }

    // Generic case: append [focus] when the activation-range buff branch is
    // selected on a non-Move skill (the engine emits one action per branch).
    const mods = d.focusMode ? " [focus]" : "";
    return `${name} ${srcSq}→${tgtSq}${mods}`;
  }

  return "?";
}
