<script lang="ts">
  import { Router, Route } from "svelte-routing";
  import MapView from "./lib/MapView.svelte";
  import LandDetail from "./lib/LandDetail.svelte";
  import { selectedLandId } from "./lib/store";
  import { onMount } from "svelte";

  // Sync URL → store on initial load so deep links like /?selected-land-id=land-1
  // restore the selection after the SPA boots.
  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    const id = params.get("selected-land-id");
    if (id) {
      selectedLandId.set(id);
    } else {
      selectedLandId.set(null);
    }
  });

  $effect(() => {
    // Track the store so the panel reactively appears / disappears.
    void $selectedLandId;
  });
</script>

<Router>
  <main class="layout">
    <header class="banner">
      <h1>The Realm</h1>
      <span class="subtitle">A chronicle of these lands</span>
    </header>

    <div class="stage">
      <Route path="/"><MapView /></Route>
    </div>
  </main>
</Router>

<!--
  LandDetail is driven entirely by the store, not by a Route. The URL is
  already kept in sync by navigate() calls in MapView/LandDetail (using the
  ?selected-land-id= query param), and the onMount block above restores the
  selection from the URL on deep link.
-->
{#if $selectedLandId}
  <LandDetail />
{/if}

<style>
  .layout {
    position: relative;
    display: grid;
    grid-template-rows: auto 1fr;
    height: 100%;
  }
  .banner {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    padding: 0.9rem 1.25rem;
    background: linear-gradient(180deg, var(--paper-bg-deep), var(--paper-bg));
    border-bottom: 2px solid var(--ink);
    box-shadow: 0 4px 10px rgba(61, 40, 23, 0.15);
  }
  .banner h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 2rem;
    letter-spacing: 0.04em;
  }
  .subtitle {
    font-family: var(--font-smallcaps);
    color: var(--ink-soft);
    letter-spacing: 0.08em;
    font-size: 0.9rem;
  }
  .stage {
    position: relative;
    overflow: hidden;
    min-height: 0;
  }
</style>
