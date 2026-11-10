<!--
  @component
  About Cloister: version, author and licence.
-->
<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';

  import Mark from '$lib/components/common/Mark.svelte';

  let version = $state('');

  $effect(() => {
    getVersion()
      .then((v) => (version = v))
      .catch(() => (version = ''));
  });
</script>

<section class="about" aria-labelledby="about-title">
  <Mark size={36} />
  <div class="text">
    <h3 id="about-title">Cloister {version}</h3>
    <p>Designed and built by <strong>Muntahi Safwan</strong>.</p>
    <p class="fine">Open source under the MIT License. Not affiliated with any of the services it opens.</p>
  </div>
</section>

<style>
  .about {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-5);
    border: 1px solid var(--line-subtle);
    border-radius: var(--radius-lg);
    background: var(--bg-panel);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  h3 {
    font-size: var(--text-lg);
    font-weight: 600;
  }

  p {
    color: var(--text-muted);
  }

  strong {
    color: var(--text);
    font-weight: 600;
  }

  .fine {
    color: var(--text-faint);
    font-size: 12px;
  }
</style>
