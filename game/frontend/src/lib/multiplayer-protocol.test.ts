// Unit tests for the multiplayer wire protocol helpers.
//
// All four exported helpers are pure functions — no PeerJS, no DOM. Run
// under the default node env via `npm test`.

import { describe, it, expect } from "vitest";
import {
  generateCode,
  isValidCode,
  encodeMessage,
  decodeMessage,
  derivePillState,
  type WireMessage,
} from "./multiplayer-protocol";

describe("generateCode + isValidCode", () => {
  it("generateCode produces 6-digit codes that pass validation", () => {
    for (let i = 0; i < 200; i++) {
      const c = generateCode();
      expect(c).toMatch(/^[1-9][0-9]{5}$/);
      expect(isValidCode(c)).toBe(true);
    }
  });

  it("isValidCode rejects malformed input", () => {
    expect(isValidCode("000000")).toBe(false); // leading zero
    expect(isValidCode("12345")).toBe(false);  // too short
    expect(isValidCode("1234567")).toBe(false); // too long
    expect(isValidCode("12345a")).toBe(false);
    expect(isValidCode("")).toBe(false);
    expect(isValidCode("100000")).toBe(true);
    expect(isValidCode("999999")).toBe(true);
  });
});

describe("encode/decode round-trip", () => {
  const cases: WireMessage[] = [
    { kind: "ping", t: 1_234_567 },
    { kind: "pong", t: 0 },
    { kind: "error", reason: "session-full" },
  ];
  for (const m of cases) {
    it(`round-trips ${m.kind}${"reason" in m ? `:${m.reason}` : ""}`, () => {
      const s = encodeMessage(m);
      const back = decodeMessage(s);
      expect(back).toEqual(m);
    });
  }

  it("decodeMessage returns null for invalid input", () => {
    expect(decodeMessage("not json")).toBeNull();
    expect(decodeMessage("null")).toBeNull();
    expect(decodeMessage("[]")).toBeNull();
    expect(decodeMessage('{"kind":"unknown"}')).toBeNull();
    expect(decodeMessage('{"kind":"ping"}')).toBeNull(); // missing t
    expect(decodeMessage('{"kind":"error"}')).toBeNull(); // missing reason
    // V1 kinds removed in L7c Step 5 — these now decode as null.
    expect(decodeMessage('{"kind":"snapshot","snapshotJson":"{}"}')).toBeNull();
    expect(decodeMessage('{"kind":"ready"}')).toBeNull();
    expect(decodeMessage('{"kind":"action","raw":0}')).toBeNull();
    expect(decodeMessage('{"kind":"draft-mode","mode":"custom"}')).toBeNull();
    expect(decodeMessage('{"kind":"draft-turn","raw":0}')).toBeNull();
    expect(decodeMessage('{"kind":"resume-request","code":"123456","plyCount":0,"zobrist":"0"}')).toBeNull();
  });
});

describe("derivePillState", () => {
  const NOW = 1_000_000;

  it("connected + fresh pong → live", () => {
    expect(derivePillState("connected", NOW - 500, NOW)).toBe("live");
  });

  it("connected + 5s stale → unstable", () => {
    expect(derivePillState("connected", NOW - 5_000, NOW)).toBe("unstable");
  });

  it("connected + 15s stale → disconnected (still in grace)", () => {
    expect(derivePillState("connected", NOW - 15_000, NOW)).toBe("disconnected");
  });

  it("connected but no pong yet → unstable", () => {
    expect(derivePillState("connected", null, NOW)).toBe("unstable");
  });

  it("disconnected within grace → disconnected", () => {
    expect(derivePillState("disconnected", NOW - 1_000, NOW)).toBe("disconnected");
  });

  it("disconnected past 5-min grace → forfeit", () => {
    expect(derivePillState("disconnected", NOW - 6 * 60_000, NOW)).toBe("forfeit");
  });

  it("connected past 5-min stale → forfeit", () => {
    expect(derivePillState("connected", NOW - 6 * 60_000, NOW)).toBe("forfeit");
  });

  it("idle/hosting/joining/connecting/error map to disconnected (HUD should hide pill)", () => {
    for (const s of ["idle", "hosting", "joining", "connecting", "error"] as const) {
      expect(derivePillState(s, NOW - 100, NOW)).toBe("disconnected");
    }
  });
});
