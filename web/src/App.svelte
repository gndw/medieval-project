<script lang="ts">
  import { Router, Route } from "svelte-routing";
  import MapView from "./lib/MapView.svelte";
  import LandDetail from "./lib/LandDetail.svelte";
  import { selectedLandId } from "./lib/store";
  import { onMount } from "svelte";

  // Sync URL → store on initial load so deep links like /lands/land-1
  // restore the selection after the SPA boots.
  onMount(() => {
    const m = window.location.pathname.match(/^\/lands\/([^/]+)/);
    if (m) {
      selectedLandId.set(decodeURIComponent(m[1]));
    } else {
      selectedLandId.set(null);
    }
  });

  // Keep the URL in sync when the store changes via in-app navigation.
  $effect(() => {
    // no-op: store updates are already driven by navigate() in MapView/LandDetail.
    // This effect exists to make Svelte track the dependency.
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
      <Route path="/lands/:id"><MapView /></Route>
    </div>

    {#if $selectedLandId}
      <Route path="/lands/:id"><LandDetail /></Route>
    {/if}
  </main>
</Router>

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
