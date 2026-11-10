/**
 * Update state: checks quietly on start and every few hours (when enabled),
 * and installs on request. Kept apart from the main app store because it has
 * its own lifecycle and talks to GitHub rather than to Cloister's backend.
 */

import { findUpdate, installUpdate, type Update } from '$lib/api/updater';

/** First check shortly after start, so launching stays quick. */
const FIRST_CHECK_DELAY_MS = 20_000;
const CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;

export type UpdateStatus =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'current'; checkedAt: number }
  | { kind: 'available'; version: string; notes: string }
  | { kind: 'installing'; version: string; progress: number | null }
  | { kind: 'failed'; message: string };

class UpdateStore {
  status = $state<UpdateStatus>({ kind: 'idle' });
  #update: Update | null = null;
  #first: ReturnType<typeof setTimeout> | null = null;
  #timer: ReturnType<typeof setInterval> | null = null;

  /** Starts or stops automatic checks. Safe to call repeatedly. */
  setAutomatic(enabled: boolean): void {
    if (enabled && !this.#timer) {
      this.#first = setTimeout(() => void this.check({ quiet: true }), FIRST_CHECK_DELAY_MS);
      this.#timer = setInterval(() => void this.check({ quiet: true }), CHECK_INTERVAL_MS);
    } else if (!enabled && this.#timer) {
      if (this.#first) clearTimeout(this.#first);
      clearInterval(this.#timer);
      this.#first = null;
      this.#timer = null;
    }
  }

  /** Looks for a new version. A quiet check never shows errors (e.g. offline). */
  async check({ quiet = false } = {}): Promise<void> {
    if (this.status.kind === 'installing' || this.status.kind === 'checking') return;
    const previous = this.status;
    this.status = { kind: 'checking' };
    try {
      this.#update = await findUpdate();
      this.status = this.#update
        ? { kind: 'available', version: this.#update.version, notes: this.#update.body ?? '' }
        : { kind: 'current', checkedAt: Date.now() };
    } catch (error) {
      this.status = quiet
        ? previous
        : { kind: 'failed', message: `Could not check for updates: ${String(error)}` };
    }
  }

  /** Downloads, installs and restarts. */
  async install(): Promise<void> {
    if (!this.#update) return;
    const version = this.#update.version;
    this.status = { kind: 'installing', version, progress: null };
    try {
      await installUpdate(this.#update, (progress) => {
        this.status = { kind: 'installing', version, progress };
      });
    } catch (error) {
      this.status = { kind: 'failed', message: `The update could not be installed: ${String(error)}` };
    }
  }
}

export const updates = new UpdateStore();
