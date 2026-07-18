import { describe, expect, it } from "vitest";
import { SKILLS } from "$lib/engine";
import { t, setLocale } from "$lib/state/i18n";
import en from "$lib/i18n/en.json";
import de from "$lib/i18n/de.json";

// Guards the i18n keys that HelpModal.svelte resolves. `t()` returns the raw
// dotted key on a miss (English fallback then key), so a typo in a
// `skills.<key>.name` or `help.*` path would silently render the key string
// instead of throwing. These assertions catch that. No DOM / component render
// is involved - the project has no component-render test harness.

// Mirror the key lists HelpModal iterates.
const RULE_KEYS = [
  "goal", "rounds", "move", "moveAttack", "health", "armor",
  "skillPhase", "money", "path", "strikeMove", "bodyguard", "drafting", "progression",
] as const;
const CONTROL_KEYS = ["select", "wheel", "end", "undo", "sandbox"] as const;

function flat(o: unknown, prefix = ""): string[] {
  if (!o || typeof o !== "object") return [];
  return Object.entries(o as Record<string, unknown>).flatMap(([k, v]) =>
    v && typeof v === "object" ? flat(v, `${prefix}${k}.`) : [`${prefix}${k}`],
  );
}

describe("HelpModal i18n content", () => {
  it("resolves name + desc for every skill in en", () => {
    setLocale("en");
    for (const s of Object.values(SKILLS)) {
      const name = t(`skills.${s.key}.name`);
      const desc = t(`skills.${s.key}.desc`);
      expect(name, `skills.${s.key}.name`).not.toBe(`skills.${s.key}.name`);
      expect(desc, `skills.${s.key}.desc`).not.toBe(`skills.${s.key}.desc`);
    }
  });

  it("resolves every help.rules and help.controls term/body in en", () => {
    setLocale("en");
    const keys = [
      "help.title", "help.tabSkills", "help.tabRules", "help.tabControls",
      ...RULE_KEYS.flatMap((k) => [`help.rules.${k}Term`, `help.rules.${k}Body`]),
      ...CONTROL_KEYS.flatMap((k) => [`help.controls.${k}Term`, `help.controls.${k}Body`]),
    ];
    for (const key of keys) {
      expect(t(key), key).not.toBe(key);
    }
  });

  it("has full help.* key parity between en and de", () => {
    const enHelp = flat((en as Record<string, unknown>).help).sort();
    const deHelp = flat((de as Record<string, unknown>).help).sort();
    expect(deHelp).toEqual(enHelp);
  });
});
