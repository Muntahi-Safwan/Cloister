<!--
  @component
  Title-bar status for a running focus session: countdown, held count, end button.
-->
<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { formatClock } from '$lib/utils/format';
</script>

{#if app.snapshot.focus}
  <div class="pill" role="status" aria-label="Focus session running">
    <span class="dot"></span>
    <span class="mono clock">{formatClock(app.focusRemainingMs)}</span>
    <span class="held">{app.snapshot.focus.held} held</span>
    <button type="button" class="end" onclick={() => app.endFocus()}>End</button>
  </div>
{/if}

<style>
  .pill {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 24px;
    padding: 0 3px 0 10px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    color: var(--accent-text);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .held {
    color: color-mix(in srgb, var(--accent-text) 75%, var(--text-muted));
    font-weight: 500;
  }

  .end {
    height: 18px;
    padding: 0 8px;
    border: none;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 24%, transparent);
    color: var(--accent-text);
    font-size: 11px;
  }

  .end:hover {
    background: color-mix(in srgb, var(--accent) 34%, transparent);
  }
</style>
