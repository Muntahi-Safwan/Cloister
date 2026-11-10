<!--
  @component
  A keyboard shortcut hint, e.g. `<Kbd keys="Ctrl K" />`.
-->
<script lang="ts">
  let { keys }: { keys: string } = $props();
</script>

<kbd class="mono">{keys}</kbd>

<style>
  kbd {
    padding: 2px 6px;
    border: 1px solid var(--line);
    border-radius: 5px;
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
  }
</style>
