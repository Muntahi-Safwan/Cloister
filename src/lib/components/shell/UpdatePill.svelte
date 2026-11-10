<!--
  @component
  Title-bar pill shown when a new version is ready. Clicking installs it and
  restarts Cloister; progress shows in place.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import { updates } from '$lib/stores/updates.svelte';

  const status = $derived(updates.status);
</script>

{#if status.kind === 'available'}
  <button
    type="button"
    class="pill"
    title="Install Cloister {status.version} and restart"
    onclick={() => updates.install()}
  >
    <Icon name="download" size={14} strokeWidth={2} />
    Update to {status.version}
  </button>
{:else if status.kind === 'installing'}
  <span class="pill is-busy" role="status">
    Updating{status.progress !== null ? ` ${Math.round(status.progress * 100)}%` : '…'}
  </span>
{/if}

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 11px;
    border: 1px solid color-mix(in srgb, var(--accent) 45%, var(--line));
    border-radius: 999px;
    background: var(--veil-bg);
    color: var(--accent-text);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }

  button.pill:hover {
    border-color: var(--accent);
  }

  .is-busy {
    color: var(--text-muted);
  }
</style>
