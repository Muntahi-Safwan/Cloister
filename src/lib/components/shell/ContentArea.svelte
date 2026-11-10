<!--
  @component
  The region where service pages appear.

  Service pages are native child webviews positioned by the backend. This
  component measures its own box and reports it, so the backend can place
  them exactly here. It also renders what shows when no page is visible.
-->
<script lang="ts">
  import { backend } from '$lib/api/backend';
  import { app } from '$lib/stores/app.svelte';

  import EmptyState from './EmptyState.svelte';

  let area: HTMLElement | undefined = $state();

  function report() {
    if (!area) return;
    const rect = area.getBoundingClientRect();
    void backend.setContentBounds({ x: rect.left, y: rect.top, width: rect.width, height: rect.height });
  }

  $effect(() => {
    if (!area) return;
    const observer = new ResizeObserver(report);
    observer.observe(area);
    window.addEventListener('resize', report);
    report();
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', report);
    };
  });
</script>

<main bind:this={area} class="content">
  {#if app.ready && app.snapshot.services.length === 0}
    <EmptyState />
  {:else if app.activeService && !app.activeService.loaded}
    <div class="asleep">
      <p>{app.activeService.name} is asleep to save memory.</p>
      <button type="button" class="btn" onclick={() => app.activate(app.activeService!.id)}>Wake it up</button
      >
    </div>
  {:else if app.activeService}
    <p class="loading" aria-live="polite">Opening {app.activeService.name}…</p>
  {/if}
</main>

<style>
  .content {
    position: relative;
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-app);
  }

  .asleep {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    color: var(--text-muted);
  }

  .loading {
    color: var(--text-faint);
    font-size: var(--text-sm);
  }
</style>
