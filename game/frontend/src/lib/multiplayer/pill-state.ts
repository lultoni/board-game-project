// Pure derivation for the connectivity-pill UI signal + the
// `disconnectedSince` anchor write. Lives outside `multiplayer.svelte.ts` so
// it can be unit-tested without runes.
//
// The wrapper module's `pillState()` getter used to inline the anchor write,
// which made it a getter with a hidden side-effect. Now the derivation
// returns *what* the anchor should be (or null = leave as-is); the wrapper
// is the one source that actually writes `mpState.disconnectedSince`.

import { derivePillState, type MpStatus, type PillState } from "../multiplayer-protocol";

export interface PillStateInput {
  status: MpStatus;
  lastPongAt: number | null;
  now: number;
  peerEverPaired: boolean;
  disconnectedSince: number | null;
}

export interface PillStateOutput {
  pill: PillState;
  /** New value for `disconnectedSince` — `null` means "leave the existing
   *  value as-is". When non-null, the caller writes it back to mpState.
   *  We only emit a non-null anchor when we'd actually be transitioning an
   *  unanchored disconnect into the visible UI, AND a peer was paired at
   *  some point (so host pre-join "hosting"→"disconnected" never anchors). */
  nextDisconnectedSince: number | null;
}

/** Derive the pill state and, when the pill enters a disconnected branch
 *  without a prior anchor, propose the anchor timestamp the wrapper should
 *  write. Pure — no I/O, no mutation of inputs. */
export function derivePillStateWithAnchor(input: PillStateInput): PillStateOutput {
  const pill = derivePillState(input.status, input.lastPongAt, input.now);
  const wantsAnchor =
    (pill === "disconnected" || pill === "forfeit")
    && input.disconnectedSince === null
    && input.peerEverPaired;
  return {
    pill,
    nextDisconnectedSince: wantsAnchor ? input.now : null,
  };
}
