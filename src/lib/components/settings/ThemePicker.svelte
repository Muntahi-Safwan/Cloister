<!--
  @component
  Theme choice with live previews. Each preview carries its own `data-theme`
  attribute, so it is drawn with that theme's real tokens.
-->
<script lang="ts">
  import Icon from '$lib/components/common/Icon.svelte';
  import { THEMES, type ThemeChoice, type ThemeId } from '$lib/theme';

  interface Props {
    value: ThemeChoice;
    onchange: (choice: ThemeChoice) => void;
  }

  let { value, onchange }: Props = $props();

  const OPTIONS: { id: ThemeChoice; name: string; description: string; previews: ThemeId[] }[] = [
    {
      id: 'system',
      name: 'Match Windows',
      description: 'Light or dark with your system',
      previews: ['paper', 'quiet-room'],
    },
    ...THEMES.map((t) => ({ id: t.id, name: t.name, description: t.description, previews: [t.id] })),
  ];
</script>

<div class="grid" role="radiogroup" aria-label="Theme">
  {#each OPTIONS as option (option.id)}
    <button
      type="button"
      role="radio"
      aria-checked={value === option.id}
      class="card"
      class:is-selected={value === option.id}
      onclick={() => onchange(option.id)}
    >
      <span class="preview" aria-hidden="true">
        {#each option.previews as theme (theme)}
          <span class="mini" data-theme={theme}>
            <span class="mini-rail">
              <span class="mini-tile is-active"></span>
              <span class="mini-tile"></span>
              <span class="mini-tile"></span>
            </span>
            <span class="mini-app">
              <span class="mini-line is-wide"></span>
              <span class="mini-line"></span>
              <span class="mini-button"></span>
            </span>
          </span>
        {/each}
      </span>
      <span class="label">
        <span class="name">{option.name}</span>
        <span class="description">{option.description}</span>
      </span>
      {#if value === option.id}
        <span class="check"><Icon name="check" size={12} strokeWidth={3} /></span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--space-3);
  }

  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    background: transparent;
    text-align: left;
    transition: border-color var(--duration-fast) var(--ease-out);
  }

  .card:hover {
    border-color: var(--line-strong);
  }

  .card.is-selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .preview {
    display: flex;
    height: 64px;
    overflow: hidden;
    border-radius: var(--radius-md);
  }

  .mini {
    flex: 1;
    display: flex;
    background: var(--bg-app);
  }

  .mini-rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 18px;
    padding-top: 6px;
    background: var(--bg-rail);
    border-right: 1px solid var(--line-subtle);
  }

  .mini-tile {
    width: 9px;
    height: 9px;
    border-radius: 3px;
    background: var(--bg-active);
  }

  .mini-tile.is-active {
    background: var(--accent);
  }

  .mini-app {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 8px;
  }

  .mini-line {
    width: 55%;
    height: 5px;
    border-radius: 3px;
    background: var(--text-faint);
    opacity: 0.6;
  }

  .mini-line.is-wide {
    width: 80%;
    background: var(--text);
    opacity: 0.85;
  }

  .mini-button {
    width: 28px;
    height: 10px;
    margin-top: auto;
    border-radius: 3px;
    background: var(--accent);
  }

  .label {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 0 2px 2px;
  }

  .name {
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .description {
    color: var(--text-faint);
    font-size: 11px;
  }

  .check {
    position: absolute;
    top: 6px;
    right: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--accent);
    color: var(--accent-ink);
  }
</style>
