<!--
  @component
  Modal dialog over the content area.

  Closes on Escape and on backdrop click. While any dialog is open the store
  tells the backend to hide the active service page, because native service
  webviews always draw above the main UI (see `App.svelte`).
-->
<script lang="ts">
  import type { Snippet } from 'svelte';

  import Icon from './Icon.svelte';

  interface Props {
    label: string;
    width?: number;
    placement?: 'center' | 'top';
    showClose?: boolean;
    onclose: () => void;
    children: Snippet;
  }

  let { label, width = 560, placement = 'center', showClose = false, onclose, children }: Props = $props();

  let panel: HTMLDivElement | undefined = $state();

  $effect(() => {
    // Move focus into the dialog: the element marked `data-autofocus`, else the panel itself.
    const target = panel?.querySelector<HTMLElement>('[data-autofocus]') ?? panel;
    target?.focus();
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- Escape is handled on the window; backdrop clicks are a mouse convenience. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="backdrop" class:is-top={placement === 'top'} role="presentation" onclick={onclose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    bind:this={panel}
    class="panel"
    style:width="{width}px"
    role="dialog"
    aria-modal="true"
    aria-label={label}
    tabindex="-1"
    onclick={(event) => event.stopPropagation()}
  >
    {#if showClose}
      <button type="button" class="icon-btn close" aria-label="Close" onclick={onclose}>
        <Icon name="close" size={16} />
      </button>
    {/if}
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: var(--titlebar-height) 0 0 0;
    z-index: var(--z-dialog);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-5);
    background: var(--backdrop);
    animation: fade var(--duration) var(--ease-out);
  }

  .backdrop.is-top {
    align-items: flex-start;
    padding-top: 72px;
  }

  .panel {
    position: relative;
    max-width: 100%;
    max-height: 100%;
    overflow: auto;
    border: 1px solid var(--line-strong);
    border-radius: 14px;
    background: var(--bg-raised);
    box-shadow: var(--shadow-float);
    outline: none;
    animation: rise var(--duration) var(--ease-out);
  }

  .close {
    position: absolute;
    top: var(--space-3);
    right: var(--space-3);
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
