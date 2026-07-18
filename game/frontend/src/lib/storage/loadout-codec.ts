// Share-code + JSON codec for custom loadouts.
//
// Two on-the-wire forms:
//
//  1. **Share code** - short, quick to copy/paste. Format:
//     `L1:<base64url>` where the body is:
//        6 bytes of packed skill IDs  (12 × 4 bits, MSB-first, piece order
//                                      matches SideLoadout - King @ 0, then
//                                      5 Champions; slot 1 nibble before
//                                      slot 2 within each piece)
//        1 byte name length N         (UTF-8 byte length; hard cap 63)
//        N bytes of UTF-8 name
//
//     `L1:` is the version prefix. Future breaking changes bump to `L2:` etc.
//     Skill IDs must fit in 4 bits, i.e. 0..15 - the current SKILL_COUNT is
//     15 so slot value 0 (empty) plus 1..15 is exactly the representable set.
//
//  2. **JSON** - archival form. Plain `{ name, loadout }` object. `id` and
//     `createdAt` are minted fresh on import so re-importing an old export
//     never collides with an existing row's ID.
//
// `parseImport` auto-detects: any string starting with `L1:` is treated as
// a share code; anything else is fed to JSON.parse.

import type { SideLoadout } from "$lib/engine";

const SHARE_PREFIX = "L1:";
const MAX_NAME_BYTES = 63;
const PIECE_COUNT = 6;
const SLOTS_PER_PIECE = 2;

type OkPayload = { loadout: SideLoadout; name: string };
type CodecResult = OkPayload | { error: string };

/** Encode a loadout + name into a `L1:<base64url>` share string. Throws on
 *  invalid input (skill out of range 0..15, name too long). Encoding is
 *  deterministic - same input always produces the same string. */
export function encodeShareCode(loadout: SideLoadout, name: string): string {
  const nameBytes = new TextEncoder().encode(name);
  if (nameBytes.length > MAX_NAME_BYTES) {
    throw new Error(`loadout name too long: ${nameBytes.length} bytes, max ${MAX_NAME_BYTES}`);
  }
  const bytes = new Uint8Array(6 + 1 + nameBytes.length);
  // Pack 12 nibbles into 6 bytes.
  let bitBuf = 0;
  let bitCount = 0;
  let byteIdx = 0;
  for (let piece = 0; piece < PIECE_COUNT; piece++) {
    for (let slot = 0; slot < SLOTS_PER_PIECE; slot++) {
      const id = loadout[piece][slot];
      if (id < 0 || id > 15 || !Number.isInteger(id)) {
        throw new Error(`skill id out of range at piece ${piece} slot ${slot}: ${id}`);
      }
      bitBuf = (bitBuf << 4) | id;
      bitCount += 4;
      if (bitCount === 8) {
        bytes[byteIdx++] = bitBuf & 0xff;
        bitBuf = 0;
        bitCount = 0;
      }
    }
  }
  bytes[6] = nameBytes.length;
  bytes.set(nameBytes, 7);
  return SHARE_PREFIX + toBase64Url(bytes);
}

/** Decode a share code produced by `encodeShareCode`. Returns the loadout +
 *  name on success, or `{ error }` on any malformed input. Never throws. */
export function decodeShareCode(s: string): CodecResult {
  if (!s.startsWith(SHARE_PREFIX)) {
    return { error: `missing ${SHARE_PREFIX} prefix` };
  }
  let bytes: Uint8Array;
  try {
    bytes = fromBase64Url(s.slice(SHARE_PREFIX.length));
  } catch (e) {
    return { error: `invalid base64: ${(e as Error).message}` };
  }
  if (bytes.length < 7) {
    return { error: `truncated: got ${bytes.length} bytes, need ≥ 7` };
  }
  const nameLen = bytes[6];
  if (bytes.length !== 7 + nameLen) {
    return { error: `length mismatch: header says name is ${nameLen}, payload has ${bytes.length - 7}` };
  }
  const pairs: [number, number][] = [];
  let bitBuf = 0;
  let bitCount = 0;
  let byteIdx = 0;
  for (let piece = 0; piece < PIECE_COUNT; piece++) {
    const slots: number[] = [];
    for (let slot = 0; slot < SLOTS_PER_PIECE; slot++) {
      while (bitCount < 4) {
        bitBuf = (bitBuf << 8) | bytes[byteIdx++];
        bitCount += 8;
      }
      bitCount -= 4;
      slots.push((bitBuf >> bitCount) & 0xf);
    }
    pairs.push([slots[0], slots[1]]);
  }
  let name: string;
  try {
    name = new TextDecoder("utf-8", { fatal: true }).decode(bytes.subarray(7));
  } catch {
    return { error: "name is not valid UTF-8" };
  }
  return { loadout: pairs as unknown as SideLoadout, name };
}

/** JSON export form. `id` + `createdAt` are intentionally omitted so re-
 *  imports get fresh IDs. */
export function encodeJson(row: { name: string; loadout: SideLoadout }): string {
  return JSON.stringify({ name: row.name, loadout: row.loadout });
}

/** Try to parse `raw` as either a `L1:` share code or a JSON export.
 *  Returns `{ loadout, name }` on success or `{ error }` on failure. */
export function parseImport(raw: string): CodecResult {
  const trimmed = raw.trim();
  if (trimmed.startsWith(SHARE_PREFIX)) {
    return decodeShareCode(trimmed);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch (e) {
    return { error: `not a share code and not valid JSON: ${(e as Error).message}` };
  }
  if (!parsed || typeof parsed !== "object") {
    return { error: "JSON payload is not an object" };
  }
  const obj = parsed as { name?: unknown; loadout?: unknown };
  if (typeof obj.name !== "string") {
    return { error: "JSON payload missing string `name`" };
  }
  if (!Array.isArray(obj.loadout) || obj.loadout.length !== PIECE_COUNT) {
    return { error: `JSON \`loadout\` must be an array of ${PIECE_COUNT} pairs` };
  }
  for (let i = 0; i < PIECE_COUNT; i++) {
    const p = obj.loadout[i];
    if (!Array.isArray(p) || p.length !== SLOTS_PER_PIECE) {
      return { error: `JSON \`loadout[${i}]\` must be a pair` };
    }
    for (let s = 0; s < SLOTS_PER_PIECE; s++) {
      const v = p[s];
      if (typeof v !== "number" || !Number.isInteger(v) || v < 0 || v > 15) {
        return { error: `JSON \`loadout[${i}][${s}]\` must be an integer 0..15` };
      }
    }
  }
  return { loadout: obj.loadout as unknown as SideLoadout, name: obj.name };
}

// --- base64url helpers -----------------------------------------------------
//
// Node/browsers both expose `btoa`/`atob`, but they work on binary strings
// and use the standard alphabet with `=` padding. Share codes are meant to
// go into URLs and chat clients, so use base64url (- and _, no padding).

function toBase64Url(bytes: Uint8Array): string {
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  const std = btoa(bin);
  return std.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function fromBase64Url(s: string): Uint8Array {
  let std = s.replace(/-/g, "+").replace(/_/g, "/");
  while (std.length % 4 !== 0) std += "=";
  const bin = atob(std);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}
