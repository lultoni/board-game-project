// Tests for the loadout share-code + JSON codec.

import { describe, expect, it } from "vitest";
import type { SideLoadout } from "$lib/engine";
import { decodeShareCode, encodeJson, encodeShareCode, parseImport } from "./loadout-codec";

const A: SideLoadout = [[6, 9], [1, 6], [1, 10], [1, 9], [6, 10], [1, 9]] as const;
const EMPTY: SideLoadout = [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0]] as const;

describe("encodeShareCode / decodeShareCode", () => {
  it("round-trips a complete loadout with an ASCII name", () => {
    const code = encodeShareCode(A, "My Loadout");
    expect(code).toMatch(/^L1:/);
    const back = decodeShareCode(code);
    expect("error" in back).toBe(false);
    if ("error" in back) return;
    expect(back.loadout).toEqual(A);
    expect(back.name).toBe("My Loadout");
  });

  it("round-trips an empty loadout (all zeros)", () => {
    const code = encodeShareCode(EMPTY, "blank");
    const back = decodeShareCode(code);
    if ("error" in back) throw new Error(back.error);
    expect(back.loadout).toEqual(EMPTY);
    expect(back.name).toBe("blank");
  });

  it("round-trips a UTF-8 name (Umlauts / emoji)", () => {
    const code = encodeShareCode(A, "Königsspiel 🐉");
    const back = decodeShareCode(code);
    if ("error" in back) throw new Error(back.error);
    expect(back.name).toBe("Königsspiel 🐉");
  });

  it("is deterministic (same input → same code)", () => {
    expect(encodeShareCode(A, "x")).toBe(encodeShareCode(A, "x"));
  });

  it("uses base64url alphabet (no +, /, or =)", () => {
    const code = encodeShareCode(A, "z".repeat(50));
    expect(code.slice(3)).not.toMatch(/[+/=]/);
  });

  it("rejects skill IDs > 15", () => {
    const bad: SideLoadout = [[16, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0]] as const;
    expect(() => encodeShareCode(bad, "x")).toThrow(/out of range/);
  });

  it("rejects names over 63 bytes", () => {
    expect(() => encodeShareCode(A, "x".repeat(64))).toThrow(/too long/);
  });

  it("decode rejects missing prefix", () => {
    const r = decodeShareCode("not-a-code");
    expect("error" in r).toBe(true);
  });

  it("decode rejects truncated payload", () => {
    const r = decodeShareCode("L1:AAAA");
    expect("error" in r).toBe(true);
  });

  it("decode rejects length-mismatched payload", () => {
    const good = encodeShareCode(A, "hi");
    // Chop off the tail so declared-name-length exceeds actual bytes.
    const r = decodeShareCode(good.slice(0, good.length - 1));
    expect("error" in r).toBe(true);
  });
});

describe("encodeJson / parseImport", () => {
  it("parseImport handles a share code", () => {
    const code = encodeShareCode(A, "shared");
    const r = parseImport(code);
    if ("error" in r) throw new Error(r.error);
    expect(r.loadout).toEqual(A);
    expect(r.name).toBe("shared");
  });

  it("parseImport handles JSON export", () => {
    const json = encodeJson({ name: "archived", loadout: A });
    const r = parseImport(json);
    if ("error" in r) throw new Error(r.error);
    expect(r.loadout).toEqual(A);
    expect(r.name).toBe("archived");
  });

  it("parseImport trims whitespace", () => {
    const code = encodeShareCode(A, "x");
    const r = parseImport("  " + code + "\n");
    expect("error" in r).toBe(false);
  });

  it("parseImport rejects garbage", () => {
    expect("error" in parseImport("not json and not L1")).toBe(true);
  });

  it("parseImport rejects JSON without loadout array", () => {
    expect("error" in parseImport('{"name":"x"}')).toBe(true);
  });

  it("parseImport rejects JSON with wrong loadout length", () => {
    expect("error" in parseImport('{"name":"x","loadout":[[1,2]]}')).toBe(true);
  });

  it("parseImport rejects JSON with out-of-range skill", () => {
    expect(
      "error" in
        parseImport(
          '{"name":"x","loadout":[[99,0],[0,0],[0,0],[0,0],[0,0],[0,0]]}',
        ),
    ).toBe(true);
  });

  it("parseImport rejects JSON with non-string name", () => {
    expect(
      "error" in
        parseImport(
          '{"name":42,"loadout":[[1,2],[3,4],[5,6],[7,8],[9,10],[11,12]]}',
        ),
    ).toBe(true);
  });
});
