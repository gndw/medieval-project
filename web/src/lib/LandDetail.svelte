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
      <div class="header-text">
        <span class="eyebrow">land</span>
        <h2>{land.name}</h2>
      </div>
      <button class="close" onclick={close} aria-label="Close details">×</button>
    </header>

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
  </aside>
{/if}

<style>
  .panel {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(360px, 38vw);
    padding: 1.25rem 1.25rem 1.25rem;
    background: var(--surface);
    color: var(--ink);
    border-left: 1px solid var(--ink-faint);
    overflow-y: auto;
    font-family: var(--font-body);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    border-bottom: 1px solid var(--ink);
    padding-bottom: 0.6rem;
    margin-bottom: 1rem;
  }
  .header-text {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
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
    font-size: 1.5rem;
    font-weight: 400;
    letter-spacing: 0.02em;
    line-height: 1.1;
    color: var(--ink);
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

  .kv {
    display: grid;
    grid-template-columns: max-content 1fr;
    column-gap: 0.85rem;
    row-gap: 0.45rem;
    margin: 0 0 1.25rem;
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
    margin: 1.25rem 0 0.5rem;
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
