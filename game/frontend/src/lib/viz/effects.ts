// Lightweight FX queue consumed by the Canvas overlay. Components push
// effect descriptors after each `tryApply` and the overlay drains them over
// the next ~700ms. Effects own no state outside the canvas itself.

export type Effect =
  | {
      kind: "dust";
      /** path of squares walked, src → ... → final (inclusive). */
      path: number[];
      /** ms timestamp when this effect was created. */
      startedAt: number;
    }
  | {
      kind: "impact";
      at: number;
      startedAt: number;
    }
  | {
      kind: "damageNumber";
      at: number;
      amount: number;
      startedAt: number;
    }
  | {
      kind: "shake";
      /** Square whose piece should shake briefly. */
      at: number;
      startedAt: number;
    };

export const FX_LIFETIME_MS = {
  dust: 650,
  impact: 450,
  damageNumber: 800,
  shake: 320,
} as const;
