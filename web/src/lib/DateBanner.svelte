<script lang="ts">
  import { onMount } from "svelte";
  import { fetchDate, onDateUpdated, setPause } from "./api";
  import { date, isPaused } from "./store";

  // Zero-pad to `width` digits so the banner never changes width.
  const pad = (n: number, width = 2) => String(n).padStart(width, "0");

  let formatted = $derived(
    $date ? `${pad($date.year, 4)}/${pad($date.month)}/${pad($date.day)}` : null,
  );

  // Set while a POST is in flight so the event cannot overwrite the
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

    async function seed() {
      try {
        const d = await fetchDate();
        if (stopped) return;
        date.set(d.date);
        isPaused.set(d.is_paused);
      } catch {
        // Transient failures are ignored; the event stream will catch up.
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

    let unlisten: (() => void) | null = null;
    onDateUpdated((d) => {
      if (stopped) return;
      date.set(d.date);
      if (!pending) isPaused.set(d.is_paused);
    }).then((fn) => {
      if (stopped) fn();
      else unlisten = fn;
    });

    seed();
    window.addEventListener("keydown", onKeydown);
    return () => {
      stopped = true;
      if (unlisten) unlisten();
      window.removeEventListener("keydown", onKeydown);
    };
  });
</script>

<div class="clock" class:paused={$isPaused}>
  <span class="clock-label">date</span>
  {#if formatted}
    <button
      type="button"
      class="date-button"
      class:paused={$isPaused}
      onclick={togglePause}
      title="Press space to {$isPaused ? 'resume' : 'pause'}"
      aria-label={$isPaused ? "Resume simulation" : "Pause simulation"}
    >
      <span class="date-value">{formatted}</span>
    </button>
    <span class="badge" aria-live="polite">
      <span class="dot" aria-hidden="true"></span>
      <span class="badge-text">{$isPaused ? "paused" : "live"}</span>
    </span>
  {:else}
    <span class="date-value pending">—/—/—</span>
    <span class="badge pending"><span class="badge-text">pending</span></span>
  {/if}
</div>

<style>
  .clock {
    display: grid;
    grid-template-columns: auto auto auto;
    align-items: center;
    gap: 0.85rem;
  }
  .clock-label {
    font-family: var(--font-body);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-mid);
  }

  .date-button {
    background: transparent;
    border: none;
    padding: 0;
    margin: 0;
    cursor: pointer;
    font-family: inherit;
    color: inherit;
    line-height: 1;
  }
  .date-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
    border-radius: 2px;
  }
  .date-value {
    font-family: var(--font-body);
    font-size: 1.45rem;
    font-weight: 500;
    color: var(--ink);
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }
  .date-value.pending {
    color: var(--ink-mid);
  }
  .clock.paused .date-value {
    color: var(--accent);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.22rem 0.55rem;
    border: 1px solid var(--ink-faint);
    border-radius: 2px;
    font-family: var(--font-body);
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-mid);
  }
  .badge.pending {
    border-style: dashed;
  }
  .badge-text {
    line-height: 1;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #1f6b3a;
    animation: pulse 1.6s ease-out infinite;
  }
  .clock.paused .badge {
    border-color: var(--accent);
    color: var(--accent);
  }
  .clock.paused .dot {
    background: var(--accent);
    animation: none;
  }

  @keyframes pulse {
    0%   { box-shadow: 0 0 0 0 rgba(31, 107, 58, 0.5); }
    70%  { box-shadow: 0 0 0 6px rgba(31, 107, 58, 0); }
    100% { box-shadow: 0 0 0 0 rgba(31, 107, 58, 0); }
  }
  @media (prefers-reduced-motion: reduce) {
    .dot { animation: none; }
  }
</style>
