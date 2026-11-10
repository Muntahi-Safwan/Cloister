/**
 * The native right-click menu for a service in the sidebar.
 *
 * A native menu is used rather than an HTML one because service pages are
 * native views that would cover an HTML menu next to the sidebar.
 */

import { Menu } from '@tauri-apps/api/menu';

import type { BackgroundMode, MuteDuration, Service } from '$lib/types';
import { isMuted } from '$lib/utils/format';

/** What the menu can do. Implemented by the app store. */
export interface ServiceMenuActions {
  open: (id: string) => void;
  rename: (id: string) => void;
  mute: (id: string, duration: MuteDuration) => void;
  sleep: (id: string) => void;
  background: (id: string, mode: BackgroundMode) => void;
  reload: (id: string) => void;
  openExternally: (id: string) => void;
  copyLink: (url: string) => void;
  erase: (id: string) => void;
  remove: (id: string) => void;
}

/** Builds and shows the menu at the pointer. `canPause` is false where pause mode is unsupported. */
export async function showServiceMenu(
  service: Service,
  actions: ServiceMenuActions,
  canPause: boolean,
): Promise<void> {
  const { id } = service;
  const muted = isMuted(service.mutedUntil, Date.now());

  const muteMenu = muted
    ? { id: 'unmute', text: 'Unmute notifications', action: () => actions.mute(id, 'off') }
    : {
        text: 'Mute notifications',
        items: [
          { id: 'mute-hour', text: 'For 1 hour', action: () => actions.mute(id, 'oneHour') },
          {
            id: 'mute-tomorrow',
            text: 'Until 8:00 tomorrow',
            action: () => actions.mute(id, 'untilTomorrow'),
          },
          { id: 'mute-forever', text: 'Until I unmute', action: () => actions.mute(id, 'forever') },
        ],
      };

  const modes: { mode: BackgroundMode; text: string }[] = [
    { mode: 'keepNotifying', text: 'Keep notifying' },
    ...(canPause ? [{ mode: 'pause' as const, text: 'Pause, check for messages every few minutes' }] : []),
    { mode: 'unloadWhenIdle', text: 'Unload when idle' },
  ];
  const backgroundMenu = {
    text: 'In the background',
    items: modes.map(({ mode, text }) => ({
      id: `background-${mode}`,
      text,
      checked: service.options.background === mode,
      action: () => actions.background(id, mode),
    })),
  };

  const menu = await Menu.new({
    items: [
      { id: 'open', text: `Open ${service.name}`, action: () => actions.open(id) },
      { item: 'Separator' },
      muteMenu,
      backgroundMenu,
      { id: 'rename', text: 'Rename…', action: () => actions.rename(id) },
      { id: 'reload', text: 'Reload', enabled: service.loaded, action: () => actions.reload(id) },
      {
        id: 'sleep',
        text: 'Sleep now (free memory)',
        enabled: service.loaded,
        action: () => actions.sleep(id),
      },
      { item: 'Separator' },
      { id: 'external', text: 'Open in browser', action: () => actions.openExternally(id) },
      { id: 'copy', text: 'Copy link', action: () => actions.copyLink(service.url) },
      { item: 'Separator' },
      { id: 'erase', text: 'Sign out and clear data…', action: () => actions.erase(id) },
      { id: 'remove', text: 'Remove…', action: () => actions.remove(id) },
    ],
  });
  await menu.popup();
}
