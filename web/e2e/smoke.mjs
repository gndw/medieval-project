// Smoke test: boots Chromium against the running backend on :7777,
// verifies the map renders, clicks a land, and checks the detail panel
// shows the correct settlement info.
//
// Usage:
//   1. cargo run      (in another terminal, on 127.0.0.1:7777)
//   2. node web/e2e/smoke.mjs

import { chromium } from "playwright";
import { writeFileSync } from "node:fs";

const BASE = "http://127.0.0.1:7777";

function log(...a) { console.log("[smoke]", ...a); }
function fail(msg) { console.error("[smoke] FAIL:", msg); process.exit(1); }

const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
const page = await ctx.newPage();

page.on("pageerror", (e) => console.error("[page error]", e.message));
page.on("console", (m) => {
  if (m.type() === "error") console.error("[console error]", m.text());
});

try {
  log("loading", BASE);
  await page.goto(BASE, { waitUntil: "networkidle" });

  // ---- Verify map rendered: 4 land polygons + 3 road polylines ----
  await page.waitForSelector("svg.map", { timeout: 5000 });
  const landCount  = await page.locator(".land polygon").count();
  const roadCount  = await page.locator(".road").count();
  log(`land polygons: ${landCount}, road polylines: ${roadCount}`);
  if (landCount !== 4) fail(`expected 4 land polygons, got ${landCount}`);
  if (roadCount !== 3) fail(`expected 3 road polylines, got ${roadCount}`);

  await page.screenshot({ path: "web/e2e/01-map.png", fullPage: true });
  log("screenshot saved: web/e2e/01-map.png");

  // ---- Click Goldharbour (has a settlement, pop 100) ----
  log("clicking Goldharbour");
  await page.getByRole("button", { name: "Goldharbour" }).click();

  await page.waitForSelector("aside.panel", { timeout: 3000 });
  const heading = await page.locator("aside.panel h2").textContent();
  if (heading?.trim() !== "Goldharbour") fail(`expected panel heading 'Goldharbour', got '${heading}'`);

  // Settlement block should show population 100.
  const popText = await page.locator(".population").textContent();
  log("population text:", popText);
  if (!popText?.includes("100")) fail(`expected population to include '100', got '${popText}'`);

  await page.screenshot({ path: "web/e2e/02-goldharbour.png", fullPage: true });
  log("screenshot saved: web/e2e/02-goldharbour.png");

  // URL should carry ?selected-land-id=land-1
  const url = new URL(page.url());
  const sel = url.searchParams.get("selected-land-id");
  if (sel !== "land-1") fail(`expected selected-land-id=land-1, got '${sel}' (url: ${url})`);
  log("URL:", url.toString());

  // ---- Click Hawkrest (no settlement) ----
  // Hawkrest sits on the right side of the map, partially under the panel,
  // so close the panel first (realistic UX), then click.
  log("closing panel");
  await page.locator("aside.panel .close").click();
  await page.waitForSelector("aside.panel", { state: "detached", timeout: 3000 });

  log("clicking Hawkrest");
  await page.getByRole("button", { name: "Hawkrest" }).click();
  await page.waitForFunction(() => {
    const h = document.querySelector("aside.panel h2");
    return h && h.textContent?.trim() === "Hawkrest";
  }, { timeout: 3000 });

  const muted = await page.locator(".muted").textContent();
  log("no-settlement text:", muted);
  if (!muted?.toLowerCase().includes("no settlement")) {
    fail(`expected 'No settlement' message for Hawkrest, got '${muted}'`);
  }

  await page.screenshot({ path: "web/e2e/03-hawkrest.png", fullPage: true });
  log("screenshot saved: web/e2e/03-hawkrest.png");

  log("ALL CHECKS PASSED");
} catch (e) {
  await page.screenshot({ path: "web/e2e/error.png", fullPage: true });
  console.error("[smoke] exception:", e);
  process.exit(1);
} finally {
  await browser.close();
}
