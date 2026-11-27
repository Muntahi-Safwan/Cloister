<!--
  @component
  First-run screen: pick the services you use (several at once), then a short
  welcome guide opens (see `WelcomeDialog.svelte`).
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import Mark from '$lib/components/common/Mark.svelte';
  import { DEFAULT_OPTIONS } from '$lib/constants';
  import { app } from '$lib/stores/app.svelte';
  import type { Recipe } from '$lib/types';

  import ServiceAvatar from '$lib/components/services/ServiceAvatar.svelte';

  const QUICK_PICKS = ['whatsapp', 'discord', 'gmail', 'messenger', 'telegram', 'outlook'];
  const picks = $derived(app.catalog.filter((r) => QUICK_PICKS.includes(r.id)));

  /** Recipe ids picked so far, in the order they were picked. */
  let chosen = $state<string[]>([]);
  let adding = $state(false);

  function toggle(recipe: Recipe) {
    chosen = chosen.includes(recipe.id) ? chosen.filter((id) => id !== recipe.id) : [...chosen, recipe.id];
  }

  async function addChosen() {
    adding = true;
    const recipes = chosen.map((id) => picks.find((r) => r.id === id)).filter((r): r is Recipe => !!r);
    await app.addServices(
      recipes.map((recipe) => ({
        name: recipe.name,
        url: recipe.url,
        recipeId: recipe.id,
        icon: 'favicon',
        color: 0,
        options: { ...DEFAULT_OPTIONS, userAgent: recipe.userAgent },
      })),
    );
    adding = false;
  }

  const PROMISES = [
    ['Local only.', 'No account and no server of ours between you and your messages.'],
    ['Isolated.', 'Every service gets its own private browser profile.'],
    ['Open source.', 'Every line that touches your sessions can be read and audited.'],
  ] as const;
</script>

<section class="empty">
  <Mark size={40} />
  <h1>One quiet window for every conversation.</h1>
  <p class="lead">Pick the ones you use. You sign in to each inside Cloister, just like in your browser.</p>

  <div class="picks">
    {#each picks as recipe (recipe.id)}
      {@const on = chosen.includes(recipe.id)}
      <button type="button" class="pick" class:is-on={on} aria-pressed={on} onclick={() => toggle(recipe)}>
        <ServiceAvatar
          name={recipe.name}
          url={recipe.url}
          icon="favicon"
          color={0}
          fallbackUrl={app.catalogIcon(recipe)}
          size={32}
        />
        <span class="pick-text">
          <span class="pick-name">{recipe.name}</span>
          <span class="pick-kind">{recipe.category}</span>
        </span>
        <span class="tick" aria-hidden="true">
          <Icon name={on ? 'check' : 'plus'} size={13} strokeWidth={2.2} />
        </span>
      </button>
    {/each}
  </div>

  <div class="row">
    <button
      type="button"
      class="btn btn-primary"
      disabled={chosen.length === 0 || adding}
      onclick={addChosen}
    >
      {#if adding}
        Adding…
      {:else if chosen.length === 0}
        Choose at least one
      {:else}
        Add {chosen.length} {chosen.length === 1 ? 'service' : 'services'}
      {/if}
    </button>
    <button type="button" class="btn" onclick={() => app.openDialog('add-service')}>
      Browse all services or add a website
    </button>
  </div>

  <ul class="promises">
    {#each PROMISES as [title, text] (title)}
      <li><Icon name="check" size={14} strokeWidth={2.2} /><span><strong>{title}</strong> {text}</span></li>
    {/each}
  </ul>

  <p class="credit">Cloister · Made by Muntahi Safwan</p>
</section>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-4);
    width: min(640px, 100% - 64px);
  }

  h1 {
    margin-top: var(--space-3);
    font-size: 32px;
    font-weight: 600;
    line-height: 1.2;
    text-wrap: balance;
  }

  .lead {
    color: var(--text-muted);
    font-size: var(--text-lg);
  }

  .picks {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
    width: 100%;
    margin-top: var(--space-3);
  }

  .pick {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 10px 12px;
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    background: var(--bg-panel);
    color: var(--text-faint);
    text-align: left;
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  .pick:hover {
    border-color: var(--line-strong);
    color: var(--text);
  }

  .pick.is-on {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, var(--bg-panel));
  }

  .tick {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    transition: background var(--duration-fast) var(--ease-out);
  }

  .is-on .tick {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .pick-text {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .pick-name {
    color: var(--text);
    font-weight: 600;
  }

  .pick-kind {
    font-size: var(--text-xs);
  }

  .promises {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: var(--space-5) 0 0;
    padding: var(--space-5) 0 0;
    border-top: 1px solid var(--line-subtle);
    list-style: none;
    width: 100%;
    color: var(--text-muted);
  }

  .promises li {
    display: flex;
    gap: var(--space-3);
    align-items: baseline;
  }

  .promises strong {
    color: var(--text);
  }

  .credit {
    color: var(--text-faint);
    font-size: var(--text-xs);
  }
</style>
