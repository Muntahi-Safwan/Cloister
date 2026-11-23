<!--
  @component
  Title-bar status for downloads. While a file is saving, and for a few seconds
  after it finishes, a pill shows its name with Open and Show in folder.
  Afterwards a download button stays, opening the list of this run's downloads.
  File names are hidden while Veil is on.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';

  import Icon from '$lib/components/common/Icon.svelte';
  import { app } from '$lib/stores/app.svelte';
  import type { DownloadItem } from '$lib/types';

  /** How long a finished download stays in the pill. */
  const SHOW_FINISHED_MS = 8_000;

  let now = $state(Date.now());
  /** When each download was first seen finished, by id. */
  let finishedAt = $state<Record<number, number>>({});
  let timer: ReturnType<typeof setInterval> | null = null;

  const latest = $derived<DownloadItem | undefined>(app.snapshot.downloads[0]);

  $effect(() => {
    const item = latest;
    if (item && item.state !== 'saving' && finishedAt[item.id] === undefined) {
      finishedAt[item.id] = Date.now();
    }
  });

  const showPill = $derived(
    !!latest && (latest.state === 'saving' || now - (finishedAt[latest.id] ?? 0) < SHOW_FINISHED_MS),
  );

  // Tick only while the pill is up, so it can hide itself.
  $effect(() => {
    if (showPill && !timer) {
      now = Date.now();
      timer = setInterval(() => (now = Date.now()), 1000);
    } else if (!showPill && timer) {
      clearInterval(timer);
      timer = null;
    }
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  const name = $derived(latest ? (app.snapshot.veil ? 'File' : latest.fileName) : '');
</script>

{#if latest && showPill}
  <div class="pill" class:is-failed={latest.state === 'failed'} role="status">
    {#if latest.state === 'saving'}
      <span class="spinner" aria-hidden="true"></span>
      <span class="name truncate saving" title="Saving {name}">{name}</span>
      <span class="sr-only">Saving</span>
    {:else if latest.state === 'failed'}
      <Icon name="alert" size={13} strokeWidth={2} />
      <span class="name truncate" title={name}>{name}</span>
      <span class="muted">Didn't download</span>
    {:else}
      <button
        type="button"
        class="name-btn"
        title={latest.risky ? 'Show in folder' : `Open ${name}`}
        onclick={() => (latest.risky ? app.revealDownload(latest.id) : app.openDownload(latest.id))}
      >
        <Icon name="check" size={13} strokeWidth={2.2} />
        <span class="name truncate">{name}</span>
      </button>
      <button
        type="button"
        class="small"
        title="Show in folder"
        aria-label="Show in folder"
        onclick={() => app.revealDownload(latest.id)}
      >
        <Icon name="folder" size={13} />
      </button>
    {/if}
  </div>
{/if}

{#if app.snapshot.downloads.length > 0}
  <button
    type="button"
    class="icon-btn downloads"
    aria-label={app.hasUnseenDownloads ? 'Downloads, new items' : 'Downloads'}
    title="Downloads"
    onclick={() => app.openDownloads()}
  >
    <Icon name="download" size={16} />
    {#if app.hasUnseenDownloads && !showPill}<span class="dot" aria-hidden="true"></span>{/if}
  </button>
{/if}

<style>
  .pill {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 190px;
    min-width: 0;
    height: 26px;
    padding: 0 4px 0 10px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--bg-panel);
    color: var(--text);
    font-size: 12px;
    animation: rise var(--duration-fast) var(--ease-out);
  }

  .pill.is-failed {
    padding-right: 10px;
    color: var(--danger);
  }

  .name {
    min-width: 0;
    font-weight: 500;
  }

  .saving {
    padding-right: 6px;
    color: var(--text-muted);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }

  .muted {
    flex: none;
    white-space: nowrap;
    padding-right: 6px;
    color: var(--text-faint);
  }

  .name-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    height: 20px;
    padding: 0 6px 0 0;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
  }

  .name-btn :global(svg) {
    flex: none;
    color: var(--accent-text);
  }

  .name-btn:hover .name {
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .small {
    display: grid;
    flex: none;
    place-items: center;
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 999px;
    background: transparent;
    color: var(--text-muted);
  }

  .small:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .spinner {
    flex: none;
    width: 11px;
    height: 11px;
    border: 1.6px solid var(--line-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .downloads {
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

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(3px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .pill,
    .spinner {
      animation: none;
    }
  }
</style>
