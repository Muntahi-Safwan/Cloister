<!--
  @component
  Root layout: title bar on top, the service rail on the left, and the content
  area where the active service page is drawn by the backend.
-->
<script lang="ts">
  import { updates } from '$lib/stores/updates.svelte';
  import { onMount } from 'svelte';

  import { backend } from '$lib/api/backend';
  import ErrorToast from '$lib/components/common/ErrorToast.svelte';
  import FocusSummaryDialog from '$lib/components/focus/FocusSummaryDialog.svelte';
  import EraseDialog from '$lib/components/privacy/EraseDialog.svelte';
  import LockScreen from '$lib/components/privacy/LockScreen.svelte';
  import NotificationsDialog from '$lib/components/privacy/NotificationsDialog.svelte';
  import CommandPalette from '$lib/components/palette/CommandPalette.svelte';
  import AddServiceDialog from '$lib/components/services/AddServiceDialog.svelte';
  import RemoveDialog from '$lib/components/services/RemoveDialog.svelte';
  import RenameDialog from '$lib/components/services/RenameDialog.svelte';
  import SettingsDialog from '$lib/components/settings/SettingsDialog.svelte';
  import ContentArea from '$lib/components/shell/ContentArea.svelte';
  import Rail from '$lib/components/shell/Rail.svelte';
  import TitleBar from '$lib/components/shell/TitleBar.svelte';
  import VeilBanner from '$lib/components/shell/VeilBanner.svelte';
  import { app } from '$lib/stores/app.svelte';
  import { applyTheme, resolveTheme, systemPrefersDark, watchSystemScheme } from '$lib/theme';
  import { shortcutFor } from '$lib/utils/shortcuts';

  onMount(() => {
    app.init().catch((error) => console.error('Cloister failed to start', error));
  });

  // Apply the chosen theme, and follow Windows light/dark when it is "system".
  let prefersDark = $state(systemPrefersDark());
  onMount(() => watchSystemScheme((dark) => (prefersDark = dark)));
  $effect(() => {
    if (app.ready) applyTheme(resolveTheme(app.snapshot.settings.theme, prefersDark));
  });

  // Service pages are native views drawn above this UI, so hide the active
  // one while a dialog is open and show it again afterwards.
  $effect(() => {
    void backend.setOverlayOpen(app.dialog !== null);
  });

  /** Hide the browser's own right-click menu (Back, Reload, Inspect…) except in text fields. */
  function onContextMenu(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('input, textarea, [contenteditable="true"]')) event.preventDefault();
  }

  function onKeydown(event: KeyboardEvent) {
    const action = shortcutFor(event);
    if (!action || app.snapshot.locked) return;
    event.preventDefault();
    void app.runAction(action);
  }

  // Look for new versions on start and every few hours, unless turned off.
  $effect(() => updates.setAutomatic(app.snapshot.settings.checkForUpdates));
</script>

<svelte:window onkeydown={onKeydown} oncontextmenu={onContextMenu} />

<div class="shell">
  <TitleBar />
  <div class="body">
    <Rail />
    <div class="main">
      <VeilBanner />
      <ContentArea />
    </div>
  </div>
</div>

{#if app.dialog === 'palette'}
  <CommandPalette />
{:else if app.dialog === 'add-service'}
  <AddServiceDialog />
{:else if app.dialog === 'settings'}
  <SettingsDialog />
{:else if app.dialog === 'focus-summary'}
  <FocusSummaryDialog />
{:else if app.dialog === 'rename'}
  <RenameDialog />
{:else if app.dialog === 'notifications'}
  <NotificationsDialog />
{:else if app.dialog === 'confirm-remove'}
  <RemoveDialog />
{:else if app.dialog === 'confirm-erase'}
  <EraseDialog />
{/if}

{#if app.snapshot.locked}
  <LockScreen />
{/if}

<ErrorToast />

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
</style>
