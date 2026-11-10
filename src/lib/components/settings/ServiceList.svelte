<!--
  @component
  Configured services with memory use, icon choice, reload, clear data and remove.
  Removing and clearing ask for confirmation.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';
  import { app } from '$lib/stores/app.svelte';
  import { formatBytes } from '$lib/utils/format';

  let confirming = $state<string | null>(null);

  function hostOf(url: string): string {
    try {
      return new URL(url).host;
    } catch {
      return url;
    }
  }
</script>

{#if app.snapshot.services.length === 0}
  <p class="empty">
    No services yet. <button type="button" class="link" onclick={() => app.openDialog('add-service')}
      >Add one</button
    >
  </p>
{:else}
  <ul class="list">
    {#each app.snapshot.services as service (service.id)}
      <li class="row">
        <ServiceAvatar
          name={service.name}
          url={service.url}
          icon={service.icon}
          color={service.color}
          iconUrl={service.iconUrl}
          fallbackUrl={app.catalogIcon(service)}
          size={30}
        />
        <span class="text">
          <span class="name">{service.name}</span>
          <span class="host mono truncate">
            {hostOf(service.url)}
            <span class="memory">
              · {service.loaded
                ? app.memory[service.id]
                  ? formatBytes(app.memory[service.id]!)
                  : 'measuring…'
                : 'asleep'}
            </span>
          </span>
        </span>
        {#if confirming === service.id}
          <span class="confirm-text">Remove {service.name}?</span>
          <button type="button" class="btn btn-small" onclick={() => (confirming = null)}>Keep</button>
          <button
            type="button"
            class="btn btn-small btn-danger"
            onclick={() => {
              confirming = null;
              void app.removeService(service.id);
            }}
          >
            Remove
          </button>
        {:else}
          <button
            type="button"
            class="chip"
            aria-label="Show {service.icon === 'favicon' ? 'initials' : 'the site icon'} for {service.name}"
            title={service.icon === 'favicon' ? 'Showing the site icon' : 'Showing initials'}
            onclick={() =>
              app.setServiceIcon(service.id, service.icon === 'favicon' ? 'initials' : 'favicon')}
          >
            {service.icon === 'favicon' ? 'Site icon' : 'Initials'}
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label="Reload {service.name}"
            title="Reload"
            disabled={!service.loaded}
            onclick={() => app.reloadService(service.id)}
          >
            <Icon name="reload" size={16} />
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label="Sign out of {service.name} and clear its data"
            title="Sign out and clear data"
            onclick={() => app.openDialog('confirm-erase', service.id)}
          >
            <Icon name="eraser" size={16} />
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label="Remove {service.name}"
            title="Remove"
            onclick={() => (confirming = service.id)}
          >
            <Icon name="trash" size={16} />
          </button>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  .list {
    margin: 0;
    padding: 0;
    border: 1px solid var(--line-subtle);
    border-radius: 10px;
    list-style: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 10px var(--space-4);
  }

  .row + .row {
    border-top: 1px solid var(--line-subtle);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-weight: 500;
  }

  .host,
  .empty,
  .memory {
    color: var(--text-faint);
  }

  .confirm-text {
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  .confirm-text {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .btn-small {
    height: 28px;
    padding: 0 10px;
    font-size: var(--text-sm);
  }

  .chip {
    height: 26px;
    padding: 0 10px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .chip:hover {
    border-color: var(--line-strong);
    color: var(--text);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    text-decoration: underline;
  }
</style>
