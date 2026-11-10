<!--
  @component
  Quiet hours: a daily window when notifications are silenced. Names in the
  "always let through" list still notify. Unread badges keep counting.
-->
<script lang="ts">
  import Switch from '$lib/components/common/Switch.svelte';
  import type { QuietHours } from '$lib/types';
  import { minutesToTime, timeToMinutes } from '$lib/utils/format';

  interface Props {
    value: QuietHours;
    onchange: (value: QuietHours) => void;
  }

  let { value, onchange }: Props = $props();

  function setTime(field: 'startMinute' | 'endMinute', raw: string) {
    const minutes = timeToMinutes(raw);
    if (minutes !== null) onchange({ ...value, [field]: minutes });
  }
</script>

<Switch
  label="Quiet hours"
  description="Silence notifications every day between these times. VIPs still get through."
  checked={value.enabled}
  onchange={(enabled) => onchange({ ...value, enabled })}
/>
{#if value.enabled}
  <div class="times">
    <label>
      <span>From</span>
      <input
        class="input"
        type="time"
        value={minutesToTime(value.startMinute)}
        onchange={(e) => setTime('startMinute', e.currentTarget.value)}
      />
    </label>
    <label>
      <span>Until</span>
      <input
        class="input"
        type="time"
        value={minutesToTime(value.endMinute)}
        onchange={(e) => setTime('endMinute', e.currentTarget.value)}
      />
    </label>
  </div>
{/if}

<style>
  .times {
    display: flex;
    gap: var(--space-4);
    padding: 0 0 var(--space-3);
  }

  label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .input {
    height: 32px;
    color-scheme: inherit;
  }
</style>
