<!--
  @component
  Rename a service, e.g. "WhatsApp" to "WhatsApp · Work" for a second account.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import { app } from '$lib/stores/app.svelte';

  const service = $derived(app.snapshot.services.find((s) => s.id === app.dialogTarget) ?? null);
  let name = $state(app.snapshot.services.find((s) => s.id === app.dialogTarget)?.name ?? '');

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (service && name.trim()) await app.renameService(service.id, name);
  }
</script>

{#if service}
  <Dialog label="Rename {service.name}" width={420} onclose={() => app.closeDialog()}>
    <form class="body" onsubmit={submit}>
      <h2>Rename {service.name}</h2>
      <label class="field">
        <span class="field-label">Name</span>
        <input
          class="input"
          type="text"
          name="service-name"
          autocomplete="off"
          maxlength="60"
          placeholder="WhatsApp · Work…"
          bind:value={name}
          data-autofocus
        />
        <span class="field-hint">Useful when you add the same service twice for different accounts.</span>
      </label>
      <div class="buttons">
        <button type="button" class="btn" onclick={() => app.closeDialog()}>Cancel</button>
        <button type="submit" class="btn btn-primary" disabled={!name.trim()}>Save</button>
      </div>
    </form>
  </Dialog>
{/if}

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  h2 {
    font-size: var(--text-lg);
    font-weight: 600;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
</style>
