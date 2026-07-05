// Tests for the loadout dedupe helpers.

import { describe, expect, it } from "vitest";
import type { SideLoadout } from "$lib/engine";
import type { SavedLoadout } from "./types";
import { findDuplicate, loadoutKey } from "./loadout-dedupe";

const A: SideLoadout = [[6, 9], [1, 6], [1, 10], [1, 9], [6, 10], [1, 9]] as const;
const B: SideLoadout = [[2, 6], [1, 8], [2, 9], [1, 14], [2, 6], [1, 8]] as const;
// A with slots swapped on the first piece — different order = different key.
const A_SWAPPED: SideLoadout = [[9, 6], [1, 6], [1, 10], [1, 9], [6, 10], [1, 9]] as const;

function row(id: string, name: string, l: SideLoadout): SavedLoadout {
  return { id, name, loadout: l, createdAt: 0 };
}

describe("loadoutKey", () => {
  it("is stable for identical loadouts", () => {
    expect(loadoutKey(A)).toBe(loadoutKey(A));
  });

  it("differs for different loadouts", () => {
    expect(loadoutKey(A)).not.toBe(loadoutKey(B));
  });

  it("preserves slot order (swap = new key)", () => {
    expect(loadoutKey(A)).not.toBe(loadoutKey(A_SWAPPED));
  });

  it("distinguishes incomplete loadouts (0 slot) from complete", () => {
    const partial: SideLoadout = [[6, 9], [1, 6], [1, 10], [1, 9], [6, 10], [1, 0]] as const;
    expect(loadoutKey(partial)).not.toBe(loadoutKey(A));
  });
});

describe("findDuplicate", () => {
  it("returns null on empty existing list", () => {
    expect(findDuplicate(A, [])).toBeNull();
  });

  it("returns null when no match", () => {
    expect(findDuplicate(A, [row("x", "other", B)])).toBeNull();
  });

  it("matches on skills only, not name", () => {
    const hit = findDuplicate(A, [row("x", "totally different label", A)]);
    expect(hit).not.toBeNull();
    expect(hit!.name).toBe("totally different label");
  });

  it("returns the first match when multiple rows collide", () => {
    const rows: SavedLoadout[] = [
      row("first", "one", A),
      row("second", "two", A),
    ];
    expect(findDuplicate(A, rows)!.id).toBe("first");
  });
});
