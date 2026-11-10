<!--
  @component
  The sidebar: services (drag to reorder), the add button, and the
  Focus / Veil / Settings actions.
  During a focus session an ember line fills the rail's edge from the bottom.
-->
<script lang="ts">
  import { reorderable } from '$lib/actions/reorderable';
  import Icon from '$lib/components/common/Icon.svelte';
  import { app } from '$lib/stores/app.svelte';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  import RailTile from './RailTile.svelte';

  const focusProgress = $derived.by(() => {
    const focus = app.snapshot.focus;
    if (!focus) return 0;
    const total = focus.endsAtMs - focus.startedAtMs;
    return total > 0 ? Math.min(1, Math.max(0, 1 - app.focusRemainingMs / total)) : 0;
  });

  const focusLabel = $derived(
    `${app.snapshot.focus ? 'End focus session' : 'Start focus session'} (${SHORTCUT_LABELS.focus})`,
  );
  const veilLabel = $derived(
    `${app.snapshot.veil ? 'Turn off Veil' : 'Turn on Veil'} (${SHORTCUT_LABELS.veil})`,
  );
</script>

<nav class="rail" aria-label="Services">
  <span class="focus-line" style:transform="scaleY({focusProgress})" aria-hidden="true"></span>

  <div class="services">
    <div class="tiles" use:reorderable={{ onreorder: (id, toIndex) => app.moveService(id, toIndex) }}>
      {#each app.visibleServices as service, index (service.id)}
        <RailTile
          {service}
          position={index + 1}
          active={service.id === app.snapshot.active}
          veil={app.snapshot.veil}
          onselect={() => app.activate(service.id)}
          onmove={(delta) => app.moveService(service.id, index + delta)}
          onmenu={() => app.openServiceMenu(service)}
        />
      {/each}
    </div>

    <button
      type="button"
      class="add"
      aria-label="Add a service"
      title="Add a service"
      onclick={() => app.openDialog('add-service')}
    >
      <Icon name="plus" size={16} strokeWidth={2} />
    </button>
  </div>

  <div class="actions">
    <button
      type="button"
      class="action"
      class:is-on={app.snapshot.focus}
      aria-pressed={app.snapshot.focus !== null}
      aria-label={focusLabel}
      title={focusLabel}
      onclick={() => app.toggleFocus()}
    >
      <Icon name="focus" size={20} />
    </button>
    <button
      type="button"
      class="action"
      class:is-veiled={app.snapshot.veil}
      aria-pressed={app.snapshot.veil}
      aria-label={veilLabel}
      title={veilLabel}
      onclick={() => app.toggleVeil()}
    >
      <Icon name={app.snapshot.veil ? 'eye-off' : 'eye'} size={20} />
    </button>
    <button
      type="button"
      class="action"
      aria-label="Settings ({SHORTCUT_LABELS.settings})"
      title="Settings ({SHORTCUT_LABELS.settings})"
      onclick={() => app.openDialog('settings')}
    >
      <Icon name="sliders" size={20} />
    </button>
  </div>
</nav>

<style>
  .rail {
    position: relative;
    display: flex;
    flex-direction: column;
    width: var(--rail-width);
    flex: none;
    border-right: 1px solid var(--line-subtle);
    background: var(--bg-rail);
  }

  .focus-line {
    position: absolute;
    top: 0;
    right: -1px;
    bottom: 0;
    width: 2px;
    background: var(--accent);
    transform-origin: bottom;
    transition: transform 1s linear;
  }

  .services {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 14px 0;
    overflow-y: auto;
    scrollbar-width: none;
  }

  .tiles {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .add {
    display: flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--radius-tile);
    background: transparent;
    color: var(--text-faint);
    transition: color var(--duration-fast) var(--ease-out);
  }

  .add:hover {
    color: var(--text);
  }

  .actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 10px 0 14px;
  }

  .action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border: none;
    border-radius: var(--radius-lg);
    background: transparent;
    color: var(--text-muted);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .action:hover {
    background: var(--bg-hover);
  }

  .action.is-on {
    color: var(--accent);
  }

  .action.is-veiled {
    background: var(--bg-hover);
    color: var(--text);
  }
</style>
