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

  // ---- Verify the banner shows the in-game date and that it advances ----
  await page.waitForSelector(".date-value", { timeout: 5000 });
  const dateText = (await page.locator(".date-value").textContent())?.trim();
  log("date text:", dateText);
  const parse = (t) => {
    const m = t?.match(/^(\d{4})\/(\d{2})\/(\d{2})$/);
    return m ? { year: +m[1], month: +m[2], day: +m[3] } : null;
  };
  const first = parse(dateText);
  if (!first) fail(`date banner did not match 'YYYY/MM/DD', got '${dateText}'`);
  if (first.year === 0 || first.month < 1 || first.day < 1) {
    fail(`date banner has uninitialised values: ${JSON.stringify(first)}`);
  }

  // Backend ticks once per second, so the day must change within ~4s.
  const changed = await page
    .waitForFunction(
      (before) => document.querySelector(".date-value")?.textContent?.trim() !== before,
      dateText,
      { timeout: 4000 },
    )
    .then(() => true)
    .catch(() => false);
  if (!changed) fail(`date did not advance within 4s (stuck at '${dateText}')`);
  const after = parse((await page.locator(".date-value").textContent())?.trim());
  log("date advanced:", JSON.stringify(first), "->", JSON.stringify(after));

  await page.screenshot({ path: "web/e2e/04-date.png", fullPage: true });
  log("screenshot saved: web/e2e/04-date.png");

  // ---- Spacebar pauses the tick; the badge appears and the date freezes ----
  log("pressing space to pause");
  await page.keyboard.press("Space");
  await page.waitForSelector(".badge", { timeout: 3000 });

  const paused = (await page.locator(".date-value").textContent())?.trim();
  await page.waitForTimeout(3000);
  const stillPaused = (await page.locator(".date-value").textContent())?.trim();
  log("paused at:", paused, "after 3s:", stillPaused);
  if (paused !== stillPaused) fail(`date advanced while paused: ${paused} -> ${stillPaused}`);

  // The backend must agree, not just the UI.
  const apiPaused = await page.evaluate(async () => {
    const r = await fetch("/api/v1/date");
    return (await r.json()).data.is_paused;
  });
  if (apiPaused !== true) fail(`GET /api/v1/date reported is_paused=${apiPaused}, expected true`);

  await page.screenshot({ path: "web/e2e/05-paused.png", fullPage: true });
  log("screenshot saved: web/e2e/05-paused.png");

  // ---- Spacebar again resumes ----
  log("pressing space to resume");
  await page.keyboard.press("Space");
  await page.waitForSelector(".badge", { state: "detached", timeout: 3000 });
  const resumed = await page
    .waitForFunction(
      (before) => document.querySelector(".date-value")?.textContent?.trim() !== before,
      stillPaused,
      { timeout: 4000 },
    )
    .then(() => true)
    .catch(() => false);
  if (!resumed) fail(`date did not resume within 4s (stuck at '${stillPaused}')`);
  log("resumed at:", (await page.locator(".date-value").textContent())?.trim());

  // ---- Click Goldharbour (has a settlement, pop 100) ----
  log("clicking Goldharbour");
  await page.getByRole("button", { name: "Goldharbour" }).click();

  await page.waitForSelector("aside.panel", { timeout: 3000 });
  const heading = await page.locator("aside.panel h2").textContent();
  if (heading?.trim() !== "Goldharbour") fail(`expected panel heading 'Goldharbour', got '${heading}'`);

  // Settlement block should list 5 inventories, 9 populations, 5 workplaces.
  const invRows = await page.locator("table.inventories tbody tr").count();
  const popRows = await page.locator("ul.populations li").count();
  const workRows = await page.locator("ul.workplaces li").count();
  log(`inventories=${invRows} populations=${popRows} workplaces=${workRows}`);
  if (invRows !== 5) fail(`expected 5 inventory rows, got ${invRows}`);
  if (popRows !== 9) fail(`expected 9 population rows, got ${popRows}`);
  if (workRows !== 5) fail(`expected 5 workplace rows, got ${workRows}`);

  // Spot-check the bakery's two input rows are present.
  const invResources = await page.locator("table.inventories tbody tr td:first-child code").allTextContents();
  if (!invResources.includes("resource-wheat") || !invResources.includes("resource-wood") ||
      !invResources.includes("resource-grain") || !invResources.includes("resource-bread") ||
      !invResources.includes("resource-ale")) {
    fail(`expected all 5 resource ids in inventory table, got ${JSON.stringify(invResources)}`);
  }

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

  const muted = await page.locator("aside.panel .muted").textContent();
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
