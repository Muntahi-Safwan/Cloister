<!--
  @component
  Covers the whole window while Cloister is locked. Service pages are hidden
  by the backend, so nothing behind this screen is drawn at all.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import Mark from '$lib/components/common/Mark.svelte';
  import WindowControls from '$lib/components/shell/WindowControls.svelte';
  import { app } from '$lib/stores/app.svelte';

  let button: HTMLButtonElement | undefined = $state();

  // Enter or Space unlocks straight away.
  $effect(() => button?.focus());
</script>

<div class="lock" role="dialog" aria-modal="true" aria-label="Cloister is locked">
  <div class="bar" data-tauri-drag-region>
    <WindowControls />
  </div>

  <div class="center">
    <Mark size={44} />
    <h1>Cloister is locked</h1>
    <p>Your conversations stay hidden until you confirm it is you.</p>
    <button
      type="button"
      class="btn btn-primary unlock"
      bind:this={button}
      disabled={app.unlocking}
      onclick={() => app.unlock()}
    >
      <Icon name="lock" size={16} strokeWidth={2} />
      {app.unlocking ? 'Waiting for Windows Hello…' : 'Unlock with Windows Hello'}
    </button>
    {#if app.snapshot.unreadTotal > 0}
      <p class="unread">New messages are waiting. Notifications hide their text while locked.</p>
    {/if}
  </div>
</div>

<style>
  .lock {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    background: var(--bg-app);
  }

  .bar {
    display: flex;
    justify-content: flex-end;
    height: var(--titlebar-height);
  }

  .center {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    padding-bottom: var(--titlebar-height);
    text-align: center;
  }

  h1 {
    margin-top: var(--space-3);
    font-size: 22px;
    font-weight: 600;
  }

  p {
    max-width: 340px;
    color: var(--text-muted);
  }

  .unlock {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-3);
  }

  .unread {
    margin-top: var(--space-2);
    color: var(--text-faint);
    font-size: var(--text-sm);
  }
</style>
