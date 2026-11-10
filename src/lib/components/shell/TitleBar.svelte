<!--
  @component
  The custom title bar: brand, active service, search, status and window controls.
  Empty areas are draggable (`data-tauri-drag-region`).
-->
<script lang="ts">
  import FocusPill from '$lib/components/focus/FocusPill.svelte';
  import UpdatePill from './UpdatePill.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import Kbd from '$lib/components/common/Kbd.svelte';
  import Mark from '$lib/components/common/Mark.svelte';
  import { app } from '$lib/stores/app.svelte';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  import WindowControls from './WindowControls.svelte';
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="start" data-tauri-drag-region>
    <Mark size={16} />
    <span class="brand" data-tauri-drag-region>cloister</span>
    {#if app.activeService}
      <span class="divider" aria-hidden="true"></span>
      <span class="active truncate" data-tauri-drag-region>{app.activeService.name}</span>
    {/if}
  </div>

  <div class="center" data-tauri-drag-region>
    <button type="button" class="search" onclick={() => app.openDialog('palette')}>
      <Icon name="search" size={14} strokeWidth={2} />
      <span class="search-label">Search or jump to a service or action</span>
      <Kbd keys={SHORTCUT_LABELS.palette} />
    </button>
  </div>

  <div class="end" data-tauri-drag-region>
    {#if app.snapshot.veil}
      <span class="veil-pill"><Icon name="eye-off" size={12} strokeWidth={2} />Veil</span>
    {/if}
    <UpdatePill />
    <FocusPill />
    <button
      type="button"
      class="icon-btn bell"
      aria-label={app.hasUnseenHistory ? 'Recent notifications, new items' : 'Recent notifications'}
      title="Recent notifications"
      onclick={() => app.openNotifications()}
    >
      <Icon name="bell" size={16} />
      {#if app.hasUnseenHistory}<span class="dot" aria-hidden="true"></span>{/if}
    </button>
    {#if app.snapshot.hasPin}
      <button
        type="button"
        class="icon-btn"
        aria-label="Lock Cloister"
        title="Lock ({SHORTCUT_LABELS.lock})"
        onclick={() => app.lock()}
      >
        <Icon name="lock" size={16} />
      </button>
    {/if}
    <WindowControls />
  </div>
</header>

<style>
  .titlebar {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
    height: var(--titlebar-height);
    padding-left: var(--space-4);
    border-bottom: 1px solid var(--line-subtle);
    background: var(--bg-chrome);
  }

  .start,
  .end {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 100%;
    min-width: 0;
  }

  .end {
    justify-content: flex-end;
  }

  .brand {
    font-size: var(--text-md);
    font-weight: 600;
  }

  .divider {
    width: 1px;
    height: 14px;
    margin: 0 var(--space-1);
    background: var(--line);
  }

  .active {
    font-size: 13px;
    color: var(--text-muted);
  }

  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    width: min(400px, 36vw);
    height: 28px;
    padding: 0 6px 0 10px;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--bg-panel);
    color: var(--text-faint);
    font-size: var(--text-sm);
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  .search:hover {
    border-color: var(--line-strong);
  }

  .search-label {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bell {
    position: relative;
  }

  .dot {
    position: absolute;
    top: 7px;
    right: 7px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .veil-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--bg-hover);
    color: var(--veil-text);
    font-size: var(--text-xs);
    font-weight: 500;
  }
</style>
