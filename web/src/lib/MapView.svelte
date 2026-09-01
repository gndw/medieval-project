<script lang="ts">
  import { lands, roads, settlements, selectedLandId, loading, error } from "./store";
  import { fetchHome } from "./api";
  import { onMount } from "svelte";
  import { navigate } from "svelte-routing";
  import type { Land, Terrain } from "./types";

  // Viewport state: pan/zoom applied to the inner <g>.
  let scale = $state(1);
  let tx = $state(0);
  let ty = $state(0);

  // Computed bounding box of all lands AND roads (in world coordinates).
  let bbox = $derived.by(() => {
    const allLands = $lands;
    const allRoads = $roads;
    if (allLands.length === 0 && allRoads.length === 0) {
      return { minX: 0, minY: 0, maxX: 1, maxY: 1 };
    }
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    const consider = (x: number, y: number) => {
      if (x < minX) minX = x;
      if (y < minY) minY = y;
      if (x > maxX) maxX = x;
      if (y > maxY) maxY = y;
    };
    for (const l of allLands) {
      for (const [x, y] of l.borders) consider(x, y);
    }
    for (const r of allRoads) {
      for (const [x, y] of r.points) consider(x, y);
    }
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

  function pointsAttr(points: Array<[number, number]>): string {
    return points.map(([x, y]) => `${x},${y}`).join(" ");
  }

  function selectLand(l: Land, ev: MouseEvent) {
    ev.stopPropagation();
    selectedLandId.set(l.id);
    navigate(`/?selected-land-id=${encodeURIComponent(l.id)}`);
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

    const k = newScale / scale;
    tx = cursorWorldX - (cursorWorldX - tx) * k;
    ty = cursorWorldY - (cursorWorldY - ty) * k;
    scale = newScale;
  }

  onMount(async () => {
    try {
      const data = await fetchHome();
      lands.set(data.lands);
      roads.set(data.roads);
      settlements.set(data.settlements);
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
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
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
    aria-label="Map of lands and roads. Use arrow keys to pan, plus and minus to zoom, Escape to deselect."
  >
    <defs>
      <!-- Settlement mark: clean monoline castle silhouette.
           Origin (0,0) sits at the ground line beneath the building. -->
      <symbol id="settlement" viewBox="-8 -16 16 16" overflow="visible">
        <g fill="none" stroke="var(--ink)" stroke-width="0.9" stroke-linejoin="miter">
          <rect x="-7" y="-9" width="14" height="9" />
          <rect x="-7" y="-11" width="2" height="2" />
          <rect x="-1" y="-13" width="2" height="4" />
          <rect x="5" y="-11" width="2" height="2" />
          <line x1="0" y1="-13" x2="0" y2="-15" />
          <path d="M 0 -15 L 3.5 -14 L 0 -13 Z" fill="var(--ink)" />
        </g>
      </symbol>
    </defs>

    <g transform="translate({tx} {ty}) scale({scale})">
      <!-- Layer 1: land polygons -->
      <g class="lands">
        {#each $lands as land (land.id)}
          <g class="land" class:selected={$selectedLandId === land.id}>
            <polygon
              points={pointsAttr(land.borders)}
              fill={fillFor(land.terrain)}
              fill-opacity="0.85"
              stroke="var(--ink)"
              stroke-width="1.4"
              stroke-linejoin="miter"
              onclick={(ev) => selectLand(land, ev)}
              onkeydown={(ev) => { if (ev.key === "Enter") selectLand(land, ev as unknown as MouseEvent); }}
              tabindex="0"
              role="button"
              aria-label={land.name}
            />
          </g>
        {/each}
      </g>

      <!-- Layer 2: roads (over land fills, under icons/labels) -->
      <g class="roads">
        {#each $roads as road (road.id)}
          <polyline
            class="road"
            points={pointsAttr(road.points)}
            fill="none"
            stroke="var(--road)"
            stroke-width="1.2"
            stroke-linecap="butt"
            stroke-linejoin="miter"
            stroke-dasharray="4 3"
            pointer-events="none"
          >
            <title>{road.id} ({road.distance_days} days)</title>
          </polyline>
        {/each}
      </g>

      <!-- Layer 3: settlement icons + labels (on top of everything) -->
      <g class="marks">
        {#each $lands as land (land.id)}
          <use
            href="#settlement"
            x={land.holding[0] - 8}
            y={land.holding[1] - 16}
            width="16"
            height="16"
            class="settlement"
            pointer-events="none"
          />
          <text
            x={land.holding[0]}
            y={land.holding[1] - 19}
            text-anchor="middle"
            dominant-baseline="middle"
            class="land-label"
          >
            {land.name}
          </text>
        {/each}
      </g>
    </g>
  </svg>
{/if}

<style>
  .map {
    display: block;
    width: 100%;
    height: 100%;
    cursor: grab;
    background: var(--sea);
  }
  .map:active { cursor: grabbing; }

  .land polygon {
    transition: fill-opacity 120ms ease, stroke-width 120ms ease;
  }
  .land:hover polygon {
    fill-opacity: 1;
  }
  .land.selected polygon {
    fill-opacity: 1;
    stroke: var(--accent);
    stroke-width: 2.4;
  }
  .land polygon:focus-visible {
    outline: none;
    stroke: var(--accent);
    stroke-width: 2.4;
  }

  .road {
    stroke: var(--road);
  }

  .land-label {
    font-family: var(--font-body);
    font-size: 9px;
    font-weight: 500;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    fill: var(--ink);
    pointer-events: none;
  }

  .status {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 0.9rem;
    color: var(--ink-mid);
    letter-spacing: 0.04em;
  }
  .status.error { color: var(--accent); }
</style>
