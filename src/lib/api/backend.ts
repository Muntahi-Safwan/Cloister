/**
 * Typed wrappers around the backend commands and events.
 *
 * Components never call `invoke` directly; everything goes through this
 * module so command names and payload shapes live in one place.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type {
  BackgroundMode,
  AppSettings,
  ContentBounds,
  HeldNotification,
  HistoryItem,
  IconKind,
  MemoryReport,
  MuteDuration,
  NewService,
  Recipe,
  ShortcutAction,
  Snapshot,
  ZoomStep,
} from '$lib/types';

const SNAPSHOT_EVENT = 'cloister://snapshot';
const SHORTCUT_EVENT = 'cloister://shortcut';

export const backend = {
  getSnapshot: () => invoke<Snapshot>('get_snapshot'),
  listCatalog: () => invoke<Recipe[]>('list_catalog'),

  addService: (input: NewService) => invoke<Snapshot>('add_service', { input }),
  removeService: (id: string) => invoke<Snapshot>('remove_service', { id }),
  reorderServices: (ids: string[]) => invoke<Snapshot>('reorder_services', { ids }),
  activateService: (id: string) => invoke<Snapshot>('activate_service', { id }),
  reloadService: (id: string) => invoke<void>('reload_service', { id }),
  setServiceIcon: (id: string, icon: IconKind) => invoke<Snapshot>('set_service_icon', { id, icon }),
  renameService: (id: string, name: string) => invoke<Snapshot>('rename_service', { id, name }),
  muteService: (id: string, duration: MuteDuration) => invoke<Snapshot>('mute_service', { id, duration }),
  sleepService: (id: string) => invoke<Snapshot>('sleep_service', { id }),
  setServiceBackground: (id: string, mode: BackgroundMode) =>
    invoke<Snapshot>('set_service_background', { id, mode }),
  zoomService: (id: string, step: ZoomStep) => invoke<Snapshot>('zoom_service', { id, step }),
  openServiceExternally: (id: string) => invoke<void>('open_service_externally', { id }),

  setContentBounds: (bounds: ContentBounds) => invoke<void>('set_content_bounds', { bounds }),
  setOverlayOpen: (open: boolean) => invoke<void>('set_overlay_open', { open }),

  setVeil: (on: boolean) => invoke<Snapshot>('set_veil', { on }),
  startFocus: (minutes?: number) => invoke<Snapshot>('start_focus', { minutes }),
  endFocus: () => invoke<HeldNotification[]>('end_focus'),
  updateSettings: (settings: AppSettings) => invoke<Snapshot>('update_settings', { settings }),
  sendTestNotification: () => invoke<void>('send_test_notification'),

  lockNow: () => invoke<Snapshot>('lock_now'),
  /** Shows the Windows Hello prompt; resolves to whether Cloister unlocked. */
  unlock: () => invoke<boolean>('unlock'),
  /** Checks the Cloister PIN; resolves to whether Cloister unlocked. */
  unlockWithPin: (pin: string) => invoke<boolean>('unlock_with_pin', { pin }),
  /** Sets or changes the Cloister PIN. `current` is needed when one is set. */
  setPin: (current: string | null, pin: string) => invoke<Snapshot>('set_pin', { current, pin }),
  /** Removes the Cloister PIN and turns app lock off. */
  removePin: (current: string) => invoke<Snapshot>('remove_pin', { current }),
  getHistory: () => invoke<HistoryItem[]>('get_history'),
  clearHistory: () => invoke<Snapshot>('clear_history'),
  serviceMemory: () => invoke<MemoryReport>('service_memory'),
  eraseServiceData: (id: string) => invoke<Snapshot>('erase_service_data', { id }),
  eraseAllData: () => invoke<Snapshot>('erase_all_data'),

  onSnapshot: (handler: (snapshot: Snapshot) => void): Promise<UnlistenFn> =>
    listen<Snapshot>(SNAPSHOT_EVENT, (event) => handler(event.payload)),
  onShortcut: (handler: (action: ShortcutAction) => void): Promise<UnlistenFn> =>
    listen<ShortcutAction>(SHORTCUT_EVENT, (event) => handler(event.payload)),
};

/** Turns any thrown value into a message suitable for the UI. */
export function errorMessage(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return 'Something went wrong. Please try again.';
}
