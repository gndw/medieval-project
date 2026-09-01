// Smoke test: drives the Tauri app via the WebDriver protocol exposed by
// `tauri-driver` on http://127.0.0.1:4444. Verifies the map renders, clicks
// a land, and checks the detail panel shows the correct settlement info.
// Date advances come through the `date-updated` Tauri event.
//
// Usage:
//   1. Start Vite (separate terminal):
//        npm --prefix web run dev
//   2. Start the Tauri binary (separate terminal):
//        cargo run
//      The binary in dev mode loads from http://localhost:5173.
//   3. Start tauri-driver (separate terminal):
//        tauri-driver --port 4444
//   4. node web/e2e/smoke.mjs
//
// Override the WebDriver URL with TAURI_DRIVER_URL or the binary path with
// TAURI_BINARY.

const BASE = process.env.TAURI_DRIVER_URL ?? "http://127.0.0.1:4444";
const BINARY = process.env.TAURI_BINARY ??
  "C:/Users/gndw/Projects/medieval-project/target/debug/medieval-project.exe";

function log(...a) { console.log("[smoke]", ...a); }
function fail(msg) { console.error("[smoke] FAIL:", msg); process.exit(1); }

async function wd(path, init = {}) {
  const res = await fetch(`${BASE}${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...(init.headers ?? {}) },
  });
  const body = await res.json();
  if (body.value && body.value.error) fail(`${path}: ${body.value.message ?? "unknown error"}`);
  return body.value;
}

// Extract the W3C element id from a `{ "<key>": "<uuid>" }` object.
function eid(el) { return el && (el.ELEMENT ?? Object.values(el)[0]); }

const session = await wd("/session", {
  method: "POST",
  body: JSON.stringify({
    capabilities: {
      alwaysMatch: {
        browserName: "webview2",
        "ms:edgeOptions": { binary: BINARY },
      },
    },
  }),
});
const sid = session.sessionId;
log("session:", sid);

try {
  // Wait until the page has settled (Svelte mounted and `fetchHome` resolved).
  await waitFor(async () => {
    const els = await wd(`/session/${sid}/elements`, {
      method: "POST",
      body: JSON.stringify({ using: "css selector", value: "svg.map" }),
    });
    return Array.isArray(els) && els.length > 0;
  }, 10_000, "svg.map never appeared");

  const lands = await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: ".land polygon" }),
  });
  const roads = await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: ".road" }),
  });
  log(`land polygons: ${lands.length}, road polylines: ${roads.length}`);
  if (lands.length !== 4) fail(`expected 4 land polygons, got ${lands.length}`);
  if (roads.length !== 3) fail(`expected 3 road polylines, got ${roads.length}`);

  await screenshot(sid, "web/e2e/01-map.png");

  // ---- Date banner ----
  await waitFor(async () => {
    const els = await wd(`/session/${sid}/elements`, {
      method: "POST",
      body: JSON.stringify({ using: "css selector", value: ".date-value" }),
    });
    return Array.isArray(els) && els.length > 0;
  }, 5_000, "date banner never appeared");

  const dateEl = eid((await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: ".date-value" }),
  }))[0]);
  const dateText = (await wd(`/session/${sid}/element/${dateEl}/text`)).trim();
  log("date text:", dateText);
  const first = parseDate(dateText);
  if (!first) fail(`date banner did not match 'YYYY/MM/DD', got '${dateText}'`);
  if (first.year === 0 || first.month < 1 || first.day < 1) {
    fail(`date banner has uninitialised values: ${JSON.stringify(first)}`);
  }

  const changed = await waitFor(async () => {
    const t = (await wd(`/session/${sid}/element/${dateEl}/text`)).trim();
    return t !== dateText;
  }, 4_000, null);
  if (!changed) fail(`date did not advance within 4s (stuck at '${dateText}')`);
  const after = parseDate((await wd(`/session/${sid}/element/${dateEl}/text`)).trim());
  log("date advanced:", JSON.stringify(first), "->", JSON.stringify(after));

  await screenshot(sid, "web/e2e/04-date.png");

  // ---- Spacebar pauses ----
  await sendKey(sid, " ");
  await waitFor(async () => {
    const els = await wd(`/session/${sid}/elements`, {
      method: "POST",
      body: JSON.stringify({ using: "css selector", value: ".badge" }),
    });
    return Array.isArray(els) && els.length > 0;
  }, 3_000, "pause badge never appeared");

  const paused = (await wd(`/session/${sid}/element/${dateEl}/text`)).trim();
  await new Promise((r) => setTimeout(r, 3000));
  const stillPaused = (await wd(`/session/${sid}/element/${dateEl}/text`)).trim();
  log("paused at:", paused, "after 3s:", stillPaused);
  if (paused !== stillPaused) fail(`date advanced while paused: ${paused} -> ${stillPaused}`);

  await screenshot(sid, "web/e2e/05-paused.png");

  // ---- Spacebar resumes ----
  await sendKey(sid, " ");
  await waitFor(async () => {
    const els = await wd(`/session/${sid}/elements`, {
      method: "POST",
      body: JSON.stringify({ using: "css selector", value: ".badge" }),
    });
    return Array.isArray(els) && els.length === 0;
  }, 3_000, "pause badge did not disappear");

  const resumed = await waitFor(async () => {
    const t = (await wd(`/session/${sid}/element/${dateEl}/text`)).trim();
    return t !== stillPaused;
  }, 4_000, null);
  if (!resumed) fail(`date did not resume within 4s (stuck at '${stillPaused}')`);
  log("resumed at:", (await wd(`/session/${sid}/element/${dateEl}/text`)).trim());

  // ---- Click Goldharbour ----
  const goldharbourBtn = eid((await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({
      using: "xpath",
      value: "//*[@aria-label='Goldharbour']",
    }),
  }))[0]);
  if (!goldharbourBtn) fail("could not find Goldharbour polygon");
  await wd(`/session/${sid}/element/${goldharbourBtn}/click`, { method: "POST", body: "{}" });

  await waitFor(async () => {
    const els = await wd(`/session/${sid}/elements`, {
      method: "POST",
      body: JSON.stringify({ using: "css selector", value: "aside.panel" }),
    });
    return Array.isArray(els) && els.length > 0;
  }, 3_000, "detail panel never appeared");

  const heading = (await wd(`/session/${sid}/element/${eid((await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: "aside.panel h2" }),
  }))[0])}/text`)).trim();
  if (heading !== "Goldharbour") fail(`expected panel heading 'Goldharbour', got '${heading}'`);

  const invRows = (await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: "table.inventories tbody tr" }),
  })).length;
  const popRows = (await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: "ul.populations li" }),
  })).length;
  const workRows = (await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: "ul.workplaces li" }),
  })).length;
  log(`inventories=${invRows} populations=${popRows} workplaces=${workRows}`);
  if (invRows !== 5) fail(`expected 5 inventory rows, got ${invRows}`);
  if (popRows !== 9) fail(`expected 9 population rows, got ${popRows}`);
  if (workRows !== 5) fail(`expected 5 workplace rows, got ${workRows}`);

  await screenshot(sid, "web/e2e/02-goldharbour.png");

  const url = await wd(`/session/${sid}/url`);
  const sel = new URL(url).searchParams.get("selected-land-id");
  if (sel !== "land-1") fail(`expected selected-land-id=land-1, got '${sel}' (url: ${url})`);
  log("URL:", url);

  // ---- Click Hawkrest (no settlement) ----
  const closeBtn = eid((await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({ using: "css selector", value: "aside.panel .close" }),
  }))[0]);
  await wd(`/session/${sid}/element/${closeBtn}/click`, { method: "POST", body: "{}" });
  await waitFor(async () => {
    const els = await wd(`/session/${sid}/elements`, {
      method: "POST",
      body: JSON.stringify({ using: "css selector", value: "aside.panel" }),
    });
    return Array.isArray(els) && els.length === 0;
  }, 3_000, "panel did not close");

  const hawkrestBtn = eid((await wd(`/session/${sid}/elements`, {
    method: "POST",
    body: JSON.stringify({
      using: "xpath",
      value: "//*[@aria-label='Hawkrest']",
    }),
  }))[0]);
  await wd(`/session/${sid}/element/${hawkrestBtn}/click`, { method: "POST", body: "{}" });
  await waitFor(async () => {
    const txt = await wd(`/session/${sid}/execute/sync`, {
      method: "POST",
      body: JSON.stringify({
        script: "return document.querySelector('aside.panel h2')?.textContent?.trim() ?? ''",
        args: [],
      }),
    });
    return txt === "Hawkrest";
  }, 3_000, "Hawkrest detail never appeared");

  const muted = await wd(`/session/${sid}/execute/sync`, {
    method: "POST",
    body: JSON.stringify({
      script: "return document.querySelector('aside.panel .muted')?.textContent ?? ''",
      args: [],
    }),
  });
  log("no-settlement text:", muted);
  if (!muted?.toLowerCase().includes("no settlement")) {
    fail(`expected 'No settlement' message for Hawkrest, got '${muted}'`);
  }

  await screenshot(sid, "web/e2e/03-hawkrest.png");

  log("ALL CHECKS PASSED");
} catch (e) {
  try { await screenshot(sid, "web/e2e/error.png"); } catch {}
  console.error("[smoke] exception:", e);
  process.exit(1);
} finally {
  await fetch(`${BASE}/session/${sid}`, { method: "DELETE" });
}

// ----- helpers -----------------------------------------------------------

async function waitFor(fn, timeoutMs, errMsg) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try { if (await fn()) return true; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
  if (errMsg) fail(errMsg);
  return false;
}

async function sendKey(sid, key) {
  // W3C actions API: build a key-down + key-up pair for one character.
  await wd(`/session/${sid}/actions`, {
    method: "POST",
    body: JSON.stringify({
      actions: [{
        type: "key",
        id: "kb1",
        actions: [
          { type: "keyDown", value: key },
          { type: "keyUp", value: key },
        ],
      }],
    }),
  });
}

async function screenshot(sid, path) {
  const b64 = await wd(`/session/${sid}/screenshot`);
  const { writeFileSync } = await import("node:fs");
  writeFileSync(path, Buffer.from(b64, "base64"));
  log(`screenshot saved: ${path}`);
}

function parseDate(t) {
  const m = t?.match(/^(\d{4})\/(\d{2})\/(\d{2})$/);
  return m ? { year: +m[1], month: +m[2], day: +m[3] } : null;
}
