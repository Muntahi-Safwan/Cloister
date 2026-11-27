<!--
  @component
  Title-bar status while presenting: Cloister is hidden from screen sharing and
  notifications wait. Shows how many are waiting and ends with one click.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import { app } from '$lib/stores/app.svelte';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';
</script>

{#if app.snapshot.presenting}
  <div
    class="pill"
    role="status"
    title={app.snapshot.canHideFromCapture
      ? 'Cloister is hidden from screen sharing and recording. Notifications wait until you end.'
      : 'Notifications wait until you end. This system cannot hide windows from screen sharing.'}
  >
    <Icon name="eye-off" size={13} strokeWidth={2} />
    <span>{app.snapshot.canHideFromCapture ? 'Hidden from screen share' : 'Presenting'}</span>
    {#if app.snapshot.presenting.held > 0}
      <span class="held">{app.snapshot.presenting.held} waiting</span>
    {/if}
    <button
      type="button"
      class="end"
      title="Stop presenting ({SHORTCUT_LABELS.present})"
      onclick={() => app.togglePresenting()}>End</button
    >
  </div>
{/if}

<style>
  .pill {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-2);
    height: 24px;
    padding: 0 3px 0 10px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    color: var(--accent-text);
    font-size: var(--text-xs);
    font-weight: 600;
    white-space: nowrap;
  }

  .held {
    opacity: 0.8;
    font-weight: 500;
  }

  .end {
    height: 18px;
    padding: 0 8px;
    border: none;
    border-radius: 999px;
    background: color-mix(in srgb, currentColor 18%, transparent);
    color: inherit;
    font-size: 11px;
  }

  .end:hover {
    background: color-mix(in srgb, currentColor 28%, transparent);
  }
</style>
