<!--
  @component
  Shows the latest error from the store, announced to screen readers.

  It sits in the title bar: anything over the content area would be hidden
  behind the native service page.
-->
<script lang="ts">
  import { app } from '$lib/stores/app.svelte';

  import Icon from './Icon.svelte';
</script>

<div class="region" aria-live="polite">
  {#if app.error}
    <div class="toast" role="alert">
      <Icon name="alert" size={16} />
      <span class="message">{app.error}</span>
      <button type="button" class="icon-btn" aria-label="Dismiss" onclick={() => app.dismissError()}>
        <Icon name="close" size={14} />
      </button>
    </div>
  {/if}
</div>

<style>
  .region {
    position: fixed;
    top: 4px;
    left: 50%;
    z-index: var(--z-toast);
    transform: translateX(-50%);
  }

  .toast {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    max-width: 520px;
    height: 32px;
    padding: 0 var(--space-1) 0 var(--space-3);
    border: 1px solid var(--danger-line);
    border-radius: var(--radius-lg);
    background: var(--bg-raised);
    color: var(--danger);
    box-shadow: var(--shadow-float);
  }

  .message {
    flex: 1;
    color: var(--text);
    font-size: var(--text-sm);
  }
</style>
