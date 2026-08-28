<script lang="ts">
  import { lands, selectedLandId, loading, error } from "./store";
  import { fetchHome } from "./api";
  import { onMount } from "svelte";
  import { navigate } from "svelte-routing";
  import type { Land, Terrain } from "./types";

  // Viewport state: pan/zoom applied to the inner <g>.
  let scale = $state(1);
  let tx = $state(0);
  let ty = $state(0);

  // Computed bounding box of all lands (in world coordinates).
  let bbox = $derived.by(() => {
    const all = $lands;
    if (all.length === 0) return { minX: 0, minY: 0, maxX: 1, maxY: 1 };
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const l of all) {
      for (const [x, y] of l.borders) {
        if (x < minX) minX = x;
        if (y < minY) minY = y;
        if (x > maxX) maxX = x;
        if (y > maxY) maxY = y;
      }
    }
    // Pad a little.
    const padX = (maxX - minX) * 0.04;
    const padY = (maxY - minY) * 0.04;
    return {
      minX: minX - padX,
      minY: minY - padY,
      maxX: maxX + padX,
      maxY: maxY + padY,
    };
  });

  // SVG viewBox in world coordinates.
  let viewBox = $derived(
    `${bbox.minX} ${bbox.minY} ${bbox.maxX - bbox.minX} ${bbox.maxY - bbox.minY}`,
  );

  // Fill color per terrain.
  function fillFor(t: Terrain): string {
    switch (t) {
      case "plains": return "var(--terrain-plains)";
      case "wetlands": return "var(--terrain-wetlands)";
      case "forest": return "var(--terrain-forest)";
    }
  }

  function pointsAttr(borders: Array<[number, number]>): string {
    return borders.map(([x, y]) => `${x},${y}`).join(" ");
  }

  function selectLand(l: Land, ev: MouseEvent) {
    ev.stopPropagation();
    selectedLandId.set(l.id);
    navigate(`/lands/${encodeURIComponent(l.id)}`);
  }

  function clearSelection() {
    selectedLandId.set(null);
    navigate("/");
  }

  // Pan with mousedown + drag.
  let dragging = $state(false);
  let lastX = 0, lastY = 0;

  function onMouseDown(ev: MouseEvent) {
    if (ev.button !== 0) return;
    dragging = true;
    lastX = ev.clientX;
    lastY = ev.clientY;
  }

  function onMouseMove(ev: MouseEvent) {
    if (!dragging) return;
    const dx = ev.clientX - lastX;
    const dy = ev.clientY - lastY;
    lastX = ev.clientX;
    lastY = ev.clientY;
    // Convert pixel delta into world delta using current viewBox scale.
    const w = bbox.maxX - bbox.minX;
    const h = bbox.maxY - bbox.minY;
    const svg = ev.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const pxToWorldX = w / rect.width;
    const pxToWorldY = h / rect.height;
    tx += dx * pxToWorldX / scale;
    ty += dy * pxToWorldY / scale;
  }

  function onMouseUp() {
    dragging = false;
  }

  // Keyboard: arrows pan, +/- zoom, Escape clears selection.
  function onKeyDown(ev: KeyboardEvent) {
    const w = bbox.maxX - bbox.minX;
    const h = bbox.maxY - bbox.minY;
    const step = 0.08;
    switch (ev.key) {
      case "ArrowLeft":  tx -= w * step / scale; ev.preventDefault(); break;
      case "ArrowRight": tx += w * step / scale; ev.preventDefault(); break;
      case "ArrowUp":    ty -= h * step / scale; ev.preventDefault(); break;
      case "ArrowDown":  ty += h * step / scale; ev.preventDefault(); break;
      case "+": case "=":
        scale = Math.min(20, scale * 1.2); ev.preventDefault(); break;
      case "-": case "_":
        scale = Math.max(0.2, scale / 1.2); ev.preventDefault(); break;
      case "Escape":
        clearSelection(); ev.preventDefault(); break;
    }
  }

  // Wheel zoom anchored at cursor.
  function onWheel(ev: WheelEvent) {
    ev.preventDefault();
    const svg = ev.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const w = bbox.maxX - bbox.minX;
    const h = bbox.maxY - bbox.minY;
    const cursorWorldX = bbox.minX + ((ev.clientX - rect.left) / rect.width) * w;
    const cursorWorldY = bbox.minY + ((ev.clientY - rect.top) / rect.height) * h;

    const factor = ev.deltaY < 0 ? 1.15 : 1 / 1.15;
    const newScale = Math.min(20, Math.max(0.2, scale * factor));

    // Adjust translate so the world point under the cursor stays put.
    const k = newScale / scale;
    tx = cursorWorldX - (cursorWorldX - tx) * k;
    ty = cursorWorldY - (cursorWorldY - ty) * k;
    scale = newScale;
  }

  onMount(async () => {
    try {
      const data = await fetchHome();
      lands.set(data);
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
    } finally {
      loading.set(false);
    }
  });
</script>

{#if $loading}
  <div class="status">Loading the realm…</div>
{:else if $error}
  <div class="status error">Failed to load: {$error}</div>
{:else}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <svg
    class="map"
    {viewBox}
    preserveAspectRatio="xMidYMid meet"
    onmousedown={onMouseDown}
    onmousemove={onMouseMove}
    onmouseup={onMouseUp}
    onmouseleave={onMouseUp}
    onwheel={onWheel}
    onclick={clearSelection}
    onkeydown={onKeyDown}
    role="application"
    tabindex="0"
    aria-label="Map of lands. Use arrow keys to pan, plus and minus to zoom, Escape to deselect."
  >
    <defs>
      <!-- Hand-drawn wobble filter applied to all strokes. -->
      <filter id="wobble" x="-5%" y="-5%" width="110%" height="110%">
        <feTurbulence type="fractalNoise" baseFrequency="0.02" numOctaves="2" seed="3" />
        <feDisplacementMap in="SourceGraphic" scale="3" />
      </filter>

      <!-- Slight inner shadow for parchment depth on fills. -->
      <filter id="parchment-fill">
        <feGaussianBlur in="SourceAlpha" stdDeviation="2" />
        <feOffset dx="1" dy="2" result="off" />
        <feComposite in="off" in2="SourceAlpha" operator="arithmetic" k2="-1" k3="1" result="shadow" />
        <feColorMatrix in="shadow"
          values="0 0 0 0 0.24
                  0 0 0 0 0.16
                  0 0 0 0 0.09
                  0 0 0 0.35 0" />
        <feComposite in2="SourceGraphic" operator="over" />
      </filter>
    </defs>

    <g transform="translate({tx} {ty}) scale({scale})">
      {#each $lands as land (land.id)}
        <g class="land" class:selected={$selectedLandId === land.id}>
          <polygon
            points={pointsAttr(land.borders)}
            fill={fillFor(land.terrain)}
            fill-opacity="0.78"
            stroke="var(--ink)"
            stroke-width="1.5"
            stroke-linejoin="round"
            filter="url(#wobble)"
            onclick={(ev) => selectLand(land, ev)}
            onkeydown={(ev) => { if (ev.key === "Enter") selectLand(land, ev as unknown as MouseEvent); }}
            tabindex="0"
            role="button"
            aria-label={land.name}
          />
          <text
            x={land.holding[0]}
            y={land.holding[1]}
            text-anchor="middle"
            dominant-baseline="middle"
            class="land-label"
            filter="url(#wobble)"
          >
            {land.name}
          </text>
        </g>
      {/each}
    </g>
  </svg>
{/if}

<style>
  .map {
    display: block;
    width: 100%;
    height: 100%;
    cursor: grab;
    background: var(--paper-bg);
  }
  .map:active { cursor: grabbing; }

  .land polygon {
    transition: fill-opacity 120ms ease, stroke-width 120ms ease;
  }
  .land:hover polygon {
    fill-opacity: 0.95;
    stroke-width: 2;
  }
  .land.selected polygon {
    fill-opacity: 1;
    stroke: var(--ink);
    stroke-width: 3;
  }

  .land-label {
    font-family: var(--font-smallcaps);
    font-size: 22px;
    fill: var(--ink);
    pointer-events: none;
    paint-order: stroke;
    stroke: var(--paper-bg);
    stroke-width: 3;
    stroke-linejoin: round;
  }

  .status {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 1.4rem;
    color: var(--ink-soft);
  }
  .status.error { color: var(--accent); }
</style>
