<script lang="ts">
  import { lands, settlements, selectedLandId } from "./store";
  import { navigate } from "svelte-routing";

  let land = $derived($lands.find((l) => l.id === $selectedLandId) ?? null);
  let settlement = $derived(
    land ? $settlements.find((s) => s.land_id === land!.id) ?? null : null,
  );

  function close() {
    selectedLandId.set(null);
    navigate("/");
  }

  function fmtCoord([x, y]: [number, number]): string {
    return `${x.toFixed(2)}, ${y.toFixed(2)}`;
  }
</script>

{#if land}
  <aside class="panel">
    <header>
      <h2>{land.name}</h2>
      <button class="close" onclick={close} aria-label="Close details">×</button>
    </header>

    <dl>
      <dt>ID</dt>
      <dd><code>{land.id}</code></dd>

      <dt>Terrain</dt>
      <dd class="terrain terrain-{land.terrain}">{land.terrain}</dd>

      <dt>Holding</dt>
      <dd>({fmtCoord(land.holding)})</dd>

      <dt>Border vertices</dt>
      <dd>{land.borders.length}</dd>
    </dl>

    <section class="settlement">
      <h3>Settlement</h3>
      {#if settlement}
        <dl>
          <dt>ID</dt>
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
          <li>{i + 1}. ({x.toFixed(2)}, {y.toFixed(2)})</li>
        {/each}
      </ol>
    </section>
  </aside>
{/if}

<style>
  .panel {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(360px, 38vw);
    padding: 1.5rem 1.25rem 1.25rem;
    background:
      linear-gradient(180deg, var(--paper-bg), var(--paper-bg-deep));
    color: var(--ink);
    border-left: 2px solid var(--ink);
    box-shadow: -6px 0 14px rgba(61, 40, 23, 0.18);
    overflow-y: auto;
    font-family: var(--font-body);
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    border-bottom: 1px solid var(--ink-soft);
    padding-bottom: 0.5rem;
    margin-bottom: 1rem;
  }
  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.8rem;
    letter-spacing: 0.02em;
  }
  .close {
    background: none;
    border: none;
    color: var(--ink);
    font-size: 1.8rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 0.25rem;
  }
  .close:hover { color: var(--accent); }

  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    column-gap: 0.75rem;
    row-gap: 0.4rem;
    margin: 0 0 1.25rem;
  }
  dt {
    font-family: var(--font-smallcaps);
    color: var(--ink-soft);
    letter-spacing: 0.04em;
  }
  dd {
    margin: 0;
    font-family: var(--font-body);
  }
  code {
    font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
    font-size: 0.9em;
    background: rgba(61, 40, 23, 0.08);
    padding: 0.05em 0.35em;
    border-radius: 3px;
  }

  .terrain {
    font-family: var(--font-smallcaps);
    letter-spacing: 0.05em;
  }
  .terrain-plains  { color: #4a5a25; }
  .terrain-wetlands { color: #2f4a5a; }
  .terrain-forest  { color: #2f3d1c; }

  section h3 {
    font-family: var(--font-display);
    font-size: 1.1rem;
    margin: 1rem 0 0.5rem;
    border-bottom: 1px solid var(--ink-faint);
    padding-bottom: 0.25rem;
  }
  .borders ol {
    margin: 0;
    padding-left: 1.25rem;
    font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
    font-size: 0.85rem;
    line-height: 1.45;
    color: var(--ink-soft);
  }
  .muted {
    color: var(--ink-soft);
    font-style: italic;
    margin: 0;
  }

  .children-title {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin: 0.9rem 0 0.35rem;
    padding-bottom: 0.2rem;
    border-bottom: 1px dashed var(--ink-faint);
    font-family: var(--font-smallcaps);
    font-size: 0.95rem;
    letter-spacing: 0.06em;
    color: var(--ink-soft);
  }
  .count {
    font-family: var(--font-body);
    font-size: 0.85rem;
    color: var(--ink-faint);
  }

  table.children {
    width: 100%;
    border-collapse: collapse;
    margin: 0 0 0.5rem;
    font-size: 0.9rem;
  }
  table.children th,
  table.children td {
    text-align: left;
    padding: 0.2rem 0.3rem;
    border-bottom: 1px solid rgba(61, 40, 23, 0.12);
  }
  table.children th {
    font-family: var(--font-smallcaps);
    color: var(--ink-soft);
    font-weight: normal;
    letter-spacing: 0.04em;
    border-bottom-color: var(--ink-soft);
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
    padding: 0.25rem 0;
    border-bottom: 1px solid rgba(61, 40, 23, 0.12);
    font-size: 0.9rem;
  }
  ul.children li:last-child { border-bottom: none; }
  .muted-inline {
    color: var(--ink-soft);
  }
  .refs {
    margin-top: 0.15rem;
    color: var(--ink-soft);
    font-size: 0.8rem;
    line-height: 1.4;
  }
</style>
