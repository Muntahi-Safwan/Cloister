<!--
  @component
  One service in the sidebar: icon, active indicator, unread badge and an
  "asleep" mark when the page is unloaded to save memory. Drag to reorder
  (handled by the rail), or press Alt+Up / Alt+Down while it has focus.
-->
<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import type { Service } from '$lib/types';
  import { badgeCount, isMuted } from '$lib/utils/format';

  interface Props {
    service: Service;
    active: boolean;
    position: number;
    veil: boolean;
    onselect: () => void;
    onmove: (delta: number) => void;
    onmenu: () => void;
  }

  let { service, active, position, veil, onselect, onmove, onmenu }: Props = $props();

  const muted = $derived(isMuted(service.mutedUntil, Date.now()));

  function onKeydown(event: KeyboardEvent) {
    if (!event.altKey || (event.key !== 'ArrowUp' && event.key !== 'ArrowDown')) return;
    event.preventDefault();
    onmove(event.key === 'ArrowUp' ? -1 : 1);
  }

  const asleep = $derived(!service.loaded && service.options.background === 'unloadWhenIdle');
  const label = $derived(
    [
      service.name,
      service.unread > 0 ? `${service.unread} unread` : null,
      asleep ? 'asleep' : null,
      muted ? 'muted' : null,
      position <= 9 ? `Ctrl ${position}` : null,
      'Alt Up or Down to move',
    ]
      .filter(Boolean)
      .join(', '),
  );
</script>

<div class="tile-wrap" data-reorder-id={service.id}>
  <span class="indicator" class:is-active={active} aria-hidden="true"></span>
  <button
    type="button"
    class="tile"
    class:is-active={active}
    class:is-asleep={asleep}
    aria-label={label}
    aria-current={active ? 'page' : undefined}
    title={service.name}
    onclick={onselect}
    onkeydown={onKeydown}
    oncontextmenu={(event) => {
      event.preventDefault();
      onmenu();
    }}
  >
    <ServiceAvatar
      name={service.name}
      url={service.url}
      icon={service.icon}
      color={service.color}
      iconUrl={service.iconUrl}
      fallbackUrl={app.catalogIcon(service)}
      size={44}
      dimmed={asleep}
    />
  </button>
  {#if service.unread > 0}
    <span class="badge mono" class:is-quiet={veil || muted} aria-hidden="true"
      >{badgeCount(service.unread)}</span
    >
  {/if}
  {#if muted}
    <span class="muted" aria-hidden="true"><Icon name="bell-off" size={10} strokeWidth={2.4} /></span>
  {/if}
  {#if asleep}
    <span class="asleep" aria-hidden="true"><Icon name="moon" size={10} strokeWidth={2.4} /></span>
  {/if}
</div>

<style>
  .tile-wrap {
    position: relative;
    touch-action: none;
    transition: transform var(--duration) var(--ease-out);
  }

  /* While dragging, the dragged tile follows the pointer without easing. */
  :global(.is-dragging).tile-wrap {
    z-index: 1;
    transition: none;
    cursor: grabbing;
  }

  :global(.is-dragging) .tile {
    box-shadow: var(--shadow-float);
  }

  .indicator {
    position: absolute;
    top: 12px;
    left: -16px;
    width: 3px;
    height: 20px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
    opacity: 0;
    transform: scaleY(0.4);
    transition:
      opacity var(--duration) var(--ease-out),
      transform var(--duration) var(--ease-out);
  }

  .indicator.is-active {
    opacity: 1;
    transform: scaleY(1);
  }

  .tile {
    display: block;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-tile);
    background: none;
    transition: transform var(--duration-fast) var(--ease-out);
  }

  .tile:hover {
    transform: translateY(-1px);
  }

  .tile.is-active {
    border-color: var(--line-strong);
  }

  .tile.is-asleep {
    border-style: dashed;
    border-color: var(--line-strong);
  }

  .badge {
    position: absolute;
    top: -5px;
    right: -7px;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    border: 2px solid var(--bg-rail);
    border-radius: 9px;
    background: var(--accent);
    color: var(--accent-ink);
    font-size: 10px;
  }

  .badge.is-quiet {
    background: var(--text-muted);
  }

  .muted {
    position: absolute;
    bottom: -3px;
    left: -4px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--bg-rail);
    color: var(--text-faint);
  }

  .asleep {
    position: absolute;
    right: -4px;
    bottom: -3px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--bg-rail);
    color: var(--text-faint);
  }
</style>
