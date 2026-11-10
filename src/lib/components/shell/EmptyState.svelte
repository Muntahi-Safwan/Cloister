<!--
  @component
  First-run screen: a short promise and one-click picks from the catalog.
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

  function add(recipe: Recipe) {
    void app.addService({
      name: recipe.name,
      url: recipe.url,
      recipeId: recipe.id,
      icon: 'favicon',
      color: 0,
      options: { ...DEFAULT_OPTIONS, userAgent: recipe.userAgent },
    });
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
  <p class="lead">Pick the services you use. Each one sleeps when you are not looking at it.</p>

  <div class="picks">
    {#each picks as recipe (recipe.id)}
      <button type="button" class="pick" onclick={() => add(recipe)}>
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
        <Icon name="plus" size={14} strokeWidth={2} />
      </button>
    {/each}
  </div>

  <button type="button" class="btn" onclick={() => app.openDialog('add-service')}>
    Browse all services or add a website
  </button>

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
