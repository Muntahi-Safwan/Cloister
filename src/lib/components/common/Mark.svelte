<!--
  @component
  The Cloister mark: an arched doorway with a light inside.
-->
<script lang="ts">
  let { size = 16 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
  <path
    d="M5.5 20.5V10.5a6.5 6.5 0 0 1 13 0v10"
    fill="none"
    stroke="var(--text)"
    stroke-width="2.4"
    stroke-linecap="round"
  />
  <path d="M3.5 20.5h17" stroke="var(--text)" stroke-width="2.4" stroke-linecap="round" />
  <circle cx="12" cy="15.5" r="2.5" fill="var(--accent)" />
</svg>
