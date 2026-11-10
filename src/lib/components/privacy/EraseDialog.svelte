<!--
  @component
  Confirms deleting sign-ins, cookies and cache for one service, or for all of
  them when the dialog has no target. Services stay in the sidebar, signed out.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import { app } from '$lib/stores/app.svelte';

  const service = $derived(app.snapshot.services.find((s) => s.id === app.dialogTarget) ?? null);
  const subject = $derived(service ? service.name : 'every service');
</script>

<Dialog label="Sign out and clear data for {subject}" width={440} onclose={() => app.closeDialog()}>
  <div class="body">
    <h2>Sign out of {subject}?</h2>
    <p>
      Cloister deletes the saved sign-in, cookies and cache{service ? '' : ' of every service'} from this computer.
      {service ? 'It stays' : 'They stay'} in the sidebar, and you sign in again next time you open {service
        ? 'it'
        : 'them'}. This cannot be undone.
    </p>
    <div class="buttons">
      <button type="button" class="btn" onclick={() => app.closeDialog()} data-autofocus>Cancel</button>
      <button type="button" class="btn btn-danger" onclick={() => app.eraseData(service?.id ?? null)}>
        Sign out and clear
      </button>
    </div>
  </div>
</Dialog>

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
