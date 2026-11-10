<!--
  @component
  "Add a service" dialog with two tabs: the catalog and a custom website.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import { app } from '$lib/stores/app.svelte';

  import CatalogPanel from './CatalogPanel.svelte';
  import CustomServiceForm from './CustomServiceForm.svelte';

  type Tab = 'catalog' | 'custom';
  let tab = $state<Tab>('catalog');

  const close = () => app.closeDialog();
</script>

<Dialog label="Add a service" width={920} showClose onclose={close}>
  <div class="header">
    <h2>Add a service</h2>
    <div class="tabs" role="tablist">
      <button type="button" role="tab" aria-selected={tab === 'catalog'} onclick={() => (tab = 'catalog')}>
        Catalog
      </button>
      <button type="button" role="tab" aria-selected={tab === 'custom'} onclick={() => (tab = 'custom')}>
        Custom website
      </button>
    </div>
  </div>

  <div role="tabpanel">
    {#if tab === 'catalog'}
      <CatalogPanel oncustom={() => (tab = 'custom')} />
    {:else}
      <CustomServiceForm oncancel={close} />
    {/if}
  </div>
</Dialog>

<style>
  .header {
    padding: var(--space-5) var(--space-5) 0;
    border-bottom: 1px solid var(--line);
  }

  h2 {
    font-size: var(--text-lg);
    font-weight: 600;
  }

  .tabs {
    display: flex;
    gap: var(--space-1);
    margin-top: var(--space-3);
  }

  [role='tab'] {
    height: 36px;
    padding: 0 14px;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--text-muted);
    font-weight: 500;
  }

  [role='tab'][aria-selected='true'] {
    border-bottom-color: var(--accent);
    color: var(--text);
  }
</style>
