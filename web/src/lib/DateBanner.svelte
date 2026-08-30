<script lang="ts">
  import { onMount } from "svelte";
  import { fetchDate } from "./api";
  import { date } from "./store";

  // Poll interval in ms. Matches the backend tick (TICK_PER_SECONDS).
  const POLL_MS = 1000;

  // Zero-pad to `width` digits so the banner never changes width.
  const pad = (n: number, width = 2) => String(n).padStart(width, "0");

  let formatted = $derived(
    $date ? `${pad($date.year, 4)}/${pad($date.month)}/${pad($date.day)}` : null,
  );

  onMount(() => {
    let stopped = false;

    async function refresh() {
      try {
        const d = await fetchDate();
        if (!stopped) date.set(d);
      } catch {
        // Transient failures are ignored; the next poll retries.
      }
    }

    refresh();
    const timer = setInterval(refresh, POLL_MS);
    return () => {
      stopped = true;
      clearInterval(timer);
    };
  });
</script>

{#if formatted}
  <span class="game-date">{formatted}</span>
{:else}
  <span class="game-date pending">…</span>
{/if}

<style>
  .game-date {
    font-family: var(--font-smallcaps);
    letter-spacing: 0.08em;
    font-size: 0.9rem;
    color: var(--ink-soft);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .pending {
    opacity: 0.5;
  }
</style>
