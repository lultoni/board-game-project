// Dev-only UI verification via Playwright. NOT part of the build or release.
// Run with the dev server up on :5173:  node tests-e2e/verify-backbtn.mjs
import { chromium } from "playwright";

const BASE = "http://localhost:5173";
const results = [];
function check(name, cond, detail = "") {
  results.push({ name, ok: !!cond, detail });
  console.log(`${cond ? "PASS" : "FAIL"}  ${name}${detail ? "  — " + detail : ""}`);
}

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));

async function pills() {
  return {
    back: await page.locator("a.back-btn").count(),
    help: await page.locator("button.help-btn").count(),
    gear: await page.locator("button.gear-btn").count(),
  };
}

// 1. HUB: no back button, but help + gear present.
await page.goto(BASE + "/", { waitUntil: "networkidle" });
await page.waitForTimeout(300);
let p = await pills();
check("hub: no back button", p.back === 0, `back=${p.back}`);
check("hub: help+gear present", p.help === 1 && p.gear === 1, `help=${p.help} gear=${p.gear}`);

// 2. SETUP: all three pills present.
await page.goto(BASE + "/setup/", { waitUntil: "networkidle" });
await page.waitForTimeout(300);
p = await pills();
check("setup: back present", p.back === 1, `back=${p.back}`);
check("setup: help+gear present", p.help === 1 && p.gear === 1);

// 2b. back button is actually top-left (x small, y small) and left of help/gear.
if (p.back === 1) {
  const bb = await page.locator("a.back-btn").boundingBox();
  const gb = await page.locator("button.gear-btn").boundingBox();
  check("setup: back is top-left", bb && bb.x < 120 && bb.y < 60, bb ? `x=${Math.round(bb.x)} y=${Math.round(bb.y)}` : "no box");
  check("setup: back left of gear", bb && gb && bb.x < gb.x, bb && gb ? `back.x=${Math.round(bb.x)} gear.x=${Math.round(gb.x)}` : "");
  // title not hidden under the pill: the h1 should start to the right of the pill OR below it.
  const h1 = await page.locator("main header h1").first().boundingBox();
  check("setup: title clear of back pill", h1 && bb && (h1.x >= bb.x + bb.width - 2 || h1.y >= bb.y + bb.height - 2),
    h1 && bb ? `h1.x=${Math.round(h1.x)} h1.y=${Math.round(h1.y)} pill right=${Math.round(bb.x + bb.width)}` : "");
}

// 3. SETUP back click → hub.
await page.locator("a.back-btn").click();
await page.waitForTimeout(400);
check("setup back → hub", new URL(page.url()).pathname === "/", `landed=${new URL(page.url()).pathname}`);

// 4. REPLAY back → library (context override).
await page.goto(BASE + "/replay/", { waitUntil: "networkidle" });
await page.waitForTimeout(400);
p = await pills();
check("replay: back present", p.back === 1);
if (p.back === 1) {
  await page.locator("a.back-btn").click();
  await page.waitForTimeout(500);
  check("replay back → library", new URL(page.url()).pathname === "/library/", `landed=${new URL(page.url()).pathname}`);
}

// 5. LIBRARY back → hub (default).
await page.goto(BASE + "/library/", { waitUntil: "networkidle" });
await page.waitForTimeout(400);
await page.locator("a.back-btn").click();
await page.waitForTimeout(400);
check("library back → hub", new URL(page.url()).pathname === "/", `landed=${new URL(page.url()).pathname}`);

// 6. STALE-OVERRIDE REGRESSION: visit replay (sets →library), then draft
//    (sets no override). Draft's back must default to hub, NOT inherit library.
await page.goto(BASE + "/replay/", { waitUntil: "networkidle" });
await page.waitForTimeout(300);
await page.goto(BASE + "/draft/", { waitUntil: "networkidle" });
await page.waitForTimeout(300);
if (await page.locator("a.back-btn").count()) {
  await page.locator("a.back-btn").click();
  await page.waitForTimeout(400);
  check("draft after replay → hub (no stale override)", new URL(page.url()).pathname === "/", `landed=${new URL(page.url()).pathname}`);
} else {
  check("draft: back present", false, "no back button on draft");
}

check("no page errors", errors.length === 0, errors.slice(0, 3).join(" | "));

await browser.close();
const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 1 : 0);
