/**
 * Types shared with the Rust backend.
 *
 * These mirror the serde structs in `src-tauri/src` (camelCase on the wire).
 * Keep them in sync when changing a backend type.
 */

import type { ThemeChoice } from '$lib/theme';

export type { ThemeChoice };

export type IconKind = 'initials' | 'favicon';
export type BackgroundMode = 'keepNotifying' | 'unloadWhenIdle' | 'pause';
export type UserAgentPreset = 'default' | 'chrome' | 'firefox';
export type UnreadRule = 'titleParentheses' | 'none';

export interface ServiceOptions {
  notifications: boolean;
  unreadFromTitle: boolean;
  veilWholePage: boolean;
  hideDuringFocus: boolean;
  background: BackgroundMode;
  userAgent: UserAgentPreset;
}

/** A configured service plus its live status (`ServiceView` in Rust). */
export interface Service {
  id: string;
  name: string;
  url: string;
  recipeId: string | null;
  icon: IconKind;
  color: number;
  /** The site's own icon, reported by the page after it loads. */
  iconUrl: string | null;
  /** Notifications are muted until this Unix time in ms (`Number.MAX_SAFE_INTEGER`-ish = forever). */
  mutedUntil: number | null;
  /** Page zoom factor, 1 = 100 %. */
  zoom: number;
  options: ServiceOptions;
  unread: number;
  loaded: boolean;
}

/** Input for `add_service`. */
export interface NewService {
  name: string;
  url: string;
  recipeId?: string | null;
  icon: IconKind;
  color: number;
  options: ServiceOptions;
}

/** A built-in catalog entry. */
export interface Recipe {
  id: string;
  name: string;
  category: string;
  url: string;
  hosts: string[];
  unread: UnreadRule;
  veilSelectors: string[];
  userAgent: UserAgentPreset;
}

export interface QuietHours {
  enabled: boolean;
  /** Minutes after local midnight. */
  startMinute: number;
  endMinute: number;
}

export type MuteDuration = 'oneHour' | 'untilTomorrow' | 'forever' | 'off';
export type ZoomStep = 'in' | 'out' | 'reset';

export interface AppSettings {
  veilOnLaunch: boolean;
  redactToastsInVeil: boolean;
  vipNames: string[];
  focusMinutes: number;
  unloadAfterMinutes: number;
  closeToTray: boolean;
  lowMemoryBackground: boolean;
  theme: ThemeChoice;
  quietHours: QuietHours;
  launchAtLogin: boolean;
  globalShortcuts: boolean;
  /** Ask for Windows Hello when Cloister starts and after being away. */
  appLock: boolean;
  /** Lock after this many minutes away (0 = only on start and "Lock now"). */
  lockAfterMinutes: number;
  /** Turn on Veil as soon as the window loses focus. */
  veilOnBlur: boolean;
  /** Turn on Veil after this many minutes away (0 = off). */
  veilAfterMinutes: number;
  /** How often a paused background service wakes to check for messages (0 = never). */
  pauseCheckMinutes: number;
  /** Reload hidden pages that grow beyond this many MB (0 = never). */
  refreshOverMb: number;
  /** Look for a new version on start and every few hours. */
  checkForUpdates: boolean;
}

export interface FocusView {
  startedAtMs: number;
  endsAtMs: number;
  held: number;
  /** The timer ran out; the UI should end the session and show the summary. */
  expired: boolean;
}

export interface HeldNotification {
  serviceId: string;
  title: string;
  body: string;
  atMs: number;
}

/** The single view model pushed by the backend whenever something changes. */
export interface Snapshot {
  /** Strictly increasing; older snapshots that arrive late are ignored. */
  revision: number;
  services: Service[];
  active: string | null;
  veil: boolean;
  focus: FocusView | null;
  unreadTotal: number;
  settings: AppSettings;
  /** Cloister is locked; only the lock screen is shown. */
  locked: boolean;
  /** A Cloister PIN is set, so Cloister can be locked. */
  hasPin: boolean;
  /** Windows Hello is set up and can unlock as well as the PIN. */
  helloAvailable: boolean;
  /** Number of notifications in the in-memory history. */
  historyCount: number;
  /** When the newest notification arrived (Unix ms), or `null` when the history is empty. */
  historyLatestMs: number | null;
  /** "Pause in background" is supported on this system (Windows). */
  canPause: boolean;
}

/** What happened to a notification when it arrived. */
export type Delivery = 'shown' | 'held' | 'silenced';

/** A recent notification. Kept in memory only, never written to disk. */
export interface HistoryItem {
  serviceId: string;
  title: string;
  body: string;
  atMs: number;
  delivery: Delivery;
}

/** Memory in bytes: per loaded service, plus what no single service owns. */
export interface MemoryReport {
  services: Record<string, number>;
  /** Cloister itself and, on Windows, the browser engine all services share. */
  shared: number;
}

/** Rectangle in logical pixels where service pages are drawn. */
export interface ContentBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Actions triggered by shortcuts, the tray or the command palette. */
export type ShortcutAction =
  'palette' | 'veil' | 'focus' | 'lock' | 'settings' | `switch:${number}` | `zoom:${ZoomStep}`;
