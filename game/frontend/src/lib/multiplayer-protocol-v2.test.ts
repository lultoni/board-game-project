// Unit tests for the v2 wire protocol (authoritative-host redesign).
// See `.claude/plans/twinkling-questing-quiche.md` for context.

import { describe, it, expect } from "vitest";
import {
  decodeMessageV2,
  encodeMessageV2,
  newIntentNonce,
  type WireMessageV2,
} from "./multiplayer-protocol-v2";

describe("encode/decode round-trip (v2)", () => {
  const cases: WireMessageV2[] = [
    { kind: "ping", t: 1_234_567 },
    { kind: "pong", t: 0 },
    { kind: "session-hello", matchId: "01ARZ3NDEKTSV4RRFFQ69G5FAV", phase: "draft", seq: 0, code: "281947" },
    { kind: "session-hello", matchId: "abc", phase: "play", seq: 42, code: "100000" },
    { kind: "intent", phase: "draft", nonce: "i-abc123", raw: 0 },
    { kind: "intent", phase: "play", nonce: "i-zz", raw: 0xffffffff },
    { kind: "intent", phase: "play", nonce: "i-t", raw: 5, thoughtMs: 2500 },
    { kind: "intent", phase: "play", nonce: "i-t0", raw: 5, thoughtMs: 0 },
    {
      kind: "committed",
      seq: 1,
      phase: "draft",
      raw: 7,
      postZobrist: "12345678901234567890",
      originNonce: null,
    },
    {
      kind: "committed",
      seq: 99,
      phase: "play",
      raw: 0xdeadbe,
      postZobrist: "0",
      originNonce: "i-abc123",
    },
    { kind: "intent-rejected", nonce: "i-abc", reason: "illegal" },
    { kind: "intent-rejected", nonce: "i-abc", reason: "out-of-turn" },
    { kind: "intent-rejected", nonce: "i-abc", reason: "phase-mismatch" },
    { kind: "intent-rejected", nonce: "i-abc", reason: "paused" },
    {
      kind: "phase-change",
      from: "draft",
      to: "play",
      snapshotJson: '{"start_fen":"…","actions":[1,2],"config":{}}',
      seq: 12,
    },
    {
      kind: "snapshot",
      snapshotJson: '{"start_fen":"…","actions":[],"config":{}}',
      seq: 0,
      phase: "draft",
      matchId: "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    },
    { kind: "request-snapshot", mySeq: 0, reason: "reconnect" },
    { kind: "request-snapshot", mySeq: 5, reason: "audit-mismatch" },
    { kind: "request-snapshot", mySeq: 5, reason: "stale" },
    { kind: "cheat-detected", seq: 7, raw: 0xdead },
    { kind: "handoff-announce", matchId: "newrow", seq: 18 },
    { kind: "paused" },
    { kind: "resumed" },
    { kind: "error", reason: "session-full" },
  ];

  for (const m of cases) {
    const label = "reason" in m ? `${m.kind}:${m.reason}` : m.kind;
    it(`round-trips ${label}`, () => {
      const s = encodeMessageV2(m);
      const back = decodeMessageV2(s);
      expect(back).toEqual(m);
    });
  }
});

describe("decodeMessageV2 rejects malformed payloads", () => {
  it("returns null for non-JSON / non-object / unknown kind", () => {
    expect(decodeMessageV2("not json")).toBeNull();
    expect(decodeMessageV2("null")).toBeNull();
    expect(decodeMessageV2("[]")).toBeNull();
    expect(decodeMessageV2('{"kind":"unknown"}')).toBeNull();
  });

  it("ping/pong require numeric t", () => {
    expect(decodeMessageV2('{"kind":"ping"}')).toBeNull();
    expect(decodeMessageV2('{"kind":"ping","t":"oops"}')).toBeNull();
    expect(decodeMessageV2('{"kind":"pong","t":1}')).toEqual({ kind: "pong", t: 1 });
  });

  it("session-hello requires non-empty matchId, valid phase, u32 seq, non-empty code", () => {
    expect(
      decodeMessageV2('{"kind":"session-hello","matchId":"","phase":"draft","seq":0,"code":"281947"}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"session-hello","matchId":"x","phase":"nope","seq":0,"code":"281947"}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"session-hello","matchId":"x","phase":"draft","seq":-1,"code":"281947"}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"session-hello","matchId":"x","phase":"draft","seq":0,"code":""}'),
    ).toBeNull();
  });

  it("intent requires non-empty nonce, valid phase, u32 raw", () => {
    expect(decodeMessageV2('{"kind":"intent","phase":"draft","nonce":"","raw":0}')).toBeNull();
    expect(decodeMessageV2('{"kind":"intent","phase":"foo","nonce":"i-x","raw":0}')).toBeNull();
    expect(decodeMessageV2('{"kind":"intent","phase":"draft","nonce":"i-x","raw":-1}')).toBeNull();
    expect(decodeMessageV2('{"kind":"intent","phase":"draft","nonce":"i-x","raw":4294967296}')).toBeNull();
    expect(decodeMessageV2('{"kind":"intent","phase":"draft","nonce":"i-x","raw":1.5}')).toBeNull();
  });

  it("intent accepts optional thoughtMs but rejects a malformed one (B2)", () => {
    // Absent → valid (back-compat with pre-B2 joiners).
    expect(decodeMessageV2('{"kind":"intent","phase":"play","nonce":"i-x","raw":0}')).not.toBeNull();
    // Valid non-negative number → accepted.
    expect(decodeMessageV2('{"kind":"intent","phase":"play","nonce":"i-x","raw":0,"thoughtMs":1200}')).not.toBeNull();
    // Negative / non-finite / non-number → whole message rejected.
    expect(decodeMessageV2('{"kind":"intent","phase":"play","nonce":"i-x","raw":0,"thoughtMs":-1}')).toBeNull();
    expect(decodeMessageV2('{"kind":"intent","phase":"play","nonce":"i-x","raw":0,"thoughtMs":"5"}')).toBeNull();
  });

  it("committed requires seq>=1, decimal-string postZobrist, originNonce null or non-empty string", () => {
    expect(
      decodeMessageV2('{"kind":"committed","seq":0,"phase":"draft","raw":1,"postZobrist":"1","originNonce":null}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"committed","seq":1,"phase":"draft","raw":1,"postZobrist":"abc","originNonce":null}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"committed","seq":1,"phase":"draft","raw":1,"postZobrist":"1","originNonce":""}'),
    ).toBeNull();
    // valid
    expect(
      decodeMessageV2('{"kind":"committed","seq":1,"phase":"draft","raw":1,"postZobrist":"1","originNonce":null}'),
    ).not.toBeNull();
  });

  it("intent-rejected requires known reason", () => {
    expect(decodeMessageV2('{"kind":"intent-rejected","nonce":"i-x","reason":"made-up"}')).toBeNull();
    expect(decodeMessageV2('{"kind":"intent-rejected","nonce":"i-x","reason":"illegal"}')).not.toBeNull();
  });

  it("phase-change requires from=draft, to=play, non-empty snapshotJson, u32 seq", () => {
    expect(
      decodeMessageV2('{"kind":"phase-change","from":"play","to":"play","snapshotJson":"{}","seq":0}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"phase-change","from":"draft","to":"draft","snapshotJson":"{}","seq":0}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"phase-change","from":"draft","to":"play","snapshotJson":"","seq":0}'),
    ).toBeNull();
  });

  it("snapshot requires non-empty snapshotJson, u32 seq, valid phase, non-empty matchId", () => {
    expect(
      decodeMessageV2('{"kind":"snapshot","snapshotJson":"","seq":0,"phase":"draft","matchId":"x"}'),
    ).toBeNull();
    expect(
      decodeMessageV2('{"kind":"snapshot","snapshotJson":"{}","seq":0,"phase":"draft","matchId":""}'),
    ).toBeNull();
  });

  it("request-snapshot requires u32 mySeq and known reason", () => {
    expect(decodeMessageV2('{"kind":"request-snapshot","mySeq":-1,"reason":"reconnect"}')).toBeNull();
    expect(decodeMessageV2('{"kind":"request-snapshot","mySeq":0,"reason":"made-up"}')).toBeNull();
  });

  it("cheat-detected requires seq>=1 and u32 raw", () => {
    expect(decodeMessageV2('{"kind":"cheat-detected","seq":0,"raw":0}')).toBeNull();
    expect(decodeMessageV2('{"kind":"cheat-detected","seq":1,"raw":-1}')).toBeNull();
  });

  it("handoff-announce requires non-empty matchId and u32 seq", () => {
    expect(decodeMessageV2('{"kind":"handoff-announce","matchId":"","seq":0}')).toBeNull();
    expect(decodeMessageV2('{"kind":"handoff-announce","matchId":"x","seq":-1}')).toBeNull();
  });

  it("bodyguard-prompt is rejected as an unknown kind (removed in Phase 1e)", () => {
    expect(decodeMessageV2('{"kind":"bodyguard-prompt","src":0,"target":0,"approach":0}')).toBeNull();
  });
});

describe("newIntentNonce", () => {
  it("produces unique-looking nonces", () => {
    const seen = new Set<string>();
    for (let i = 0; i < 100; i++) {
      const n = newIntentNonce();
      expect(n).toMatch(/^i-/);
      expect(n.length).toBeGreaterThan(4);
      seen.add(n);
    }
    // Not a strict uniqueness guarantee, but collisions in 100 generations
    // would indicate something is very wrong with the randomness.
    expect(seen.size).toBeGreaterThan(95);
  });
});
