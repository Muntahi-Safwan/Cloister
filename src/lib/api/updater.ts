/**
 * Thin wrapper around Tauri's updater, so the rest of the UI never imports
 * the plugin directly. Updates are signed; the app refuses any package whose
 * signature does not match the public key in `tauri.conf.json`.
 */

import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';

export type { Update };

/** Asks GitHub for a newer release. Resolves to `null` when up to date. */
export function findUpdate(): Promise<Update | null> {
  return check({ timeout: 30_000 });
}

/**
 * Downloads and installs `update`, reporting progress from 0 to 1 where the
 * size is known, then restarts Cloister. On Windows the installer closes and
 * reopens the app itself.
 */
export async function installUpdate(
  update: Update,
  onProgress: (fraction: number | null) => void,
): Promise<void> {
  let total = 0;
  let received = 0;
  await update.downloadAndInstall((event) => {
    if (event.event === 'Started') total = event.data.contentLength ?? 0;
    if (event.event === 'Progress') {
      received += event.data.chunkLength;
      onProgress(total > 0 ? Math.min(received / total, 1) : null);
    }
  });
  await relaunch();
}
