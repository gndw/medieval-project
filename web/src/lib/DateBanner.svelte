<script lang="ts">
  import { onMount } from "svelte";
  import { fetchDate, setPause } from "./api";
  import { date, isPaused } from "./store";

  // Poll interval in ms. Matches the backend tick (TICK_PER_SECONDS).
  const POLL_MS = 1000;

  // Zero-pad to `width` digits so the banner never changes width.
  const pad = (n: number, width = 2) => String(n).padStart(width, "0");

  let formatted = $derived(
    $date ? `${pad($date.year, 4)}/${pad($date.month)}/${pad($date.day)}` : null,
  );

  // Set while a POST is in flight so the poll cannot overwrite the
  // optimistic value with a stale read taken before the server applied it.
  let pending = false;

  async function togglePause() {
    const next = !$isPaused;
    isPaused.set(next);
    pending = true;
    try {
      isPaused.set(await setPause(next));
    } catch {
      isPaused.set(!next);
    } finally {
      pending = false;
    }
  }

  onMount(() => {
    let stopped = false;

    async function refresh() {
      try {
        const d = await fetchDate();
        if (stopped) return;
        date.set(d.date);
        if (!pending) isPaused.set(d.is_paused);
      } catch {
        // Transient failures are ignored; the next poll retries.
      }
    }

    function onKeydown(e: KeyboardEvent) {
      if (e.code !== "Space" || e.repeat) return;
      // Ignore the shortcut while a form control or button has focus.
      const el = e.target as HTMLElement | null;
      if (el?.closest("input, textarea, select, button, [contenteditable]")) return;
      e.preventDefault();
      togglePause();
    }

    refresh();
    const timer = setInterval(refresh, POLL_MS);
    window.addEventListener("keydown", onKeydown);
    return () => {
      stopped = true;
      clearInterval(timer);
      window.removeEventListener("keydown", onKeydown);
    };
  });
</script>

{#if formatted}
  <span class="game-date" title="Press space to {$isPaused ? 'resume' : 'pause'}">
    <span class="date-value">{formatted}</span>
    {#if $isPaused}<span class="badge">paused</span>{/if}
  </span>
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
  .badge {
    margin-left: 0.45rem;
    padding: 0.05rem 0.35rem;
    border: 1px solid var(--ink-soft);
    border-radius: 3px;
    font-size: 0.75rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
</style>
