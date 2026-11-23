<!--
  @component
  Files saved from services since Cloister started, newest first. The list is
  kept in memory only; the files stay in the Downloads folder. File names are
  hidden while Veil is on.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';
  import { app } from '$lib/stores/app.svelte';
  import type { DownloadItem } from '$lib/types';
  import { timeAgo } from '$lib/utils/format';

  const servicesById = $derived(new Map(app.snapshot.services.map((s) => [s.id, s])));
  const items = $derived(app.snapshot.downloads.filter((item) => servicesById.has(item.serviceId)));
  const hasFinished = $derived(items.some((item) => item.state !== 'saving'));
  const now = Date.now();

  function status(item: DownloadItem): string {
    if (item.state === 'saving') return 'Saving…';
    if (item.state === 'failed') return "Didn't download";
    return timeAgo(item.startedMs, now);
  }
</script>

<Dialog label="Downloads" width={520} showClose onclose={() => app.closeDialog()}>
  <div class="panel">
    <header>
      <h2>Downloads</h2>
      <p class="lead">
        Files saved from your services go to your Downloads folder. This list is forgotten when Cloister
        closes; the files stay.
      </p>
    </header>

    {#if items.length === 0}
      <p class="empty">Nothing downloaded yet.</p>
    {:else}
      <ul class="list">
        {#each items as item (item.id)}
          {@const service = servicesById.get(item.serviceId)!}
          <li class="item" class:is-failed={item.state === 'failed'}>
            <ServiceAvatar
              name={service.name}
              url={service.url}
              icon={service.icon}
              color={service.color}
              iconUrl={service.iconUrl}
              fallbackUrl={app.catalogIcon(service)}
              size={30}
            />
            <span class="text">
              <span class="title truncate" class:is-veiled={app.snapshot.veil}>
                {app.snapshot.veil ? 'Hidden while Veil is on' : item.fileName}
              </span>
              <span class="meta">{service.name} · {status(item)}</span>
            </span>
            {#if item.state === 'done'}
              <span class="actions">
                {#if !item.risky}
                  <button type="button" class="btn small" onclick={() => app.openDownload(item.id)}
                    >Open</button
                  >
                {/if}
                <button
                  type="button"
                  class="icon-btn"
                  title="Show in folder"
                  aria-label="Show in folder"
                  onclick={() => app.revealDownload(item.id)}
                >
                  <Icon name="folder" size={16} />
                </button>
              </span>
            {/if}
          </li>
        {/each}
      </ul>
      {#if items.some((item) => item.risky && item.state === 'done')}
        <p class="note">Programs and scripts are only shown in their folder, so nothing runs by accident.</p>
      {/if}
      {#if hasFinished}
        <div class="buttons">
          <button type="button" class="btn" onclick={() => app.clearDownloads()}>Clear list</button>
        </div>
      {/if}
    {/if}
  </div>
</Dialog>

<style>
  .panel {
    padding: var(--space-6);
  }

  h2 {
    font-size: 22px;
    font-weight: 600;
  }

  .lead,
  .empty {
    margin-top: var(--space-2);
    color: var(--text-muted);
  }

  .empty {
    margin-top: var(--space-5);
    padding: var(--space-5);
    border: 1px dashed var(--line);
    border-radius: 10px;
    color: var(--text-faint);
    text-align: center;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: min(420px, 55vh);
    margin: var(--space-5) 0 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 8px 10px;
    border-radius: 10px;
  }

  .item:hover {
    background: var(--bg-active);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .title {
    font-weight: 600;
  }

  .title.is-veiled {
    color: var(--veil-text);
    font-style: italic;
    font-weight: 500;
  }

  .is-failed .title {
    color: var(--text-muted);
  }

  .meta {
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .btn.small {
    height: 26px;
    padding: 0 10px;
    font-size: 12px;
  }

  .note {
    margin-top: var(--space-3);
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    margin-top: var(--space-4);
  }
</style>
