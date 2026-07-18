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
//   bit  30      DRAFT_TURN_TAG (1 bit) - when set, the other bits use a
//                completely different layout: see encodeDraftTurn.
//   bit  31      BG_CHOICE_TAG (1 bit) - when set, the action is a
//                BodyguardChoice carrying only `idx` in bits 0..4 (0 =
//                decline redirect, k = redirect to eligible[k-1]). See
//                isBodyguardChoice / bgGuardIdx / encodeBodyguardChoice.
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

// === L8 - DraftTurn encoding ===============================================
//
// A DraftTurn is a u32 with bit 30 set (the `DRAFT_TURN_TAG`). When that bit
// is set, the remaining bits encode two (skill_id, sq, slot) picks the
// side-to-move is committing in one draft ply:
//
//   bits  0..4   skill1   (4 bits, 1..15)
//   bits  4..10  sq1      (6 bits)
//   bit  10      slot1    (1 bit, 0 = slot1, 1 = slot2)
//   bits 11..15  skill2
//   bits 15..21  sq2
//   bit  21      slot2
//   bit  30      DRAFT_TURN_TAG = 1
//
// Mirrors `Action::encode_draft_turn` in the Rust engine. The engine validates
// every cross-pick rule (target ownership, slot already filled, same-skill-on-
// same-piece) inside `legal_draft_turns`; the UI just encodes the player's
// two picks and submits via `tryApply`.

export const DRAFT_TURN_TAG = 1 << 30; // 0x4000_0000

export function isDraftTurn(u32: number): boolean {
  return ((u32 >>> 0) & DRAFT_TURN_TAG) !== 0;
}

export function encodeDraftTurn(
  skill1: number, sq1: number, slot1: number,
  skill2: number, sq2: number, slot2: number,
): number {
  const bits =
      (skill1 & 0xf)
    | ((sq1   & 0x3f) << 4)
    | ((slot1 & 0x1)  << 10)
    | ((skill2 & 0xf) << 11)
    | ((sq2   & 0x3f) << 15)
    | ((slot2 & 0x1)  << 21)
    | DRAFT_TURN_TAG;
  return bits >>> 0;
}

export interface DraftTurnDecoded {
  raw: number;
  pick1: { skillId: number; sq: number; slot: number };
  pick2: { skillId: number; sq: number; slot: number };
}

export function decodeDraftTurn(u32: number): DraftTurnDecoded {
  const v = u32 >>> 0;
  return {
    raw: v,
    pick1: {
      skillId: v & 0xf,
      sq: (v >>> 4) & 0x3f,
      slot: (v >>> 10) & 0x1,
    },
    pick2: {
      skillId: (v >>> 11) & 0xf,
      sq: (v >>> 15) & 0x3f,
      slot: (v >>> 21) & 0x1,
    },
  };
}

// === Commit 2 - BodyguardChoice encoding ====================================
//
// Mirrors `Action::encode_bodyguard_choice` / `is_bodyguard_choice` /
// `bg_guard_idx` in the Rust engine. A BodyguardChoice is the defender's reply
// to a tentatively-applied Move-Attack that left
// `Position::pendingBodyguard != null`. Bit 31 tags the action; bits 0..4 carry
// `idx` (0 = decline redirect, k = redirect to `pendingBodyguard.eligible[k-1]`).
//
// MUST stay byte-identical with the Rust encoding - wire/IDB serialised
// actions cross the boundary as raw u32 values.

export const ACTION_BG_CHOICE_TAG = (1 << 31) >>> 0; // 0x8000_0000
export const MAX_BODYGUARD_ELIGIBLE = 4;

export function isBodyguardChoice(u32: number): boolean {
  return ((u32 >>> 0) & ACTION_BG_CHOICE_TAG) !== 0;
}

export function bgGuardIdx(u32: number): number {
  return (u32 >>> 0) & 0xf;
}

export function encodeBodyguardChoice(idx: number): number {
  if (!Number.isInteger(idx) || idx < 0 || idx > MAX_BODYGUARD_ELIGIBLE) {
    throw new RangeError(
      `encodeBodyguardChoice: idx ${idx} out of range [0, ${MAX_BODYGUARD_ELIGIBLE}]`,
    );
  }
  return ((idx & 0xf) | ACTION_BG_CHOICE_TAG) >>> 0;
}
