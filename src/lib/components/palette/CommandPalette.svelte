<!--
  @component
  Ctrl+K command palette: jump to a service or run an action, keyboard first.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import Icon, { type IconName } from '$lib/components/common/Icon.svelte';
  import Kbd from '$lib/components/common/Kbd.svelte';
  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';
  import { app } from '$lib/stores/app.svelte';
  import type { Service } from '$lib/types';
  import { isMuted, plural } from '$lib/utils/format';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  type Item =
    | { kind: 'service'; id: string; label: string; meta: string; keys: string; service: Service }
    | { kind: 'action'; id: string; label: string; keys: string; icon: IconName; run: () => void };

  let query = $state('');
  let selected = $state(0);

  const items = $derived.by<Item[]>(() => {
    const services: Item[] = app.visibleServices.map((service, index) => ({
      kind: 'service',
      id: service.id,
      label: service.name,
      meta: service.unread > 0 ? `${plural(service.unread, 'unread')}` : '',
      keys: index < 9 ? `Ctrl ${index + 1}` : '',
      service,
    }));
    const actions: Item[] = [
      {
        kind: 'action',
        id: 'veil',
        label: app.snapshot.veil ? 'Turn off Veil' : 'Turn on Veil',
        keys: SHORTCUT_LABELS.veil,
        icon: app.snapshot.veil ? 'eye' : 'eye-off',
        run: () => app.toggleVeil(),
      },
      {
        kind: 'action',
        id: 'focus',
        label: app.snapshot.focus
          ? 'End focus session'
          : `Start a ${app.snapshot.settings.focusMinutes} minute focus session`,
        keys: SHORTCUT_LABELS.focus,
        icon: 'focus',
        run: () => app.toggleFocus(),
      },
      {
        kind: 'action',
        id: 'notifications',
        label: 'Show recent notifications',
        keys: '',
        icon: 'bell',
        run: () => app.openNotifications(),
      },
      ...(app.snapshot.hasPin
        ? [
            {
              kind: 'action' as const,
              id: 'lock',
              label: 'Lock Cloister',
              keys: SHORTCUT_LABELS.lock,
              icon: 'lock' as const,
              run: () => app.lock(),
            },
          ]
        : []),
      {
        kind: 'action',
        id: 'add',
        label: 'Add a service or website…',
        keys: '',
        icon: 'plus',
        run: () => app.openDialog('add-service'),
      },
      {
        kind: 'action',
        id: 'settings',
        label: 'Open settings',
        keys: SHORTCUT_LABELS.settings,
        icon: 'sliders',
        run: () => app.openDialog('settings'),
      },
    ];
    const current = app.activeService;
    if (current) {
      const muted = isMuted(current.mutedUntil, Date.now());
      actions.push(
        {
          kind: 'action',
          id: 'mute',
          label: muted ? `Unmute ${current.name}` : `Mute ${current.name} for 1 hour`,
          keys: '',
          icon: 'bell-off',
          run: () => app.muteService(current.id, muted ? 'off' : 'oneHour'),
        },
        {
          kind: 'action',
          id: 'rename',
          label: `Rename ${current.name}…`,
          keys: '',
          icon: 'sliders',
          run: () => app.openDialog('rename', current.id),
        },
        {
          kind: 'action',
          id: 'sleep',
          label: `Put ${current.name} to sleep`,
          keys: '',
          icon: 'moon',
          run: () => app.sleepService(current.id),
        },
      );
    }
    const needle = query.trim().toLowerCase();
    const all = [...services, ...actions];
    return needle ? all.filter((item) => item.label.toLowerCase().includes(needle)) : all;
  });

  // Keep the highlighted row valid as the list changes.
  $effect(() => {
    if (selected >= items.length) selected = Math.max(0, items.length - 1);
  });

  function run(item: Item) {
    if (item.kind === 'service') {
      app.closeDialog();
      void app.activate(item.id);
    } else {
      app.closeDialog();
      item.run();
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selected = (selected + 1) % Math.max(items.length, 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selected = (selected - 1 + items.length) % Math.max(items.length, 1);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const item = items[selected];
      if (item) run(item);
    }
  }
</script>

<Dialog label="Command palette" width={580} placement="top" onclose={() => app.closeDialog()}>
  <div class="search">
    <Icon name="search" size={16} strokeWidth={2} />
    <input
      class="query"
      type="text"
      name="palette"
      autocomplete="off"
      spellcheck="false"
      placeholder="Jump to a service or run an action…"
      aria-label="Search services and actions"
      aria-controls="palette-results"
      bind:value={query}
      onkeydown={onKeydown}
      data-autofocus
    />
    <Kbd keys="Esc" />
  </div>

  <ul id="palette-results" class="results" role="listbox" aria-label="Results">
    {#each items as item, index (item.kind + item.id)}
      <li role="option" aria-selected={index === selected}>
        <button
          type="button"
          class="item"
          class:is-selected={index === selected}
          tabindex="-1"
          onmouseenter={() => (selected = index)}
          onclick={() => run(item)}
        >
          {#if item.kind === 'service'}
            <ServiceAvatar
              name={item.service.name}
              url={item.service.url}
              icon={item.service.icon}
              color={item.service.color}
              iconUrl={item.service.iconUrl}
              fallbackUrl={app.catalogIcon(item.service)}
              size={26}
            />
            <span class="label">{item.label}</span>
            <span class="meta">{item.meta}</span>
          {:else}
            <span class="glyph"><Icon name={item.icon} size={15} strokeWidth={1.9} /></span>
            <span class="label">{item.label}</span>
          {/if}
          {#if item.keys}<span class="keys mono">{item.keys}</span>{/if}
        </button>
      </li>
    {:else}
      <li class="none">No matches for “{query}”</li>
    {/each}
  </ul>
</Dialog>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 52px;
    padding: 0 14px;
    border-bottom: 1px solid var(--line);
    color: var(--text-faint);
  }

  .query {
    flex: 1;
    height: 36px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: var(--text-lg);
    outline: none;
  }

  .results {
    max-height: 420px;
    margin: 0;
    padding: var(--space-2);
    overflow-y: auto;
    list-style: none;
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    font-size: var(--text-md);
    text-align: left;
  }

  .item.is-selected {
    background: var(--bg-active);
  }

  .glyph {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: var(--radius-md);
    background: var(--bg-active);
    color: var(--text-muted);
  }

  .label {
    flex: 1;
  }

  .meta,
  .none {
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  .keys {
    color: var(--text-muted);
    font-size: 10.5px;
  }

  .none {
    padding: var(--space-4);
    text-align: center;
  }
</style>
