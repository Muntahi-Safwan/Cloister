<!--
  @component
  About Cloister: version, author, licence and updates.
-->
<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';

  import Mark from '$lib/components/common/Mark.svelte';
  import Switch from '$lib/components/common/Switch.svelte';
  import { updates } from '$lib/stores/updates.svelte';

  interface Props {
    checkForUpdates: boolean;
    onchange: (checkForUpdates: boolean) => void;
  }

  let { checkForUpdates, onchange }: Props = $props();

  let version = $state('');
  const status = $derived(updates.status);

  const statusText = $derived.by(() => {
    switch (status.kind) {
      case 'checking':
        return 'Checking…';
      case 'current':
        return 'You have the latest version.';
      case 'available':
        return `Cloister ${status.version} is ready to install.`;
      case 'installing':
        return `Installing ${status.version}…`;
      case 'failed':
        return status.message;
      default:
        return '';
    }
  });

  $effect(() => {
    getVersion()
      .then((v) => (version = v))
      .catch(() => (version = ''));
  });
</script>

<section class="about" aria-labelledby="about-title">
  <Mark size={36} />
  <div class="text">
    <h3 id="about-title">Cloister {version}</h3>
    <p>Designed and built by <strong>Muntahi Safwan</strong>.</p>
    <p class="fine">Open source under the MIT License. Not affiliated with any of the services it opens.</p>
  </div>
</section>

<div class="updates">
  <div class="update-row">
    <p class="update-status" class:is-error={status.kind === 'failed'} role="status">
      {statusText || 'Updates are signed and downloaded from GitHub.'}
    </p>
    {#if status.kind === 'available'}
      <button type="button" class="btn btn-primary" onclick={() => updates.install()}
        >Install and restart</button
      >
    {:else}
      <button
        type="button"
        class="btn"
        disabled={status.kind === 'checking' || status.kind === 'installing'}
        onclick={() => updates.check()}
      >
        Check for updates
      </button>
    {/if}
  </div>
  <Switch
    label="Check for updates automatically"
    description="On start and every few hours. Nothing installs without your click."
    checked={checkForUpdates}
    {onchange}
  />
</div>

<style>
  .about {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-5);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-lg);
    background: var(--bg-panel);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  h3 {
    font-size: var(--text-lg);
    font-weight: 600;
  }

  p {
    color: var(--text-muted);
  }

  strong {
    color: var(--text);
    font-weight: 600;
  }

  .fine {
    color: var(--text-faint);
    font-size: 12px;
  }

  .updates {
    margin-top: var(--space-3);
  }

  .update-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) 0;
  }

  .update-status {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .update-status.is-error {
    color: var(--danger);
  }
</style>
