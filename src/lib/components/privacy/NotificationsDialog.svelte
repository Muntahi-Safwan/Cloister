<!--
  @component
  The last 50 notifications, newest first. Kept in memory only and forgotten
  when Cloister quits. Message text is hidden while Veil is on.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';
  import { app } from '$lib/stores/app.svelte';
  import type { Delivery, HistoryItem } from '$lib/types';
  import { timeAgo } from '$lib/utils/format';

  const DELIVERY_LABEL: Record<Delivery, string | null> = {
    shown: null,
    held: 'Held',
    silenced: 'Silenced',
  };

  const servicesById = $derived(new Map(app.snapshot.services.map((s) => [s.id, s])));
  const items = $derived(app.history.filter((item) => servicesById.has(item.serviceId)));
  const now = Date.now();

  function open(item: HistoryItem) {
    app.closeDialog();
    void app.activate(item.serviceId);
  }
</script>

<Dialog label="Recent notifications" width={520} showClose onclose={() => app.closeDialog()}>
  <div class="panel">
    <header>
      <h2>Recent notifications</h2>
      <p class="lead">The last 50, kept in memory only. Closing Cloister forgets them.</p>
    </header>

    {#if items.length === 0}
      <p class="empty">Nothing yet. New notifications appear here, even ones held by focus or muted.</p>
    {:else}
      <ul class="list" class:is-veiled={app.snapshot.veil}>
        {#each items as item, index (index)}
          {@const service = servicesById.get(item.serviceId)!}
          {@const label = DELIVERY_LABEL[item.delivery]}
          <li>
            <button type="button" class="item" onclick={() => open(item)}>
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
                <span class="title truncate">
                  {app.snapshot.veil ? service.name : item.title || service.name}
                </span>
                <span class="body truncate">
                  {app.snapshot.veil ? 'Hidden while Veil is on' : item.body || 'New notification'}
                </span>
              </span>
              <span class="meta">
                <span class="time mono">{timeAgo(item.atMs, now)}</span>
                {#if label}<span class="tag">{label}</span>{/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>
      <div class="buttons">
        <button type="button" class="btn" onclick={() => app.clearHistory()}>Clear all</button>
      </div>
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
    width: 100%;
    padding: 10px;
    border: none;
    border-radius: 10px;
    background: transparent;
    text-align: left;
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

  .body {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .is-veiled .body {
    color: var(--veil-text);
    font-style: italic;
  }

  .meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }

  .time {
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  .tag {
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--bg-hover);
    color: var(--text-muted);
    font-size: 11px;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    margin-top: var(--space-4);
  }
</style>
