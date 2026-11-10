<!--
  @component
  App lock (Cloister PIN, plus Windows Hello where set up) and automatic Veil settings.
-->
<script lang="ts">
  import Switch from '$lib/components/common/Switch.svelte';
  import { errorMessage } from '$lib/api/backend';
  import { app } from '$lib/stores/app.svelte';
  import type { AppSettings } from '$lib/types';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  interface Props {
    settings: AppSettings;
    hasPin: boolean;
    helloAvailable: boolean;
    onchange: (patch: Partial<AppSettings>) => void;
  }

  let { settings, hasPin, helloAvailable, onchange }: Props = $props();

  const LOCK_AFTER = [
    [0, 'Only on start'],
    [1, 'After 1 minute away'],
    [5, 'After 5 minutes away'],
    [15, 'After 15 minutes away'],
    [60, 'After 1 hour away'],
  ] as const;

  const VEIL_AFTER = [
    [0, 'Never'],
    [1, 'After 1 minute away'],
    [5, 'After 5 minutes away'],
    [15, 'After 15 minutes away'],
  ] as const;

  /** Which PIN form is open, if any. */
  let editing = $state<'set' | 'remove' | null>(null);
  let current = $state('');
  let next = $state('');
  let confirm = $state('');
  let formError = $state<string | null>(null);
  let saving = $state(false);
  /** Turn the lock on once the first PIN is saved from the switch. */
  let lockAfterSave = false;

  function open(kind: 'set' | 'remove', thenLock = false) {
    editing = kind;
    lockAfterSave = thenLock;
    current = next = confirm = '';
    formError = null;
  }

  function close() {
    editing = null;
    current = next = confirm = '';
    formError = null;
  }

  function toggleLock(on: boolean) {
    if (on && !hasPin) open('set', true);
    else onchange({ appLock: on });
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    formError = null;
    if (editing === 'set' && next !== confirm) {
      formError = 'The two PINs do not match.';
      return;
    }
    saving = true;
    try {
      if (editing === 'set') {
        await app.setPin(hasPin ? current : null, next);
        if (lockAfterSave) onchange({ appLock: true });
      } else if (editing === 'remove') {
        await app.removePin(current);
      }
      close();
    } catch (error) {
      formError = errorMessage(error);
    } finally {
      saving = false;
    }
  }
</script>

<Switch
  label="Lock Cloister"
  description={hasPin
    ? helloAvailable
      ? `Unlock with your Cloister PIN or Windows Hello. Lock any time with ${SHORTCUT_LABELS.lock}.`
      : `Unlock with your Cloister PIN. Lock any time with ${SHORTCUT_LABELS.lock}.`
    : 'Turning this on asks you to choose a Cloister PIN first.'}
  checked={settings.appLock && hasPin}
  onchange={toggleLock}
/>
{#if settings.appLock && hasPin}
  <div class="row">
    <label for="lock-after" class="row-text">
      <span>Lock again</span>
      <span class="hint">“Away” means the window is in the background or in the tray</span>
    </label>
    <select
      id="lock-after"
      class="select"
      value={settings.lockAfterMinutes}
      onchange={(e) => onchange({ lockAfterMinutes: Number(e.currentTarget.value) })}
    >
      {#each LOCK_AFTER as [minutes, label] (minutes)}
        <option value={minutes}>{label}</option>
      {/each}
    </select>
  </div>
{/if}
<div class="row">
  <div class="row-text">
    <span>Cloister PIN</span>
    <span class="hint">
      {hasPin
        ? 'Works on every computer, even when Windows Hello does not'
        : 'Four or more characters. It only unlocks Cloister and is separate from your Windows PIN.'}
    </span>
  </div>
  {#if editing === null}
    <div class="actions">
      <button type="button" class="btn" onclick={() => open('set')}>
        {hasPin ? 'Change' : 'Set PIN'}
      </button>
      {#if hasPin}
        <button type="button" class="btn btn-danger" onclick={() => open('remove')}>Remove</button>
      {/if}
    </div>
  {/if}
</div>
{#if editing}
  <form class="pin-form" onsubmit={submit}>
    {#if hasPin}
      <input
        class="input"
        type="password"
        autocomplete="off"
        placeholder="Current PIN"
        aria-label="Current PIN"
        bind:value={current}
        required
      />
    {/if}
    {#if editing === 'set'}
      <input
        class="input"
        type="password"
        autocomplete="new-password"
        placeholder="New PIN"
        aria-label="New PIN"
        minlength="4"
        maxlength="64"
        bind:value={next}
        required
      />
      <input
        class="input"
        type="password"
        autocomplete="new-password"
        placeholder="New PIN again"
        aria-label="New PIN again"
        bind:value={confirm}
        required
      />
    {:else}
      <p class="hint">Removing the PIN also turns off the lock.</p>
    {/if}
    {#if formError}<p class="hint is-error" role="alert">{formError}</p>{/if}
    <div class="actions">
      <button type="button" class="btn" onclick={close}>Cancel</button>
      <button
        type="submit"
        class="btn {editing === 'remove' ? 'btn-danger' : 'btn-primary'}"
        disabled={saving}
      >
        {editing === 'remove' ? 'Remove PIN' : 'Save PIN'}
      </button>
    </div>
  </form>
{/if}
<Switch
  label="Turn on Veil when I switch away"
  description="Veil starts the moment another window takes focus"
  checked={settings.veilOnBlur}
  onchange={(veilOnBlur) => onchange({ veilOnBlur })}
/>
{#if !settings.veilOnBlur}
  <div class="row">
    <label for="veil-after" class="row-text">
      <span>Turn on Veil when idle</span>
      <span class="hint">For when you step away and leave Cloister on screen in the background</span>
    </label>
    <select
      id="veil-after"
      class="select"
      value={settings.veilAfterMinutes}
      onchange={(e) => onchange({ veilAfterMinutes: Number(e.currentTarget.value) })}
    >
      {#each VEIL_AFTER as [minutes, label] (minutes)}
        <option value={minutes}>{label}</option>
      {/each}
    </select>
  </div>
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: 11px 0;
    border-top: 1px solid var(--line-subtle);
  }

  .row-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .hint {
    font-size: 12px;
    color: var(--text-faint);
  }

  .select {
    width: 190px;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .pin-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-width: 320px;
    margin-left: auto;
    padding: 0 0 11px;
  }

  .is-error {
    color: var(--danger);
  }
</style>
