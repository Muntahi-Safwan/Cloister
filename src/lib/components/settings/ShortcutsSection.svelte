<!--
  @component
  Keyboard shortcuts: the system-wide ones (with an on/off switch) and the
  in-app ones for reference.
-->
<script lang="ts">
  import Kbd from '$lib/components/common/Kbd.svelte';
  import Switch from '$lib/components/common/Switch.svelte';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  interface Props {
    enabled: boolean;
    onchange: (enabled: boolean) => void;
  }

  let { enabled, onchange }: Props = $props();

  const IN_APP = [
    ['Command palette', SHORTCUT_LABELS.palette],
    ['Switch to service 1 to 9', SHORTCUT_LABELS.switchTo],
    ['Find in page', SHORTCUT_LABELS.find],
    ['Toggle Veil', SHORTCUT_LABELS.veil],
    ['Start or end focus', SHORTCUT_LABELS.focus],
    ['Lock Cloister', SHORTCUT_LABELS.lock],
    ['Zoom in, out, reset', SHORTCUT_LABELS.zoom],
    ['Move a service in the sidebar', 'Alt ↑ / ↓'],
  ] as const;
</script>

<Switch
  label="System-wide shortcuts"
  description="Work even when Cloister is in the background"
  checked={enabled}
  {onchange}
/>
<ul class="list" class:is-off={!enabled}>
  <li><span>Show or hide Cloister</span><Kbd keys={SHORTCUT_LABELS.showHide} /></li>
  <li><span>Panic: turn on Veil and hide instantly</span><Kbd keys={SHORTCUT_LABELS.panic} /></li>
</ul>
<ul class="list">
  {#each IN_APP as [label, keys] (label)}
    <li><span>{label}</span><Kbd {keys} /></li>
  {/each}
</ul>

<style>
  .list {
    margin: 0 0 var(--space-2);
    padding: 0;
    list-style: none;
  }

  .list.is-off {
    opacity: 0.5;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    border-top: 1px solid var(--line-subtle);
    color: var(--text-muted);
    font-size: var(--text-sm);
  }
</style>
