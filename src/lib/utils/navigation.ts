/**
 * Moving between services from the keyboard: jump to the next unread one, and
 * flip back to the one used before.
 */

/** The part of a service this module needs. */
export interface Stop {
  id: string;
  unread: number;
}

/**
 * The first service after the active one (wrapping round) that has unread
 * messages, or `null` when nothing else is waiting.
 */
export function nextUnread(services: readonly Stop[], activeId: string | null): string | null {
  const start = services.findIndex((s) => s.id === activeId);
  for (let step = 1; step <= services.length; step++) {
    const service = services[(start + step + services.length) % services.length];
    if (service && service.unread > 0 && service.id !== activeId) return service.id;
  }
  return null;
}

/** How many recently used services are remembered. */
const RECENT_LIMIT = 20;

/**
 * Most recently used services, newest first. `visit` is called whenever the
 * active service changes; `previous` is where Ctrl+Tab goes.
 */
export class RecentServices {
  #ids: string[] = [];

  visit(id: string | null): void {
    if (!id || this.#ids[0] === id) return;
    this.#ids = [id, ...this.#ids.filter((other) => other !== id)].slice(0, RECENT_LIMIT);
  }

  /** The most recent service that is not the active one and still exists. */
  previous(activeId: string | null, existing: ReadonlySet<string>): string | null {
    return this.#ids.find((id) => id !== activeId && existing.has(id)) ?? null;
  }
}
