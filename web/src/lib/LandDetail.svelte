<script lang="ts">
  import { lands, selectedLandId } from "./store";
  import { navigate } from "svelte-routing";

  let land = $derived($lands.find((l) => l.id === $selectedLandId) ?? null);

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

  .borders h3 {
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
</style>
