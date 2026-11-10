<!--
  @component
  An accessible on/off switch with a title and optional description.
-->
<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    description?: string;
    disabled?: boolean;
    onchange: (checked: boolean) => void;
  }

  let { checked, label, description, disabled = false, onchange }: Props = $props();
</script>

<div class="row">
  <div class="text">
    <span class="label">{label}</span>
    {#if description}<span class="description">{description}</span>{/if}
  </div>
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    class="track"
    class:is-on={checked}
    {disabled}
    onclick={() => onchange(!checked)}
  >
    <span class="knob"></span>
  </button>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: 11px 0;
    border-top: 1px solid var(--line-subtle);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label {
    font-size: var(--text-md);
  }

  .description {
    font-size: 12px;
    color: var(--text-faint);
  }

  .track {
    flex: none;
    width: 40px;
    height: 24px;
    padding: 3px;
    border: none;
    border-radius: 12px;
    background: var(--line);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .track.is-on {
    background: var(--accent);
  }

  .knob {
    display: block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--text-faint);
    transition: transform var(--duration) var(--ease-out);
  }

  .is-on .knob {
    background: var(--accent-ink);
    transform: translateX(16px);
  }

  .track:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
