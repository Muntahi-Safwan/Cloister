<!--
  @component
  Banner shown above the content area while Veil is on. It takes layout space,
  so the service page is placed below it rather than covering it.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import { app } from '$lib/stores/app.svelte';
</script>

{#if app.snapshot.veil}
  <div class="banner" role="status">
    <Icon name="eye-off" size={14} strokeWidth={2} />
    <strong>Veil is on.</strong>
    <span class="detail truncate">
      Chats and messages are blurred in every service. Hover to peek. Notifications hide their text.
    </span>
    <button type="button" class="off" onclick={() => app.toggleVeil()}>Turn off</button>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    flex: none;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 var(--space-4);
    border-bottom: 1px solid color-mix(in srgb, var(--veil-text) 12%, transparent);
    background: var(--veil-bg);
    color: var(--veil-text);
    font-size: var(--text-sm);
  }

  .detail {
    flex: 1;
    color: var(--veil-muted);
  }

  .off {
    height: 24px;
    padding: 0 10px;
    border: 1px solid color-mix(in srgb, var(--veil-text) 28%, transparent);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--veil-text);
    font-size: 12px;
  }

  .off:hover {
    background: color-mix(in srgb, var(--veil-text) 8%, transparent);
  }
</style>
