/**
 * The app store: the single source of UI state.
 *
 * Backend state arrives as a `Snapshot` (pushed on every change) and is
 * stored as-is. UI-only state (which dialog is open, the focus clock) lives
 * alongside it. Components read from `app` and call its methods; they never
 * talk to the backend directly.
 */

import { backend, errorMessage } from '$lib/api/backend';
import { showServiceMenu } from '$lib/menus/serviceMenu';
import { reorderVisible } from '$lib/utils/order';
import { catalogIconUrl } from '$lib/utils/url';
import type {
  BackgroundMode,
  AppSettings,
  HeldNotification,
  HistoryItem,
  IconKind,
  MemoryReport,
  MuteDuration,
  NewService,
  Recipe,
  Service,
  ShortcutAction,
  Snapshot,
  ZoomStep,
} from '$lib/types';

/** Dialogs that cover the content area. Only one can be open at a time. */
export type DialogKind =
  | 'palette'
  | 'add-service'
  | 'settings'
  | 'focus-summary'
  | 'notifications'
  | 'rename'
  | 'confirm-remove'
  | 'confirm-erase';

const EMPTY_SNAPSHOT: Snapshot = {
  revision: 0,
  services: [],
  active: null,
  veil: false,
  focus: null,
  unreadTotal: 0,
  settings: {
    veilOnLaunch: false,
    redactToastsInVeil: true,
    vipNames: [],
    focusMinutes: 25,
    unloadAfterMinutes: 15,
    closeToTray: true,
    lowMemoryBackground: true,
    theme: 'quiet-room',
    quietHours: { enabled: false, startMinute: 23 * 60, endMinute: 8 * 60 },
    launchAtLogin: false,
    globalShortcuts: true,
    appLock: false,
    lockAfterMinutes: 5,
    veilOnBlur: false,
    veilAfterMinutes: 0,
    pauseCheckMinutes: 5,
    refreshOverMb: 800,
    checkForUpdates: true,
  },
  locked: false,
  hasPin: false,
  helloAvailable: false,
  historyCount: 0,
  historyLatestMs: null,
  canPause: false,
};

class AppStore {
  snapshot = $state<Snapshot>(EMPTY_SNAPSHOT);
  catalog = $state<Recipe[]>([]);
  dialog = $state<DialogKind | null>(null);
  /** The service a rename or remove dialog is about. */
  dialogTarget = $state<string | null>(null);
  heldNotifications = $state<HeldNotification[]>([]);
  /** Recent notifications, loaded when the notifications panel opens. */
  history = $state<HistoryItem[]>([]);
  /** Memory per service, refreshed while settings are open. */
  memory = $state<MemoryReport>({ services: {}, shared: 0 });
  /** Time the notifications panel was last opened; newer items light the bell. */
  historySeenMs = $state(0);
  /** The Windows Hello prompt is open, or a PIN is being checked. */
  unlocking = $state(false);
  /** Why the last unlock attempt failed, shown on the lock screen. */
  unlockError = $state<string | null>(null);
  error = $state<string | null>(null);
  ready = $state(false);
  now = $state(Date.now());

  #clock: ReturnType<typeof setInterval> | null = null;
  #endingFocus = false;

  /** Services shown in the sidebar (some hide during focus sessions). */
  visibleServices = $derived(
    this.snapshot.focus
      ? this.snapshot.services.filter((s) => !s.options.hideDuringFocus)
      : this.snapshot.services,
  );

  activeService = $derived<Service | null>(
    this.snapshot.services.find((s) => s.id === this.snapshot.active) ?? null,
  );

  hasUnseenHistory = $derived((this.snapshot.historyLatestMs ?? 0) > this.historySeenMs);

  focusRemainingMs = $derived(this.snapshot.focus ? this.snapshot.focus.endsAtMs - this.now : 0);

  /** Catalog icon for a recipe, or for a service created from one. */
  catalogIcon(source: Recipe | Service): string | null {
    const recipe = 'hosts' in source ? source : this.catalog.find((r) => r.id === source.recipeId);
    const domain = recipe?.hosts[0];
    return domain ? catalogIconUrl(domain) : null;
  }

  /** Loads initial state and subscribes to backend pushes. Call once at startup. */
  async init(): Promise<void> {
    await backend.onSnapshot((snapshot) => this.#apply(snapshot));
    await backend.onShortcut((action) => this.runAction(action));
    const [snapshot, catalog] = await Promise.all([backend.getSnapshot(), backend.listCatalog()]);
    this.catalog = catalog;
    this.#apply(snapshot);
    this.ready = true;

    const first = snapshot.active ?? snapshot.services[0]?.id;
    if (first) await this.activate(first);
  }

  // ---------------------------------------------------------------- services

  async activate(id: string): Promise<void> {
    await this.#run(() => backend.activateService(id));
  }

  async addService(input: NewService): Promise<boolean> {
    const ok = await this.#run(() => backend.addService(input));
    if (ok) this.closeDialog();
    return ok;
  }

  async removeService(id: string): Promise<void> {
    await this.#run(() => backend.removeService(id));
  }

  async reloadService(id: string): Promise<void> {
    await this.#run(() => backend.reloadService(id));
  }

  async setServiceIcon(id: string, icon: IconKind): Promise<void> {
    await this.#run(() => backend.setServiceIcon(id, icon));
  }

  /** Moves a visible service to `toIndex` among the visible services and saves the order. */
  async moveService(id: string, toIndex: number): Promise<void> {
    const order = reorderVisible(
      this.snapshot.services.map((s) => s.id),
      this.visibleServices.map((s) => s.id),
      id,
      toIndex,
    );
    if (order) await this.#run(() => backend.reorderServices(order));
  }

  async renameService(id: string, name: string): Promise<boolean> {
    const ok = await this.#run(() => backend.renameService(id, name));
    if (ok) this.closeDialog();
    return ok;
  }

  async muteService(id: string, duration: MuteDuration): Promise<void> {
    await this.#run(() => backend.muteService(id, duration));
  }

  async setServiceBackground(id: string, mode: BackgroundMode): Promise<void> {
    await this.#run(() => backend.setServiceBackground(id, mode));
  }

  async setServiceHidePreviews(id: string, hide: boolean): Promise<void> {
    await this.#run(() => backend.setServiceHidePreviews(id, hide));
  }

  /** Opens the find bar inside the active service. */
  async findInPage(): Promise<void> {
    if (!this.snapshot.active) return;
    this.dialog = null;
    await this.#run(() => backend.findInPage());
  }

  async sleepService(id: string): Promise<void> {
    await this.#run(() => backend.sleepService(id));
  }

  async openServiceExternally(id: string): Promise<void> {
    await this.#run(() => backend.openServiceExternally(id));
  }

  /** Zooms the active service in, out or back to 100 %. */
  async zoomActive(step: ZoomStep): Promise<void> {
    const id = this.snapshot.active;
    if (id) await this.#run(() => backend.zoomService(id, step));
  }

  /** Opens the native right-click menu for a service. */
  async openServiceMenu(service: Service): Promise<void> {
    try {
      await showServiceMenu(
        service,
        {
          open: (id) => void this.activate(id),
          rename: (id) => this.openDialog('rename', id),
          mute: (id, duration) => void this.muteService(id, duration),
          sleep: (id) => void this.sleepService(id),
          background: (id, mode) => void this.setServiceBackground(id, mode),
          hidePreviews: (id, hide) => void this.setServiceHidePreviews(id, hide),
          reload: (id) => void this.reloadService(id),
          openExternally: (id) => void this.openServiceExternally(id),
          copyLink: (url) => void navigator.clipboard.writeText(url).catch(() => undefined),
          erase: (id) => this.openDialog('confirm-erase', id),
          remove: (id) => this.openDialog('confirm-remove', id),
        },
        this.snapshot.canPause,
      );
    } catch (error) {
      this.error = errorMessage(error);
    }
  }

  /** Deletes one service's sign-in and cache. `null` erases every service. */
  async eraseData(id: string | null): Promise<void> {
    this.closeDialog();
    await this.#run(() => (id ? backend.eraseServiceData(id) : backend.eraseAllData()));
    this.memory = { services: {}, shared: 0 };
  }

  /** Re-reads how much memory each loaded service uses. */
  async refreshMemory(): Promise<void> {
    try {
      this.memory = await backend.serviceMemory();
    } catch (error) {
      console.warn('Could not read memory use', error);
    }
  }

  async sendTestNotification(): Promise<void> {
    await this.#run(() => backend.sendTestNotification());
  }

  /** Activates the n-th visible service (1-based), as used by Ctrl+1..9. */
  async switchTo(position: number): Promise<void> {
    const service = this.visibleServices[position - 1];
    if (service) await this.activate(service.id);
  }

  // --------------------------------------------------------- veil and focus

  async toggleVeil(): Promise<void> {
    await this.#run(() => backend.setVeil(!this.snapshot.veil));
  }

  async startFocus(minutes?: number): Promise<void> {
    await this.#run(() => backend.startFocus(minutes));
  }

  /** Ends the focus session and shows what was held. Safe to call more than once. */
  async endFocus(): Promise<void> {
    if (this.#endingFocus) return;
    this.#endingFocus = true;
    try {
      this.heldNotifications = await backend.endFocus();
      this.openDialog('focus-summary');
    } catch (error) {
      this.error = errorMessage(error);
    } finally {
      this.#endingFocus = false;
    }
  }

  async toggleFocus(): Promise<void> {
    if (this.snapshot.focus) await this.endFocus();
    else await this.startFocus();
  }

  // -------------------------------------------------------------- app lock

  async lock(): Promise<void> {
    if (!this.snapshot.hasPin) {
      this.error = 'Set a Cloister PIN in Settings › Lock first, so you can always unlock.';
      return;
    }
    this.unlockError = null;
    await this.#run(() => backend.lockNow());
  }

  /** Shows the Windows Hello prompt. Returns quietly if the user cancels it. */
  async unlock(): Promise<void> {
    if (this.unlocking) return;
    this.unlocking = true;
    try {
      await backend.unlock();
      this.unlockError = null;
    } catch (error) {
      this.unlockError = errorMessage(error);
    } finally {
      this.unlocking = false;
    }
  }

  /** Unlocks with the Cloister PIN. Returns whether it worked. */
  async unlockWithPin(pin: string): Promise<boolean> {
    if (this.unlocking) return false;
    this.unlocking = true;
    try {
      const ok = await backend.unlockWithPin(pin);
      this.unlockError = ok ? null : 'That PIN is not right.';
      return ok;
    } catch (error) {
      this.unlockError = errorMessage(error);
      return false;
    } finally {
      this.unlocking = false;
    }
  }

  /** Sets or changes the PIN. Throws with a message for the user on failure. */
  async setPin(current: string | null, pin: string): Promise<void> {
    this.#apply(await backend.setPin(current, pin));
  }

  /** Removes the PIN (and with it app lock). Throws with a message on failure. */
  async removePin(current: string): Promise<void> {
    this.#apply(await backend.removePin(current));
  }

  // --------------------------------------------------------- notifications

  async openNotifications(): Promise<void> {
    this.openDialog('notifications');
    this.historySeenMs = Date.now();
    try {
      this.history = await backend.getHistory();
    } catch (error) {
      this.error = errorMessage(error);
    }
  }

  async clearHistory(): Promise<void> {
    if (await this.#run(() => backend.clearHistory())) this.history = [];
  }

  async updateSettings(settings: AppSettings): Promise<void> {
    await this.#run(() => backend.updateSettings(settings));
  }

  // ------------------------------------------------------------------ dialogs

  openDialog(kind: DialogKind, target: string | null = null): void {
    this.dialog = kind;
    this.dialogTarget = target;
  }

  closeDialog(): void {
    this.dialog = null;
    this.dialogTarget = null;
  }

  /** Runs a shortcut, tray or palette action. */
  async runAction(action: ShortcutAction): Promise<void> {
    if (action === 'palette') this.dialog = this.dialog === 'palette' ? null : 'palette';
    else if (action === 'settings') this.openDialog('settings');
    else if (action === 'veil') await this.toggleVeil();
    else if (action === 'focus') await this.toggleFocus();
    else if (action === 'lock') await this.lock();
    else if (action === 'find') await this.findInPage();
    else if (action.startsWith('zoom:')) await this.zoomActive(action.slice('zoom:'.length) as ZoomStep);
    else await this.switchTo(Number(action.slice('switch:'.length)));
  }

  dismissError(): void {
    this.error = null;
  }

  // ------------------------------------------------------------------ private

  /** Applies a snapshot unless a newer one has already arrived (pushes and replies can interleave). */
  #apply(snapshot: Snapshot): void {
    if (snapshot.revision < this.snapshot.revision) return;
    // Locking closes any open dialog so nothing is left on screen behind the lock.
    if (snapshot.locked && !this.snapshot.locked) this.closeDialog();
    this.snapshot = snapshot;
    this.#syncClock();
    // Timers are throttled while the window is hidden, so the backend also flags expiry.
    if (snapshot.focus?.expired) void this.endFocus();
  }

  /** Ticks once a second while a focus session runs, and ends it on time. */
  #syncClock(): void {
    const running = this.snapshot.focus !== null;
    if (running && !this.#clock) {
      this.now = Date.now();
      this.#clock = setInterval(() => {
        this.now = Date.now();
        if (this.snapshot.focus && this.now >= this.snapshot.focus.endsAtMs) void this.endFocus();
      }, 1000);
    } else if (!running && this.#clock) {
      clearInterval(this.#clock);
      this.#clock = null;
    }
  }

  /** Runs a backend call that returns a snapshot, recording any error. */
  async #run(call: () => Promise<Snapshot | void>): Promise<boolean> {
    try {
      const result = await call();
      if (result) this.#apply(result);
      this.error = null;
      return true;
    } catch (error) {
      this.error = errorMessage(error);
      return false;
    }
  }
}

export const app = new AppStore();
