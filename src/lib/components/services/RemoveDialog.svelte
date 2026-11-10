<!--
  @component
  Confirms removing a service. The browser profile stays on disk, so adding the
  service again keeps you signed in.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import { app } from '$lib/stores/app.svelte';

  const service = $derived(app.snapshot.services.find((s) => s.id === app.dialogTarget) ?? null);

  async function remove() {
    if (!service) return;
    const id = service.id;
    app.closeDialog();
    await app.removeService(id);
  }
</script>

{#if service}
  <Dialog label="Remove {service.name}" width={420} onclose={() => app.closeDialog()}>
    <div class="body">
      <h2>Remove {service.name}?</h2>
      <p>It disappears from the sidebar. Your sign-in is kept, so adding it again later is instant.</p>
      <div class="buttons">
        <button type="button" class="btn" onclick={() => app.closeDialog()} data-autofocus>Keep</button>
        <button type="button" class="btn btn-danger" onclick={remove}>Remove</button>
      </div>
    </div>
  </Dialog>
{/if}

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-5);
  }

  h2 {
    font-size: var(--text-lg);
    font-weight: 600;
  }

  p {
    color: var(--text-muted);
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: var(--space-2);
  }
</style>
