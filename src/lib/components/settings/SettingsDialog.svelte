<!--
  @component
  Settings: appearance, privacy, focus, notifications, general behaviour, services and about.
  Every change is saved immediately.
-->
<script lang="ts">
  import Dialog from '$lib/components/common/Dialog.svelte';
  import Switch from '$lib/components/common/Switch.svelte';
  import { app } from '$lib/stores/app.svelte';
  import type { AppSettings } from '$lib/types';

  import AboutSection from './AboutSection.svelte';
  import LockSection from './LockSection.svelte';
  import QuietHoursField from './QuietHoursField.svelte';
  import ServiceList from './ServiceList.svelte';
  import ShortcutsSection from './ShortcutsSection.svelte';
  import ThemePicker from './ThemePicker.svelte';

  const settings = $derived(app.snapshot.settings);
  let vipDraft = $state(app.snapshot.settings.vipNames.join(', '));

  // Keep the memory figures fresh while settings are open.
  $effect(() => {
    void app.refreshMemory();
    const timer = setInterval(() => void app.refreshMemory(), 5000);
    return () => clearInterval(timer);
  });

  function save(patch: Partial<AppSettings>) {
    void app.updateSettings({ ...settings, ...patch });
  }

  function saveVips() {
    const vipNames = vipDraft
      .split(',')
      .map((name) => name.trim())
      .filter(Boolean);
    save({ vipNames });
  }
</script>

<Dialog label="Settings" width={680} showClose onclose={() => app.closeDialog()}>
  <div class="settings">
    <header>
      <h2>Settings</h2>
      <p class="lead">
        Everything stays on this computer. Cloister has no account, no telemetry and no server of its own.
      </p>
    </header>

    <section>
      <h3>Appearance</h3>
      <ThemePicker value={settings.theme} onchange={(theme) => save({ theme })} />
    </section>

    <section>
      <h3>Veil</h3>
      <Switch
        label="Start every session veiled"
        description="Cloister opens with Veil already on"
        checked={settings.veilOnLaunch}
        onchange={(veilOnLaunch) => save({ veilOnLaunch })}
      />
      <Switch
        label="Hide message text in notifications"
        description="While Veil is on, toasts name the service but never who or what"
        checked={settings.redactToastsInVeil}
        onchange={(redactToastsInVeil) => save({ redactToastsInVeil })}
      />
    </section>

    <section>
      <h3>Lock</h3>
      <LockSection {settings} authAvailable={app.snapshot.authAvailable} onchange={save} />
    </section>

    <section>
      <h3>Focus</h3>
      <div class="row">
        <label for="focus-length" class="row-text">
          <span>Session length</span>
          <span class="hint">Notifications are held until the session ends</span>
        </label>
        <select
          id="focus-length"
          class="select"
          value={settings.focusMinutes}
          onchange={(e) => save({ focusMinutes: Number(e.currentTarget.value) })}
        >
          {#each [15, 25, 50, 90] as minutes (minutes)}
            <option value={minutes}>{minutes} minutes</option>
          {/each}
        </select>
      </div>
      <div class="row column">
        <label for="vip-names" class="row-text">
          <span>Always let through</span>
          <span class="hint"
            >Names separated by commas. Notifications whose title contains one still show.</span
          >
        </label>
        <input
          id="vip-names"
          class="input"
          type="text"
          name="vip-names"
          autocomplete="off"
          placeholder="Mom, Client team…"
          bind:value={vipDraft}
          onblur={saveVips}
        />
      </div>
    </section>

    <section>
      <h3>Notifications</h3>
      <QuietHoursField value={settings.quietHours} onchange={(quietHours) => save({ quietHours })} />
      <div class="row">
        <div class="row-text">
          <span>Test Windows notifications</span>
          <span class="hint">
            If nothing appears, check Windows Settings › System › Notifications and turn off Do not disturb.
          </span>
        </div>
        <button type="button" class="btn" onclick={() => app.sendTestNotification()}>Send test</button>
      </div>
    </section>

    <section>
      <h3>General</h3>
      <Switch
        label="Start with Windows"
        description="Cloister opens quietly in the tray when you sign in"
        checked={settings.launchAtLogin}
        onchange={(launchAtLogin) => save({ launchAtLogin })}
      />
      <Switch
        label="Keep running in the tray when closed"
        description="Notifications keep arriving after you close the window"
        checked={settings.closeToTray}
        onchange={(closeToTray) => save({ closeToTray })}
      />
      <Switch
        label="Use less memory for background pages"
        description="Pages you are not looking at give memory back to Windows and still notify you"
        checked={settings.lowMemoryBackground}
        onchange={(lowMemoryBackground) => save({ lowMemoryBackground })}
      />
      <div class="row">
        <label for="unload-after" class="row-text">
          <span>Unload idle services after</span>
          <span class="hint">Applies to services set to “Unload when idle”</span>
        </label>
        <select
          id="unload-after"
          class="select"
          value={settings.unloadAfterMinutes}
          onchange={(e) => save({ unloadAfterMinutes: Number(e.currentTarget.value) })}
        >
          {#each [5, 15, 30, 60] as minutes (minutes)}
            <option value={minutes}>{minutes} minutes</option>
          {/each}
        </select>
      </div>
    </section>

    <section>
      <h3>Shortcuts</h3>
      <ShortcutsSection
        enabled={settings.globalShortcuts}
        onchange={(globalShortcuts) => save({ globalShortcuts })}
      />
    </section>

    <section>
      <h3>Services</h3>
      <ServiceList />
      <div class="row">
        <div class="row-text">
          <span>Sign out of everything</span>
          <span class="hint">Deletes every service's sign-in, cookies and cache from this computer</span>
        </div>
        <button type="button" class="btn btn-danger" onclick={() => app.openDialog('confirm-erase')}>
          Clear all data…
        </button>
      </div>
      <p class="note">
        When a site asks for your camera or microphone, Windows shows its own prompt and you decide.
      </p>
    </section>

    <AboutSection />
  </div>
</Dialog>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding: var(--space-6) var(--space-6) var(--space-6);
  }

  h2 {
    font-size: 22px;
    font-weight: 600;
  }

  .lead {
    margin-top: var(--space-2);
    color: var(--text-muted);
  }

  h3 {
    margin-bottom: var(--space-2);
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: 11px 0;
    border-top: 1px solid var(--line-subtle);
  }

  .row.column {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-2);
  }

  .row-text {
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
    width: 150px;
  }

  .note {
    margin-top: var(--space-2);
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
