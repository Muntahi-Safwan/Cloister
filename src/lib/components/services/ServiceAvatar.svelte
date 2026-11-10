<!--
  @component
  A service's square icon: initials on a tile colour, or the site's own icon.
  Image sources are tried in order: the icon the page reported (`iconUrl`),
  the catalog icon (`fallbackUrl`), then `/favicon.ico`. If all fail to load,
  initials are shown.
-->
<script lang="ts">
  import { TILE_INDEXES } from '$lib/constants';
  import type { IconKind } from '$lib/types';
  import { initials } from '$lib/utils/format';
  import { faviconUrl } from '$lib/utils/url';

  interface Props {
    name: string;
    url: string;
    icon: IconKind;
    color: number;
    iconUrl?: string | null;
    fallbackUrl?: string | null;
    size?: number;
    dimmed?: boolean;
  }

  let {
    name,
    url,
    icon,
    color,
    iconUrl = null,
    fallbackUrl = null,
    size = 44,
    dimmed = false,
  }: Props = $props();

  const sources = $derived([
    ...new Set([iconUrl, fallbackUrl, faviconUrl(url)].filter((s): s is string => !!s)),
  ]);
  let failed = $state(0);
  const favicon = $derived(icon === 'favicon' ? (sources[failed] ?? null) : null);
  const monogram = $derived(initials(name));
  const slot = $derived(color % TILE_INDEXES.length);

  // New sources deserve a fresh attempt.
  $effect(() => {
    void sources;
    failed = 0;
  });
</script>

<span
  class="avatar"
  class:is-dimmed={dimmed}
  style:--size="{size}px"
  style:--tile-bg="var(--tile-{slot}-bg)"
  style:--tile-fg="var(--tile-{slot}-fg)"
  style:--letter-scale={monogram.length === 1 ? 0.4 : 0.3}
  aria-hidden="true"
>
  {#if favicon}
    <img
      src={favicon}
      alt=""
      width={Math.round(size * 0.62)}
      height={Math.round(size * 0.62)}
      draggable="false"
      onerror={() => (failed += 1)}
    />
  {:else}
    {monogram}
  {/if}
</span>

<style>
  .avatar {
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    width: var(--size);
    height: var(--size);
    border-radius: calc(var(--size) * 0.32);
    background: var(--tile-bg);
    color: var(--tile-fg);
    font-size: calc(var(--size) * var(--letter-scale));
    font-weight: 600;
  }

  .avatar.is-dimmed {
    color: var(--text-faint);
  }

  img {
    border-radius: 6px;
    object-fit: contain;
  }
</style>
