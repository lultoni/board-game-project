import { describe, expect, it } from "vitest";
import { _validateSettings } from "./settings.svelte";

describe("validateSettings", () => {
  it("returns defaults when input is not an object", () => {
    const s = _validateSettings(null);
    expect(s.audioVolume).toBe(0.6);
    expect(s.locale).toBe("en");
    expect(s.p1ThinkTimeMs).toBe(1000);
  });

  it("returns defaults when input is undefined", () => {
    const s = _validateSettings(undefined);
    expect(s.locale).toBe("en");
  });

  it("clamps audioVolume above 1 down to 1", () => {
    const s = _validateSettings({ audioVolume: 2.5 });
    expect(s.audioVolume).toBe(1);
  });

  it("clamps negative audioVolume up to 0", () => {
    const s = _validateSettings({ audioVolume: -3 });
    expect(s.audioVolume).toBe(0);
  });

  it("falls back when audioVolume is NaN", () => {
    const s = _validateSettings({ audioVolume: NaN });
    expect(s.audioVolume).toBe(0.6);
  });

  it("falls back when audioVolume is non-numeric", () => {
    const s = _validateSettings({ audioVolume: "loud" });
    expect(s.audioVolume).toBe(0.6);
  });

  it("rejects unknown locale and falls back to en", () => {
    const s = _validateSettings({ locale: "klingon" });
    expect(s.locale).toBe("en");
  });

  it("accepts de as a valid locale", () => {
    const s = _validateSettings({ locale: "de" });
    expect(s.locale).toBe("de");
  });

  it("rejects negative think-time and falls back", () => {
    const s = _validateSettings({ p1ThinkTimeMs: -100 });
    expect(s.p1ThinkTimeMs).toBe(1000);
  });

  it("accepts zero think-time", () => {
    const s = _validateSettings({ p1ThinkTimeMs: 0 });
    expect(s.p1ThinkTimeMs).toBe(0);
  });

  it("rejects Infinity think-time", () => {
    const s = _validateSettings({ p2ThinkTimeMs: Infinity });
    expect(s.p2ThinkTimeMs).toBe(1000);
  });

  it("rejects non-integer max-depth", () => {
    const s = _validateSettings({ p1MaxDepth: 3.5 });
    expect(s.p1MaxDepth).toBe(6);
  });

  it("rejects zero max-depth (must be positive)", () => {
    const s = _validateSettings({ p1MaxDepth: 0 });
    expect(s.p1MaxDepth).toBe(6);
  });

  it("rejects negative max-depth", () => {
    const s = _validateSettings({ p2MaxDepth: -2 });
    expect(s.p2MaxDepth).toBe(6);
  });

  it("rejects non-boolean toggles", () => {
    const s = _validateSettings({ showLegalTargets: "yes" });
    expect(s.showLegalTargets).toBe(true);
  });

  it("preserves valid mixed-tampered settings field-by-field", () => {
    const s = _validateSettings({
      audioVolume: 0.3,
      locale: "fr", // invalid
      p1ThinkTimeMs: 500,
      p2MaxDepth: -1, // invalid
    });
    expect(s.audioVolume).toBe(0.3);
    expect(s.locale).toBe("en");
    expect(s.p1ThinkTimeMs).toBe(500);
    expect(s.p2MaxDepth).toBe(6);
  });

  it("ignores unknown extra keys", () => {
    const s = _validateSettings({ audioVolume: 0.5, evilFlag: true });
    expect(s.audioVolume).toBe(0.5);
    expect((s as unknown as { evilFlag?: unknown }).evilFlag).toBeUndefined();
  });
});
