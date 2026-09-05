<script lang="ts">
  import { lands, settlements, selectedLandId } from "./store";
  import { navigate } from "svelte-routing";

  let land = $derived($lands.find((l) => l.id === $selectedLandId) ?? null);
  let settlement = $derived(
    land ? $settlements.find((s) => s.land_id === land!.id) ?? null : null,
  );

  // Drag offset relative to the CSS-anchored position (top:24px, right:24px).
  let dx = $state(0);
  let dy = $state(0);
  let dragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let startDx = 0;
  let startDy = 0;
  let panelEl: HTMLElement | undefined = $state();

  // Clamp the panel so it cannot leave the visible map area.
  function clamp(nx: number, ny: number): { x: number; y: number } {
    const parent = panelEl?.offsetParent as HTMLElement | null;
    if (!parent || !panelEl) return { x: nx, y: ny };
    const pw = parent.clientWidth;
    const ph = parent.clientHeight;
    const w = panelEl.offsetWidth;
    const h = panelEl.offsetHeight;
    // Panel is anchored top-right with 24px margin: dx>0 slides right (off-edge),
    // dx<0 slides left toward the map interior. Mirror for the vertical axis.
    const minDx = -(pw - w - 24);
    const maxDx = 24;
    const minDy = 0;
    const maxDy = ph - h - 24;
    return {
      x: Math.max(minDx, Math.min(maxDx, nx)),
      y: Math.max(minDy, Math.min(maxDy, ny)),
    };
  }

  function onHeaderPointerDown(ev: PointerEvent) {
    if (ev.button !== 0) return;
    // Don't start a drag when the close button is the actual target.
    if ((ev.target as HTMLElement).closest(".close")) return;
    dragging = true;
    dragStartX = ev.clientX;
    dragStartY = ev.clientY;
    startDx = dx;
    startDy = dy;
    (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
  }

  function onHeaderPointerMove(ev: PointerEvent) {
    if (!dragging) return;
    const { x, y } = clamp(
      startDx + (ev.clientX - dragStartX),
      startDy + (ev.clientY - dragStartY),
    );
    dx = x;
    dy = y;
  }

  function onHeaderPointerUp(ev: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    try {
      (ev.currentTarget as HTMLElement).releasePointerCapture(ev.pointerId);
    } catch {
      // Pointer was already released; nothing to do.
    }
  }

  function close() {
    selectedLandId.set(null);
    navigate("/");
  }

  function fmtCoord([x, y]: [number, number]): string {
    return `${x.toFixed(2)}, ${y.toFixed(2)}`;
  }
</script>

{#if land}
  <div
    bind:this={panelEl}
    class="panel"
    class:dragging
    style="transform: translate({dx}px, {dy}px)"
    role="dialog"
    aria-label="Land details for {land.name}"
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header
      onpointerdown={onHeaderPointerDown}
      onpointermove={onHeaderPointerMove}
      onpointerup={onHeaderPointerUp}
      onpointercancel={onHeaderPointerUp}
    >
      <span class="grip" aria-hidden="true">
        <span></span>
        <span></span>
        <span></span>
      </span>
      <div class="header-text">
        <span class="eyebrow">land</span>
        <h2>{land.name}</h2>
      </div>
      <button class="close" onclick={close} aria-label="Close details">×</button>
    </header>

    <div class="body">
      <dl class="kv">
        <dt>id</dt>
        <dd><code>{land.id}</code></dd>

        <dt>terrain</dt>
        <dd><span class="terrain terrain-{land.terrain}">{land.terrain}</span></dd>

        <dt>holding</dt>
        <dd>({fmtCoord(land.holding)})</dd>

        <dt>border vertices</dt>
        <dd>{land.borders.length}</dd>
      </dl>

      <section class="settlement">
        <h3>Settlement</h3>
        {#if settlement}
          <dl class="kv">
            <dt>id</dt>
            <dd><code>{settlement.id}</code></dd>
          </dl>

          {#if settlement.inventories.length > 0}
            <h4 class="children-title">
              Inventories
              <span class="count">{settlement.inventories.length}</span>
            </h4>
            <table class="children inventories">
              <thead>
                <tr><th>Resource</th><th class="num">Quantity</th></tr>
              </thead>
              <tbody>
                {#each settlement.inventories as inv (inv.id)}
                  <tr>
                    <td><code>{inv.resource_id}</code></td>
                    <td class="num">{inv.quantity.toLocaleString()}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}

          {#if settlement.populations.length > 0}
            <h4 class="children-title">
              Populations
              <span class="count">{settlement.populations.length}</span>
            </h4>
            <ul class="children populations">
              {#each settlement.populations as pop (pop.id)}
                <li>
                  <code>{pop.id}</code>
                  <span class="muted-inline">— {pop.profession_id}</span>
                </li>
              {/each}
            </ul>
          {/if}

          {#if settlement.workplaces.length > 0}
            <h4 class="children-title">
              Workplaces
              <span class="count">{settlement.workplaces.length}</span>
            </h4>
            <ul class="children workplaces">
              {#each settlement.workplaces as work (work.id)}
                <li>
                  <code>{work.id}</code>
                  <div class="refs">
                    production: <code>{work.production_id}</code><br />
                    staffed by: <code>{work.population_id}</code>
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        {:else}
          <p class="muted">No settlement inhabits these lands.</p>
        {/if}
      </section>

      <section class="borders">
        <h3>Borders</h3>
        <ol>
          {#each land.borders as [x, y], i}
            <li>{String(i + 1).padStart(2, "0")} ({x.toFixed(2)}, {y.toFixed(2)})</li>
          {/each}
        </ol>
      </section>
    </div>
  </div>
{/if}

<style>
  /* Floating "folio" card. Anchored top-right by default; transform offsets it. */
  .panel {
    position: absolute;
    top: 24px;
    right: 24px;
    width: min(360px, calc(100% - 48px));
    max-height: calc(100% - 48px);
    display: flex;
    flex-direction: column;
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--ink);
    box-shadow:
      0 1px 0 rgba(0, 0, 0, 0.04),
      0 8px 24px rgba(0, 0, 0, 0.14);
    font-family: var(--font-body);
    z-index: 10;
    transition: box-shadow 120ms ease;
  }
  .panel.dragging {
    box-shadow:
      0 1px 0 rgba(0, 0, 0, 0.06),
      0 16px 36px rgba(0, 0, 0, 0.22);
  }

  /* Header is the drag handle: cursor + touch-action prevent native scroll. */
  header {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 0.7rem;
    padding: 0.65rem 0.75rem;
    background: var(--surface-deep);
    border-bottom: 1px solid var(--ink);
    cursor: grab;
    user-select: none;
    touch-action: none;
  }
  .dragging header,
  header:active {
    cursor: grabbing;
  }

  .grip {
    display: inline-flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
    width: 14px;
    height: 18px;
    padding: 3px 0;
  }
  .grip span {
    display: block;
    height: 1px;
    background: var(--ink-mid);
  }
  .dragging .grip span {
    background: var(--accent);
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }
  .eyebrow {
    font-family: var(--font-body);
    font-size: 0.7rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-mid);
  }
  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.35rem;
    font-weight: 400;
    letter-spacing: 0.02em;
    line-height: 1.1;
    color: var(--ink);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .close {
    background: transparent;
    border: 1px solid var(--ink-faint);
    border-radius: 2px;
    color: var(--ink);
    font-family: var(--font-body);
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    padding: 0.2rem 0.45rem;
    transition: border-color 120ms ease, color 120ms ease;
  }
  .close:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .close:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .body {
    flex: 1 1 auto;
    overflow-y: auto;
    padding: 1rem 1.1rem 1.25rem;
  }

  .kv {
    display: grid;
    grid-template-columns: max-content 1fr;
    column-gap: 0.85rem;
    row-gap: 0.45rem;
    margin: 0 0 1.1rem;
  }
  .kv dt {
    font-family: var(--font-body);
    color: var(--ink-mid);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    align-self: center;
  }
  .kv dd {
    margin: 0;
    font-family: var(--font-body);
    font-size: 0.85rem;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  code {
    font-family: var(--font-body);
    font-size: 0.82em;
    background: var(--surface-deep);
    padding: 0.05em 0.4em;
    border: 1px solid var(--ink-faint);
    border-radius: 2px;
  }

  .terrain {
    font-family: var(--font-body);
    font-size: 0.78rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .terrain-plains  { color: #4a5a25; }
  .terrain-wetlands { color: #2f5a52; }
  .terrain-forest  { color: #2f3d1c; }

  section h3 {
    font-family: var(--font-body);
    font-size: 0.78rem;
    font-weight: 500;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-mid);
    margin: 1.1rem 0 0.5rem;
    padding-bottom: 0.3rem;
    border-bottom: 1px solid var(--ink-faint);
  }
  .borders ol {
    margin: 0;
    padding-left: 0;
    font-family: var(--font-body);
    font-size: 0.78rem;
    line-height: 1.55;
    color: var(--ink-mid);
    list-style: none;
    columns: 2;
    column-gap: 1rem;
  }
  .borders li {
    break-inside: avoid;
  }
  .muted {
    color: var(--ink-mid);
    margin: 0;
    font-style: italic;
  }

  .children-title {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin: 1rem 0 0.4rem;
    padding-bottom: 0.25rem;
    border-bottom: 1px dashed var(--ink-faint);
    font-family: var(--font-body);
    font-size: 0.72rem;
    font-weight: 500;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-mid);
  }
  .count {
    font-family: var(--font-body);
    font-size: 0.7rem;
    color: var(--ink-faint);
  }

  table.children {
    width: 100%;
    border-collapse: collapse;
    margin: 0 0 0.5rem;
    font-size: 0.82rem;
  }
  table.children th,
  table.children td {
    text-align: left;
    padding: 0.25rem 0.35rem;
    border-bottom: 1px solid var(--ink-faint);
  }
  table.children th {
    font-family: var(--font-body);
    color: var(--ink-mid);
    font-weight: 500;
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    border-bottom-color: var(--ink-mid);
  }
  table.children td.num,
  table.children th.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  ul.children {
    list-style: none;
    margin: 0 0 0.5rem;
    padding: 0;
  }
  ul.children li {
    padding: 0.3rem 0;
    border-bottom: 1px solid var(--ink-faint);
    font-size: 0.82rem;
  }
  ul.children li:last-child { border-bottom: none; }
  .muted-inline {
    color: var(--ink-mid);
  }
  .refs {
    margin-top: 0.15rem;
    color: var(--ink-mid);
    font-size: 0.78rem;
    line-height: 1.4;
  }
</style>
