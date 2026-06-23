// u32 Action encoding (mirrors `core_engine/src/game_logic/action.rs`).
//
//   bits  0..6   src              (6 bits)
//   bits  6..12  target           (6 bits)
//   bits 12..14  kind             (2 bits)  0=Move 1=Skill 2=EndPhase 3=EndTurn
//   bits 14..18  skill_id         (4 bits)  0=none, 1..15 = Skill enum
//   bits 18..22  choice_idx       (4 bits)  player disambiguation
//   bit  22      focus_mode       (1 bit)
//   bits 23..29  aux_sq / approach_sq (6 bits, dual use)
//   bit  29      has_aux / has_approach (1 bit)
//   bits 30..32  reserved
//
// Numbers are u32; we always operate via `>>>` so JS's signed-shift semantics
// don't bite us.

export const ActionKind = {
  Move: 0,
  Skill: 1,
  EndPhase: 2,
  EndTurn: 3,
} as const;
export type ActionKindValue = (typeof ActionKind)[keyof typeof ActionKind];

export interface ActionDecoded {
  raw: number;
  src: number;
  target: number;
  kind: ActionKindValue;
  skillId: number;
  choiceIdx: number;
  focusMode: boolean;
  hasAux: boolean;
  /** Same bits as approachSq; semantics depend on kind. */
  auxSq: number;
}

export function decodeAction(u32: number): ActionDecoded {
  const v = u32 >>> 0;
  return {
    raw: v,
    src: v & 0x3f,
    target: (v >>> 6) & 0x3f,
    kind: ((v >>> 12) & 0x3) as ActionKindValue,
    skillId: (v >>> 14) & 0xf,
    choiceIdx: (v >>> 18) & 0xf,
    focusMode: ((v >>> 22) & 0x1) === 1,
    auxSq: (v >>> 23) & 0x3f,
    hasAux: ((v >>> 29) & 0x1) === 1,
  };
}

export function actionKindName(k: ActionKindValue): string {
  switch (k) {
    case ActionKind.Move:
      return "Move";
    case ActionKind.Skill:
      return "Skill";
    case ActionKind.EndPhase:
      return "EndPhase";
    case ActionKind.EndTurn:
      return "EndTurn";
  }
}
