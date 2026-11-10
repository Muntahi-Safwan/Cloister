/** Ordering helpers for drag-to-reorder. Pure functions, unit tested. */

/** Returns `items` with `item` moved to `toIndex` (clamped). */
export function moveTo<T>(items: readonly T[], item: T, toIndex: number): T[] {
  const without = items.filter((x) => x !== item);
  const index = Math.max(0, Math.min(toIndex, without.length));
  return [...without.slice(0, index), item, ...without.slice(index)];
}

/**
 * Moves `id` to `toIndex` among the visible ids and returns the new full order.
 *
 * Hidden ids (for example services hidden during a focus session) keep their
 * slots, so reordering never shuffles what the user cannot see. Returns `null`
 * when nothing changes.
 */
export function reorderVisible(
  all: readonly string[],
  visible: readonly string[],
  id: string,
  toIndex: number,
) {
  if (!visible.includes(id)) return null;
  const nextVisible = moveTo(visible, id, toIndex);
  if (nextVisible.every((value, i) => value === visible[i])) return null;

  const visibleSet = new Set(visible);
  let cursor = 0;
  return all.map((existing) => (visibleSet.has(existing) ? nextVisible[cursor++]! : existing));
}
