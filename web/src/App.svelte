<script lang="ts">
  import { Router, Route } from "svelte-routing";
  import MapView from "./lib/MapView.svelte";
  import LandDetail from "./lib/LandDetail.svelte";
  import DateBanner from "./lib/DateBanner.svelte";
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
      <div class="brand">
        <span class="brand-mark">▣</span>
        <div class="brand-text">
          <h1>The Realm</h1>
          <span class="subtitle">real-time overview of the realm</span>
        </div>
      </div>
      <DateBanner />
    </header>

    <div class="stage">
      <Route path="/"><MapView /></Route>
    </div>
  </main>
</Router>

<!-- LandDetail is store-driven, not a Route. URL is kept in sync by MapView/LandDetail's navigate() calls; onMount above restores selection on deep link. -->
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
    align-items: center;
    justify-content: space-between;
    gap: 2rem;
    padding: 0.9rem 1.25rem;
    background: var(--surface);
    border-bottom: 1px solid var(--ink-faint);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .brand-mark {
    font-family: var(--font-body);
    color: var(--accent);
    font-size: 1.1rem;
    line-height: 1;
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  .banner h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.15rem;
    font-weight: 400;
    letter-spacing: 0.02em;
    color: var(--ink);
  }
  .subtitle {
    font-family: var(--font-body);
    color: var(--ink-mid);
    font-size: 0.72rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .stage {
    position: relative;
    overflow: hidden;
    min-height: 0;
  }
</style>
