<!--
  @component
  App lock (Windows Hello) and automatic Veil settings.
-->
<script lang="ts">
  import Switch from '$lib/components/common/Switch.svelte';
  import type { AppSettings } from '$lib/types';
  import { IS_WINDOWS, SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  interface Props {
    settings: AppSettings;
    authAvailable: boolean;
    onchange: (patch: Partial<AppSettings>) => void;
  }

  let { settings, authAvailable, onchange }: Props = $props();

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
</script>

<Switch
  label="Lock with Windows Hello"
  description={authAvailable
    ? `Ask for your PIN, face or fingerprint on start. Lock any time with ${SHORTCUT_LABELS.lock}.`
    : IS_WINDOWS
      ? 'Set up a PIN, face or fingerprint in Windows Settings › Accounts › Sign-in options first.'
      : 'App lock uses Windows Hello, so it is only available on Windows for now.'}
  checked={settings.appLock && authAvailable}
  disabled={!authAvailable}
  onchange={(appLock) => onchange({ appLock })}
/>
{#if settings.appLock && authAvailable}
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
</style>
