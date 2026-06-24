// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import { setPendingMatchLog, consumePendingMatchLog } from "./library-handoff";

describe("library-handoff", () => {
  beforeEach(() => {
    sessionStorage.clear();
  });

  it("round-trips a value through sessionStorage", () => {
    setPendingMatchLog('{"start_fen":"…","plies":[]}');
    expect(consumePendingMatchLog()).toBe('{"start_fen":"…","plies":[]}');
  });

  it("returns null on consume after the value has been read once", () => {
    setPendingMatchLog("abc");
    expect(consumePendingMatchLog()).toBe("abc");
    expect(consumePendingMatchLog()).toBeNull();
  });

  it("returns null when nothing has been set", () => {
    expect(consumePendingMatchLog()).toBeNull();
  });
});
