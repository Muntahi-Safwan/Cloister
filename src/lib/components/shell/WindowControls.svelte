<!--
  @component
  Minimise, maximise and close buttons for the frameless window.
  Closing hides Cloister to the tray when that setting is on (handled in Rust).
-->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  import Icon from '$lib/components/common/Icon.svelte';

  const appWindow = getCurrentWindow();
</script>

<div class="controls">
  <button type="button" aria-label="Minimize" onclick={() => appWindow.minimize()}>
    <Icon name="minimize" size={14} strokeWidth={1.4} />
  </button>
  <button type="button" aria-label="Maximize" onclick={() => appWindow.toggleMaximize()}>
    <Icon name="maximize" size={12} strokeWidth={1.4} />
  </button>
  <button type="button" class="close" aria-label="Close" onclick={() => appWindow.close()}>
    <Icon name="close" size={14} strokeWidth={1.4} />
  </button>
</div>

<style>
  .controls {
    display: flex;
    height: 100%;
  }

  button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--text-muted);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  button:hover {
    background: var(--bg-hover);
    color: var(--text);
  }

  .close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
