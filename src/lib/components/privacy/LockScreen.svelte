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

  let field: HTMLInputElement | undefined = $state();
  let pin = $state('');

  // Ready to type the PIN straight away.
  $effect(() => field?.focus());

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!pin) return;
    const ok = await app.unlockWithPin(pin);
    if (!ok) {
      pin = '';
      field?.focus();
    }
  }
</script>

<div class="lock" role="dialog" aria-modal="true" aria-label="Cloister is locked">
  <div class="bar" data-tauri-drag-region>
    <WindowControls />
  </div>

  <div class="center">
    <Mark size={44} />
    <h1>Cloister is locked</h1>
    <p>Your conversations stay hidden until you confirm it is you.</p>
    <form class="pin" onsubmit={submit}>
      <input
        class="input"
        type="password"
        autocomplete="off"
        placeholder="Cloister PIN"
        aria-label="Cloister PIN"
        aria-invalid={app.unlockError ? 'true' : undefined}
        aria-describedby={app.unlockError ? 'unlock-error' : undefined}
        bind:this={field}
        bind:value={pin}
      />
      <button type="submit" class="btn btn-primary" disabled={app.unlocking || !pin}>
        <Icon name="lock" size={16} strokeWidth={2} />
        Unlock
      </button>
    </form>
    {#if app.unlockError}
      <p id="unlock-error" class="error" role="alert">{app.unlockError}</p>
    {/if}
    {#if app.snapshot.helloAvailable}
      <button type="button" class="btn hello" disabled={app.unlocking} onclick={() => app.unlock()}>
        {app.unlocking ? 'Waiting…' : 'Use Windows Hello instead'}
      </button>
    {/if}
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

  .pin {
    display: flex;
    gap: var(--space-2);
    width: 300px;
    margin-top: var(--space-3);
  }

  .pin .input {
    flex: 1;
    min-width: 0;
  }

  .pin .btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .error {
    color: var(--danger);
    font-size: var(--text-sm);
  }

  .hello {
    font-size: var(--text-sm);
  }

  .unread {
    margin-top: var(--space-2);
    color: var(--text-faint);
    font-size: var(--text-sm);
  }
</style>
