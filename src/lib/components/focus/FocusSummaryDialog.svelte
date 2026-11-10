<!--
  @component
  End-of-session summary: what arrived while focusing, grouped by service.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';
  import { app } from '$lib/stores/app.svelte';
  import type { HeldNotification, Service } from '$lib/types';
  import { plural } from '$lib/utils/format';

  interface Group {
    service: Service;
    items: HeldNotification[];
  }

  const groups = $derived.by<Group[]>(() => {
    const byService: Record<string, HeldNotification[]> = {};
    for (const item of app.heldNotifications) {
      (byService[item.serviceId] ??= []).push(item);
    }
    return app.snapshot.services.flatMap((service) => {
      const items = byService[service.id];
      return items ? [{ service, items }] : [];
    });
  });

  /** Up to two distinct titles, e.g. "Ayesha, Design team". */
  function summary(items: HeldNotification[]): string {
    const titles = [...new Set(items.map((i) => i.title))];
    const shown = titles.slice(0, 2).join(', ');
    return titles.length > 2 ? `${shown} and ${titles.length - 2} more` : shown;
  }

  function open(serviceId: string) {
    app.closeDialog();
    void app.activate(serviceId);
  }
</script>

<Dialog label="Focus session summary" width={520} onclose={() => app.closeDialog()}>
  <div class="summary">
    <p class="eyebrow">Focus session complete</p>
    <h2>{groups.length === 0 ? 'Nothing came in' : 'While you were away'}</h2>
    <p class="lead">
      {groups.length === 0
        ? 'No notifications arrived during your session.'
        : 'Held notifications, grouped by service. Nothing was lost; badges kept counting.'}
    </p>

    {#if groups.length > 0}
      <ul class="groups">
        {#each groups as group (group.service.id)}
          <li>
            <button type="button" class="group" onclick={() => open(group.service.id)}>
              <ServiceAvatar
                name={group.service.name}
                url={group.service.url}
                icon={group.service.icon}
                color={group.service.color}
                iconUrl={group.service.iconUrl}
                fallbackUrl={app.catalogIcon(group.service)}
                size={34}
              />
              <span class="text">
                <span class="name">{group.service.name}</span>
                <span class="line truncate">{summary(group.items)}</span>
              </span>
              <span class="count mono">{plural(group.items.length, 'item')}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    <div class="buttons">
      <button type="button" class="btn" onclick={() => app.closeDialog()}>Done</button>
      <button
        type="button"
        class="btn btn-primary"
        onclick={() => {
          app.closeDialog();
          void app.startFocus();
        }}
      >
        Start another session
      </button>
    </div>
  </div>
</Dialog>

<style>
  .summary {
    padding: 28px;
  }

  .eyebrow {
    color: var(--accent);
    font-size: 13px;
    font-weight: 600;
  }

  h2 {
    margin-top: 6px;
    font-size: 22px;
    font-weight: 600;
  }

  .lead {
    margin-top: 6px;
    color: var(--text-muted);
  }

  .groups {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: var(--space-5) 0 0;
    padding: 0;
    list-style: none;
  }

  .group {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 10px;
    border: none;
    border-radius: 10px;
    background: transparent;
    text-align: left;
  }

  .group:hover {
    background: var(--bg-active);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-weight: 600;
  }

  .line {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .count {
    color: var(--accent);
    font-size: 12px;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: var(--space-5);
  }
</style>
