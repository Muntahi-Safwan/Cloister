<!--
  @component
  Behaviour switches for a service. Used by the custom website form.
-->
<script lang="ts">
  import Switch from '$lib/components/common/Switch.svelte';
  import type { BackgroundMode, ServiceOptions, UserAgentPreset } from '$lib/types';

  let { options = $bindable() }: { options: ServiceOptions } = $props();
</script>

<div class="options">
  <Switch
    label="Show notifications"
    description="The site's notifications appear as Windows toasts"
    checked={options.notifications}
    onchange={(v) => (options.notifications = v)}
  />
  <Switch
    label="Count unread from the page title"
    description="Works for most sites that show “(3)” in their tab title"
    checked={options.unreadFromTitle}
    onchange={(v) => (options.unreadFromTitle = v)}
  />
  <Switch
    label="Blur the whole page in Veil"
    description="Custom sites have no recipe, so Veil blurs everything"
    checked={options.veilWholePage}
    onchange={(v) => (options.veilWholePage = v)}
  />
  <Switch
    label="Hide during focus sessions"
    description="Marks this service as distracting"
    checked={options.hideDuringFocus}
    onchange={(v) => (options.hideDuringFocus = v)}
  />

  <div class="select-row">
    <label for="background-mode">
      <span>When in the background</span>
      <span class="hint">Unloading frees the most memory; the page reloads when opened</span>
    </label>
    <select
      id="background-mode"
      class="select"
      value={options.background}
      onchange={(e) => (options.background = e.currentTarget.value as BackgroundMode)}
    >
      <option value="keepNotifying">Keep notifying</option>
      <option value="unloadWhenIdle">Unload when idle</option>
    </select>
  </div>

  <div class="select-row">
    <label for="user-agent">
      <span>Browser identity</span>
      <span class="hint">Change only if the site says your browser isn’t supported</span>
    </label>
    <select
      id="user-agent"
      class="select"
      value={options.userAgent}
      onchange={(e) => (options.userAgent = e.currentTarget.value as UserAgentPreset)}
    >
      <option value="default">Default (Edge)</option>
      <option value="chrome">Chrome</option>
      <option value="firefox">Firefox</option>
    </select>
  </div>
</div>

<style>
  .select-row {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: 11px 0;
    border-top: 1px solid var(--line-subtle);
  }

  label {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .hint {
    font-size: 12px;
    color: var(--text-faint);
  }

  .select {
    width: 180px;
  }
</style>
