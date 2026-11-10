<!--
  @component
  Form for adding any website as a custom service, with a live tile preview.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import { DEFAULT_OPTIONS, TILE_INDEXES } from '$lib/constants';
  import { app } from '$lib/stores/app.svelte';
  import type { IconKind, ServiceOptions } from '$lib/types';
  import { checkServiceUrl, withHttps } from '$lib/utils/url';

  import ServiceAvatar from './ServiceAvatar.svelte';
  import ServiceOptionsFields from './ServiceOptionsFields.svelte';

  let { oncancel }: { oncancel: () => void } = $props();

  let address = $state('');
  let name = $state('');
  let icon = $state<IconKind>('initials');
  let color = $state(0);
  let options = $state<ServiceOptions>({ ...DEFAULT_OPTIONS, veilWholePage: true });
  let touched = $state(false);
  let saving = $state(false);

  const check = $derived(checkServiceUrl(withHttps(address)));
  const host = $derived(check.ok ? check.url.host : '');
  const displayName = $derived(name.trim() || host || 'New service');
  const showError = $derived(touched && address.trim() !== '' && !check.ok);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    touched = true;
    if (!check.ok) return;
    saving = true;
    await app.addService({
      name: displayName,
      url: check.url.toString(),
      recipeId: null,
      icon,
      color,
      options,
    });
    saving = false;
  }
</script>

<form class="form" onsubmit={submit} novalidate>
  <div class="fields">
    <label class="field">
      <span class="field-label">Website address</span>
      <input
        class="input mono"
        class:is-invalid={showError}
        type="url"
        name="url"
        inputmode="url"
        autocomplete="off"
        spellcheck="false"
        placeholder="https://chat.example.com…"
        aria-invalid={showError}
        aria-describedby="url-hint"
        bind:value={address}
        onblur={() => (touched = true)}
        data-autofocus
      />
      <span id="url-hint" class="field-hint" class:is-error={showError}>
        {#if showError && !check.ok}{check.reason}{:else}Paste the address you use in your browser.{/if}
      </span>
    </label>

    <label class="field">
      <span class="field-label">Name</span>
      <input
        class="input"
        type="text"
        name="service-name"
        autocomplete="off"
        placeholder={host || 'Company chat…'}
        bind:value={name}
      />
    </label>

    <fieldset class="field">
      <legend class="field-label">Icon</legend>
      <div class="chips">
        <button
          type="button"
          class="chip"
          aria-pressed={icon === 'initials'}
          onclick={() => (icon = 'initials')}
        >
          Initials
        </button>
        <button
          type="button"
          class="chip"
          aria-pressed={icon === 'favicon'}
          onclick={() => (icon = 'favicon')}
        >
          Site icon
        </button>
      </div>
      <div class="swatches">
        <span class="field-hint">Colour</span>
        {#each TILE_INDEXES as index (index)}
          <button
            type="button"
            class="swatch"
            style:background="var(--tile-{index}-bg)"
            aria-label="Tile colour {index + 1}"
            aria-pressed={color === index}
            onclick={() => (color = index)}
          ></button>
        {/each}
      </div>
    </fieldset>

    <div>
      <span class="field-label">Behaviour</span>
      <ServiceOptionsFields bind:options />
    </div>
  </div>

  <aside class="preview">
    <span class="field-hint">Preview</span>
    <div class="preview-card">
      <ServiceAvatar name={displayName} url={check.ok ? check.url.toString() : ''} {icon} {color} />
      <span class="preview-text">
        <span class="preview-name truncate">{displayName}</span>
        <span class="preview-host mono truncate">{host || 'no address yet'}</span>
      </span>
    </div>
    <ul class="facts">
      <li>
        <Icon name="lock" size={16} /><span
          >Gets its own private profile. Its cookies never touch your other services.</span
        >
      </li>
      <li>
        <Icon name="external" size={16} /><span>Links to other sites open in your default browser.</span>
      </li>
    </ul>
    <div class="buttons">
      <button type="button" class="btn" onclick={oncancel}>Cancel</button>
      <button type="submit" class="btn btn-primary" disabled={saving}>
        {saving ? 'Adding…' : `Add ${displayName}`}
      </button>
    </div>
  </aside>
</form>

<style>
  .form {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 280px;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding: var(--space-5) var(--space-6) var(--space-6);
    border-right: 1px solid var(--line-subtle);
  }

  fieldset {
    margin: 0;
    padding: 0;
    border: none;
  }

  .chips,
  .swatches {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .swatches {
    padding-top: var(--space-1);
  }

  .chip {
    height: 32px;
    padding: 0 var(--space-3);
    border: 1px solid var(--line);
    border-radius: 7px;
    background: transparent;
    font-size: var(--text-sm);
  }

  .chip[aria-pressed='true'] {
    border-color: color-mix(in srgb, var(--accent) 45%, var(--line));
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }

  .swatch {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 2px solid transparent;
    border-radius: 7px;
  }

  .swatch[aria-pressed='true'] {
    border-color: var(--accent);
  }

  .preview {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
    background: var(--bg-panel);
  }

  .preview-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-lg);
    background: var(--bg-rail);
  }

  .preview-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .preview-name {
    font-weight: 600;
  }

  .preview-host {
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  .facts {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .facts li {
    display: flex;
    gap: 10px;
  }

  .facts :global(svg) {
    flex: none;
    margin-top: 2px;
  }

  .buttons {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
    margin-top: auto;
  }
</style>
