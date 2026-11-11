import { describe, expect, it } from 'vitest';

import {
  badgeCount,
  formatBytes,
  formatClock,
  initials,
  isMuted,
  minutesToTime,
  plural,
  timeAgo,
  timeToMinutes,
} from './format';
import { shortcutFor, type KeyChord } from './shortcuts';
import { catalogIconUrl, checkServiceUrl, faviconUrl, withHttps } from './url';

const chord = (key: string, mods: Partial<KeyChord> = {}): KeyChord => ({
  key,
  ctrlKey: true,
  shiftKey: false,
  altKey: false,
  metaKey: false,
  ...mods,
});

describe('format', () => {
  it('derives monograms', () => {
    expect(initials('Company chat')).toBe('CC');
    expect(initials('Gmail')).toBe('G');
    expect(initials('chat.example.com')).toBe('CE');
    expect(initials('   ')).toBe('?');
  });

  it('formats the focus clock', () => {
    expect(formatClock(25 * 60_000)).toBe('25:00');
    expect(formatClock(61_500)).toBe('01:02');
    expect(formatClock(-5)).toBe('00:00');
  });

  it('reads mute state and converts times', () => {
    expect(isMuted(null, 5)).toBe(false);
    expect(isMuted(10, 5)).toBe(true);
    expect(isMuted(10, 10)).toBe(false);
    expect(minutesToTime(23 * 60)).toBe('23:00');
    expect(minutesToTime(8 * 60 + 5)).toBe('08:05');
    expect(timeToMinutes('07:30')).toBe(450);
    expect(timeToMinutes('25:00')).toBeNull();
    expect(timeToMinutes('nope')).toBeNull();
  });

  it('caps badges and pluralises', () => {
    expect(badgeCount(7)).toBe('7');
    expect(badgeCount(140)).toBe('99+');
    expect(plural(1, 'message')).toBe('1 message');
    expect(plural(4, 'message')).toBe('4 messages');
  });
});

describe('url', () => {
  it('accepts https addresses with a domain', () => {
    expect(checkServiceUrl('https://web.whatsapp.com').ok).toBe(true);
  });

  it('rejects insecure, empty and host-less addresses', () => {
    expect(checkServiceUrl('').ok).toBe(false);
    expect(checkServiceUrl('http://example.com').ok).toBe(false);
    expect(checkServiceUrl('https://localhost').ok).toBe(false);
    expect(checkServiceUrl('nonsense').ok).toBe(false);
  });

  it('adds https when the scheme is missing', () => {
    expect(withHttps('chat.example.com')).toBe('https://chat.example.com');
    expect(withHttps('https://a.com')).toBe('https://a.com');
  });

  it('builds favicon URLs', () => {
    expect(faviconUrl('https://chat.example.com/app')).toBe('https://chat.example.com/favicon.ico');
    expect(faviconUrl('bad')).toBeNull();
    expect(catalogIconUrl('whatsapp.com')).toBe('https://icons.duckduckgo.com/ip3/whatsapp.com.ico');
  });
});

describe('shortcuts', () => {
  it('maps Cloister chords to actions', () => {
    expect(shortcutFor(chord('k'))).toBe('palette');
    expect(shortcutFor(chord('K', { shiftKey: true }))).toBe('palette');
    expect(shortcutFor(chord('P', { shiftKey: true }))).toBe('veil');
    expect(shortcutFor(chord('F', { shiftKey: true }))).toBe('focus');
    expect(shortcutFor(chord('L', { shiftKey: true }))).toBe('lock');
  });

  it('uses Cmd instead of Ctrl on a Mac', () => {
    expect(shortcutFor(chord('k', { ctrlKey: false, metaKey: true }), true)).toBe('palette');
    expect(shortcutFor(chord('k'), true)).toBeNull();
    expect(shortcutFor(chord('k', { metaKey: true }), false)).toBeNull();
    expect(shortcutFor(chord(','))).toBe('settings');
    expect(shortcutFor(chord('3'))).toBe('switch:3');
    expect(shortcutFor(chord('='))).toBe('zoom:in');
    expect(shortcutFor(chord('-'))).toBe('zoom:out');
    expect(shortcutFor(chord('0'))).toBe('zoom:reset');
    expect(shortcutFor(chord('f'))).toBe('find');
    expect(shortcutFor(chord('F', { shiftKey: true }))).toBe('focus');
  });

  it('ignores everything else', () => {
    expect(shortcutFor(chord('k', { ctrlKey: false }))).toBeNull();
    expect(shortcutFor(chord('p'))).toBeNull();
    expect(shortcutFor(chord('k', { altKey: true }))).toBeNull();
  });
});

describe('formatBytes', () => {
  it('uses MB below a gigabyte and one decimal above', () => {
    expect(formatBytes(1000)).toBe('<1 MB');
    expect(formatBytes(412 * 1024 * 1024)).toBe('412 MB');
    expect(formatBytes(1.25 * 1024 * 1024 * 1024)).toBe('1.3 GB');
  });
});

describe('timeAgo', () => {
  const now = 10 * 24 * 3_600_000;
  it('rounds down to the largest whole unit', () => {
    expect(timeAgo(now - 20_000, now)).toBe('now');
    expect(timeAgo(now - 4 * 60_000, now)).toBe('4 min');
    expect(timeAgo(now - 2 * 3_600_000 - 1, now)).toBe('2 h');
    expect(timeAgo(now - 3 * 24 * 3_600_000, now)).toBe('3 d');
  });

  it('treats future times as now', () => {
    expect(timeAgo(now + 5000, now)).toBe('now');
  });
});
