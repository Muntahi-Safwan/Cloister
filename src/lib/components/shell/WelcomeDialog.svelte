<!--
  @component
  The welcome guide, shown once after the first services are added.

  Step 1 teaches the shortcuts that make Cloister different, and ticks each one
  off when the user tries it. Step 2 offers a PIN for shared computers. Closing
  it at any point counts as done, so it never comes back.
-->
<script lang="ts">
  import { errorMessage } from '$lib/api/backend';
  import Dialog from '$lib/components/common/Dialog.svelte';
  import Icon from '$lib/components/common/Icon.svelte';
  import Kbd from '$lib/components/common/Kbd.svelte';
  import Mark from '$lib/components/common/Mark.svelte';
  import { app } from '$lib/stores/app.svelte';
  import { SHORTCUT_LABELS } from '$lib/utils/shortcuts';

  let step = $state<'keys' | 'lock'>('keys');
  const steps = $derived(app.snapshot.hasPin ? 1 : 2);

  // Each key is ticked once the user has tried it, even if they turn it off again.
  let triedVeil = $state(false);
  let triedPresenting = $state(false);
  let triedFocus = $state(false);
  $effect(() => {
    if (app.snapshot.veil) triedVeil = true;
    if (app.snapshot.presenting) triedPresenting = true;
    if (app.snapshot.focus) triedFocus = true;
  });

  const keys = $derived([
    {
      id: 'veil',
      name: 'Veil',
      keys: SHORTCUT_LABELS.veil,
      text: 'Someone walks up behind you? Every chat blurs at once. Press it again to clear.',
      tried: triedVeil,
      now: app.snapshot.veil ? 'Veil is on' : null,
    },
    {
      id: 'present',
      name: 'Presenting',
      keys: SHORTCUT_LABELS.present,
      text: app.snapshot.canHideFromCapture
        ? 'Before a screen share. People watching see a blank window, and notifications wait.'
        : 'Before a screen share. Notifications wait until you are done.',
      tried: triedPresenting,
      now: app.snapshot.presenting ? 'Presenting' : null,
    },
    {
      id: 'focus',
      name: 'Focus',
      keys: SHORTCUT_LABELS.focus,
      text: `Holds notifications for ${app.snapshot.settings.focusMinutes} minutes, then shows you what came in.`,
      tried: triedFocus,
      now: app.snapshot.focus ? 'Focus is running' : null,
    },
  ]);

  // ------------------------------------------------------------------ step 2

  let pin = $state('');
  let confirm = $state('');
  let pinError = $state<string | null>(null);
  let saving = $state(false);

  function next() {
    if (steps === 1) void app.finishWelcome();
    else step = 'lock';
  }

  async function savePin(event: SubmitEvent) {
    event.preventDefault();
    pinError = null;
    if (pin !== confirm) {
      pinError = 'The two PINs do not match.';
      return;
    }
    saving = true;
    try {
      await app.setPin(null, pin);
      await app.updateSettings({ ...app.snapshot.settings, appLock: true, onboarded: true });
      app.closeDialog();
    } catch (error) {
      pinError = errorMessage(error);
    } finally {
      saving = false;
    }
  }
</script>

<Dialog label="Welcome to Cloister" width={560} showClose onclose={() => app.finishWelcome()}>
  <div class="panel">
    {#if steps > 1}
      <p class="count mono">{step === 'keys' ? 1 : 2} of {steps}</p>
    {/if}

    {#if step === 'keys'}
      <header>
        <Mark size={28} />
        <h2>Three keys worth knowing</h2>
        <p class="lead">They work everywhere in Cloister, even while you type in a chat. Try them now.</p>
      </header>

      <ul class="keys">
        {#each keys as key (key.id)}
          <li class:is-tried={key.tried}>
            <span class="tick" aria-hidden="true">
              {#if key.tried}<Icon name="check" size={13} strokeWidth={2.4} />{/if}
            </span>
            <span class="text">
              <span class="name">
                {key.name}
                {#if key.now}<span class="now">{key.now}</span>{/if}
              </span>
              <span class="hint">{key.text}</span>
            </span>
            <Kbd keys={key.keys} />
          </li>
        {/each}
      </ul>

      <p class="more">
        <Kbd keys={SHORTCUT_LABELS.palette} /> finds any service or command.
        <Kbd keys={SHORTCUT_LABELS.nextUnread} /> jumps to the next unread chat, and
        <Kbd keys={SHORTCUT_LABELS.previous} /> flips back to the last one.
      </p>

      <div class="buttons">
        <button type="button" class="btn btn-primary" data-autofocus onclick={next}>
          {steps === 1 ? 'Start using Cloister' : 'Next'}
        </button>
      </div>
    {:else}
      <header>
        <span class="icon" aria-hidden="true"><Icon name="lock" size={22} /></span>
        <h2>Does anyone else use this computer?</h2>
        <p class="lead">
          Set a Cloister PIN and it locks itself when you step away. It only unlocks Cloister and is separate
          from your computer's password.
        </p>
      </header>

      <form class="pin" onsubmit={savePin}>
        <input
          class="input"
          type="password"
          autocomplete="new-password"
          placeholder="New PIN (4 or more characters)"
          aria-label="New PIN"
          bind:value={pin}
          data-autofocus
        />
        <input
          class="input"
          type="password"
          autocomplete="new-password"
          placeholder="Type it again"
          aria-label="Confirm PIN"
          bind:value={confirm}
        />
        {#if pinError}<p class="error" role="alert">{pinError}</p>{/if}
        <div class="buttons">
          <button type="button" class="btn" onclick={() => app.finishWelcome()}>Skip for now</button>
          <button type="submit" class="btn btn-primary" disabled={saving || pin.length === 0}>
            {saving ? 'Saving…' : 'Set PIN and lock when away'}
          </button>
        </div>
      </form>
      <p class="later">You can change this any time in Settings, under Lock.</p>
    {/if}
  </div>
</Dialog>

<style>
  .panel {
    padding: var(--space-6);
  }

  .count {
    margin-bottom: var(--space-3);
    color: var(--text-faint);
    font-size: var(--text-xs);
  }

  header {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
  }

  h2 {
    margin-top: var(--space-2);
    font-size: 24px;
    font-weight: 600;
    line-height: 1.2;
  }

  .lead {
    color: var(--text-muted);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    color: var(--accent-text);
  }

  .keys {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: var(--space-5) 0 0;
    padding: 0;
    list-style: none;
  }

  .keys li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 10px 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-lg);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

  .keys li.is-tried {
    border-color: color-mix(in srgb, var(--accent) 35%, transparent);
    background: color-mix(in srgb, var(--accent) 7%, transparent);
  }

  .tick {
    display: grid;
    flex: none;
    place-items: center;
    width: 22px;
    height: 22px;
    border: 1.5px solid var(--line-strong);
    border-radius: 50%;
  }

  .is-tried .tick {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-ink);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 600;
  }

  .now {
    padding: 1px 7px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    color: var(--accent-text);
    font-size: 11px;
    font-weight: 600;
  }

  .hint,
  .later {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .more {
    margin-top: var(--space-4);
    padding-top: var(--space-4);
    border-top: 1px solid var(--line-subtle);
    color: var(--text-muted);
    font-size: var(--text-sm);
    line-height: 1.9;
  }

  .pin {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: var(--space-5);
  }

  .error {
    color: var(--danger);
    font-size: var(--text-sm);
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-5);
  }

  .later {
    margin-top: var(--space-3);
    font-size: var(--text-xs);
  }

  @media (prefers-reduced-motion: reduce) {
    .keys li {
      transition: none;
    }
  }
</style>
