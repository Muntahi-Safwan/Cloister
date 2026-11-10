<!--
  @component
  Searchable list of built-in services with one-click add.
-->
<script lang="ts">
  import { DEFAULT_OPTIONS } from '$lib/constants';
  import { app } from '$lib/stores/app.svelte';
  import type { Recipe } from '$lib/types';

  import ServiceAvatar from './ServiceAvatar.svelte';

  let { oncustom }: { oncustom: () => void } = $props();

  let query = $state('');
  let adding = $state<string | null>(null);

  const added = $derived(new Set(app.snapshot.services.map((s) => s.recipeId).filter(Boolean)));
  const results = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return app.catalog;
    return app.catalog.filter((r) => `${r.name} ${r.category}`.toLowerCase().includes(needle));
  });

  /** "WhatsApp" the first time, "WhatsApp 2" for a second account, and so on. */
  function nameFor(recipe: Recipe): string {
    const existing = app.snapshot.services.filter((s) => s.recipeId === recipe.id).length;
    return existing === 0 ? recipe.name : `${recipe.name} ${existing + 1}`;
  }

  async function add(recipe: Recipe) {
    adding = recipe.id;
    await app.addService({
      name: nameFor(recipe),
      url: recipe.url,
      recipeId: recipe.id,
      icon: 'favicon',
      color: 0,
      options: { ...DEFAULT_OPTIONS, userAgent: recipe.userAgent },
    });
    adding = null;
  }
</script>

<div class="catalog">
  <label class="field">
    <span class="field-label">Search {app.catalog.length} ready-made services</span>
    <input
      class="input"
      type="search"
      name="catalog-search"
      autocomplete="off"
      placeholder="WhatsApp, Teams, Proton Mail…"
      bind:value={query}
      data-autofocus
    />
  </label>

  <ul class="list">
    {#each results as recipe (recipe.id)}
      <li class="row">
        <ServiceAvatar
          name={recipe.name}
          url={recipe.url}
          icon="favicon"
          color={0}
          fallbackUrl={app.catalogIcon(recipe)}
          size={34}
        />
        <span class="text">
          <span class="name">{recipe.name}</span>
          <span class="meta truncate">{recipe.category} · {new URL(recipe.url).host}</span>
        </span>
        <button
          type="button"
          class="btn"
          disabled={adding !== null}
          title={added.has(recipe.id) ? 'Add a second account, kept fully separate' : undefined}
          onclick={() => add(recipe)}
        >
          {adding === recipe.id ? 'Adding…' : added.has(recipe.id) ? 'Add another' : 'Add'}
        </button>
      </li>
    {:else}
      <li class="none">No service matches “{query}”.</li>
    {/each}
  </ul>

  <p class="meta">
    Not listed? <button type="button" class="link" onclick={oncustom}>Add any website</button> as a custom service.
  </p>
</div>

<style>
  .catalog {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5);
  }

  .list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0 var(--space-6);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--line-subtle);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-weight: 500;
  }

  .meta,
  .none {
    color: var(--text-faint);
    font-size: 12px;
  }

  .btn {
    height: 30px;
    padding: 0 var(--space-3);
    font-size: var(--text-sm);
  }

  .link {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-size: inherit;
    text-decoration: underline;
  }
</style>
