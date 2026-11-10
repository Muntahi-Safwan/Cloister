/** Formatting helpers used across the UI. Pure functions, unit tested. */

/**
 * Monogram for a service tile: one letter for single words ("Gmail" → "G"),
 * two initials for longer names ("Company chat" → "CC").
 */
export function initials(name: string): string {
  const words = name
    .trim()
    .split(/[\s._-]+/)
    .filter(Boolean);
  if (words.length === 0) return '?';
  if (words.length === 1) return words[0]![0]!.toUpperCase();
  return (words[0]![0]! + words[1]![0]!).toUpperCase();
}

/** Formats milliseconds as `mm:ss` for the focus clock. Negative values clamp to zero. */
export function formatClock(ms: number): string {
  const totalSeconds = Math.max(0, Math.ceil(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

/** Caps large unread counts so badges stay narrow. */
export function badgeCount(count: number): string {
  return count > 99 ? '99+' : String(count);
}

/** Simple English plural: `plural(1, 'message')` → "1 message", `plural(3, 'message')` → "3 messages". */
export function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`;
}

/** Whether a `mutedUntil` timestamp means "muted" at `nowMs`. */
export function isMuted(mutedUntil: number | null, nowMs: number): boolean {
  return mutedUntil !== null && nowMs < mutedUntil;
}

/** Minutes after midnight as `HH:MM`, the value format of `<input type="time">`. */
export function minutesToTime(minutes: number): string {
  const h = Math.floor(minutes / 60) % 24;
  const m = minutes % 60;
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
}

/** Parses `HH:MM` into minutes after midnight, or `null` if invalid. */
export function timeToMinutes(value: string): number | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(value);
  if (!match) return null;
  const [h, m] = [Number(match[1]), Number(match[2])];
  return h < 24 && m < 60 ? h * 60 + m : null;
}

/** Bytes as a short size: `412 MB`, `1.2 GB`. */
export function formatBytes(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  if (mb < 1) return '<1 MB';
  if (mb < 1024) return `${Math.round(mb)} MB`;
  return `${(mb / 1024).toFixed(1)} GB`;
}

/** How long ago `atMs` was, relative to `nowMs`: "now", "4 min", "2 h", "3 d". */
export function timeAgo(atMs: number, nowMs: number): string {
  const minutes = Math.floor(Math.max(0, nowMs - atMs) / 60_000);
  if (minutes < 1) return 'now';
  if (minutes < 60) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} h`;
  return `${Math.floor(hours / 24)} d`;
}
