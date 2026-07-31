// Dev-only. Verifies the fixed back/help/gear pills don't overlap the first
// header row on each route (regression #29). Runs against the already-running
// vite dev server (shared with cargo tauri dev). Pure layout check.
import { chromium } from "playwright";
const BASE = "http://localhost:5173";
const results = [];
const check = (n, ok, d = "") => { results.push({ ok: !!ok }); console.log(`${ok ? "PASS" : "FAIL"}  ${n}${d ? "  — " + d : ""}`); };

function overlaps(a, b) {
  if (!a || !b) return false;
  return !(a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y);
}

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1000, height: 800 } });

// Routes that render a header without needing the engine to draw it.
const routes = ["/setup/", "/library/", "/replay/", "/position-builder/", "/draft/", "/multiplayer/"];
for (const r of routes) {
  await page.goto(BASE + r, { waitUntil: "networkidle" });
  await page.waitForTimeout(400);
  const back = await page.locator("a.back-btn").boundingBox().catch(() => null);
  if (!back) { check(`${r} back pill present`, false); continue; }
  // Measure the first CONTENT element inside the header (title/heading), not
  // the header box itself — padding-left pushes content inward, and it's the
  // content that must clear the pill.
  const h1 = await page.locator("main header h1, main header h2, main .header-row h1").first().boundingBox().catch(() => null);
  check(`${r} header content clears back pill`, h1 && !overlaps(back, h1),
    h1 ? `pill.right=${Math.round(back.x + back.width)} content.x=${Math.round(h1.x)}` : "no header content box");
}

await browser.close();
const failed = results.filter((x) => !x.ok).length;
console.log(`\n${results.length - failed}/${results.length} passed`);
process.exit(failed ? 1 : 0);
