import { describe, expect, it } from "vitest";
import {
  SNAPSHOT_BUDGETS,
  SnapshotValidationError,
  validateMatchLog,
  validateSnapshot,
} from "./snapshot-validator";

const baseOpts = {
  maxActions: SNAPSHOT_BUDGETS.RESUME_MAX_ACTIONS,
  maxJsonBytes: SNAPSHOT_BUDGETS.MAX_JSON_BYTES,
  requireConfig: true,
  source: "host-snapshot" as const,
};

describe("validateSnapshot", () => {
  it("accepts a well-formed snapshot envelope", () => {
    const json = JSON.stringify({
      start_fen: "8/8",
      actions: [1, 2, 3],
      config: { board_w: 8, board_h: 8 },
    });
    const out = validateSnapshot(json, baseOpts);
    expect(out.json).toBe(json);
    expect(out.actionCount).toBe(3);
  });

  it("accepts an empty actions array", () => {
    const json = JSON.stringify({
      start_fen: "8/8",
      actions: [],
      config: {},
    });
    expect(validateSnapshot(json, baseOpts).actionCount).toBe(0);
  });

  it("accepts an omitted actions field", () => {
    const json = JSON.stringify({ start_fen: "8/8", config: {} });
    expect(validateSnapshot(json, baseOpts).actionCount).toBe(0);
  });

  it("treats non-string input as not-a-string", () => {
    expect(() => validateSnapshot(123, baseOpts)).toThrow(SnapshotValidationError);
    try {
      validateSnapshot(123, baseOpts);
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("not-a-string");
      expect((e as SnapshotValidationError).source).toBe("host-snapshot");
    }
  });

  it("rejects oversized payloads", () => {
    const big = "x".repeat(10);
    try {
      validateSnapshot(big, { ...baseOpts, maxJsonBytes: 5 });
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("too-large");
    }
  });

  it("rejects malformed JSON", () => {
    try {
      validateSnapshot("{not json", baseOpts);
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("malformed-json");
    }
  });

  it("rejects null root", () => {
    try {
      validateSnapshot("null", baseOpts);
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("missing-start-fen");
    }
  });

  it("rejects missing start_fen", () => {
    try {
      validateSnapshot(JSON.stringify({ actions: [], config: {} }), baseOpts);
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("missing-start-fen");
    }
  });

  it("rejects empty start_fen", () => {
    try {
      validateSnapshot(JSON.stringify({ start_fen: "", config: {} }), baseOpts);
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("missing-start-fen");
    }
  });

  it("rejects missing config when requireConfig is true", () => {
    try {
      validateSnapshot(JSON.stringify({ start_fen: "8/8", actions: [] }), baseOpts);
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("missing-config");
    }
  });

  it("allows missing config when requireConfig is false", () => {
    const json = JSON.stringify({ start_fen: "8/8", actions: [] });
    expect(() => validateSnapshot(json, { ...baseOpts, requireConfig: false })).not.toThrow();
  });

  it("rejects actions that aren't an array", () => {
    try {
      validateSnapshot(JSON.stringify({ start_fen: "8/8", actions: "nope", config: {} }), baseOpts);
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("actions-not-array");
    }
  });

  it("rejects actions arrays that exceed maxActions", () => {
    const actions = Array.from({ length: 10 }, (_, i) => i);
    try {
      validateSnapshot(
        JSON.stringify({ start_fen: "8/8", actions, config: {} }),
        { ...baseOpts, maxActions: 5 }
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("actions-too-many");
    }
  });

  it("rejects negative action ints", () => {
    try {
      validateSnapshot(
        JSON.stringify({ start_fen: "8/8", actions: [1, -1, 3], config: {} }),
        baseOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("action-malformed");
    }
  });

  it("rejects non-integer actions", () => {
    try {
      validateSnapshot(
        JSON.stringify({ start_fen: "8/8", actions: [1.5], config: {} }),
        baseOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("action-malformed");
    }
  });

  it("rejects actions outside u32 range", () => {
    try {
      validateSnapshot(
        JSON.stringify({ start_fen: "8/8", actions: [0x100000000], config: {} }),
        baseOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("action-malformed");
    }
  });

  it("preserves the source field on the error", () => {
    try {
      validateSnapshot(123, { ...baseOpts, source: "joiner-paste" });
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).source).toBe("joiner-paste");
    }
  });
});

describe("validateMatchLog", () => {
  const logOpts = { ...baseOpts, source: "library-handoff" as const };

  it("accepts a well-formed match log", () => {
    const json = JSON.stringify({
      start_fen: "8/8",
      config: { board_w: 8 },
      plies: [{ action: { raw: 11 } }, { action: { raw: 22 } }],
    });
    const out = validateMatchLog(json, logOpts);
    expect(out.actionCount).toBe(2);
  });

  it("accepts an omitted plies field", () => {
    const json = JSON.stringify({ start_fen: "8/8", config: {} });
    expect(validateMatchLog(json, logOpts).actionCount).toBe(0);
  });

  it("rejects malformed plies array", () => {
    try {
      validateMatchLog(
        JSON.stringify({ start_fen: "8/8", config: {}, plies: "nope" }),
        logOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("plies-not-array");
    }
  });

  it("rejects plies arrays that exceed maxActions", () => {
    const plies = Array.from({ length: 10 }, (_, i) => ({ action: { raw: i } }));
    try {
      validateMatchLog(
        JSON.stringify({ start_fen: "8/8", config: {}, plies }),
        { ...logOpts, maxActions: 5 }
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("plies-too-many");
    }
  });

  it("rejects ply with missing action.raw", () => {
    try {
      validateMatchLog(
        JSON.stringify({ start_fen: "8/8", config: {}, plies: [{ action: {} }] }),
        logOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("ply-malformed");
    }
  });

  it("rejects ply with negative raw", () => {
    try {
      validateMatchLog(
        JSON.stringify({ start_fen: "8/8", config: {}, plies: [{ action: { raw: -1 } }] }),
        logOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("ply-malformed");
    }
  });

  it("rejects ply with raw above u32 max", () => {
    try {
      validateMatchLog(
        JSON.stringify({
          start_fen: "8/8",
          config: {},
          plies: [{ action: { raw: 0x100000000 } }],
        }),
        logOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("ply-malformed");
    }
  });

  it("rejects ply with null entry", () => {
    try {
      validateMatchLog(
        JSON.stringify({ start_fen: "8/8", config: {}, plies: [null] }),
        logOpts
      );
      throw new Error("should have thrown");
    } catch (e) {
      expect((e as SnapshotValidationError).reason).toBe("ply-malformed");
    }
  });
});
