/**
 * Svelte action: drag items of a vertical list to reorder them.
 *
 * Uses pointer events rather than HTML5 drag and drop, which Tauri's file-drop
 * handling intercepts on Windows. Items are the descendants marked with
 * `data-reorder-id`. While dragging, the item follows the pointer and the
 * others slide to make room; on release `onreorder(id, toIndex)` is called.
 * A drag shorter than a few pixels is treated as a normal click.
 *
 * Usage: `<div use:reorderable={{ onreorder }}>…<div data-reorder-id={id}>…`
 */

import type { Action } from 'svelte/action';

export interface ReorderableOptions {
  onreorder: (id: string, toIndex: number) => void;
}

/** Distance in pixels the pointer must travel before a press becomes a drag. */
const DRAG_THRESHOLD = 5;

interface DragState {
  id: string;
  element: HTMLElement;
  items: HTMLElement[];
  fromIndex: number;
  toIndex: number;
  startY: number;
  pitch: number;
  pointerId: number;
  active: boolean;
}

export const reorderable: Action<HTMLElement, ReorderableOptions> = (node, initial) => {
  let options = initial;
  let drag: DragState | null = null;

  const itemsOf = () => [...node.querySelectorAll<HTMLElement>('[data-reorder-id]')];

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    const element = (event.target as HTMLElement).closest<HTMLElement>('[data-reorder-id]');
    if (!element || !node.contains(element)) return;
    const items = itemsOf();
    const fromIndex = items.indexOf(element);
    const next = items[fromIndex + 1] ?? items[fromIndex - 1];
    const pitch = next
      ? Math.abs(next.getBoundingClientRect().top - element.getBoundingClientRect().top)
      : element.offsetHeight;
    drag = {
      id: element.dataset.reorderId ?? '',
      element,
      items,
      fromIndex,
      toIndex: fromIndex,
      startY: event.clientY,
      pitch,
      pointerId: event.pointerId,
      active: false,
    };
  }

  function onPointerMove(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    const dy = event.clientY - drag.startY;
    if (!drag.active) {
      if (Math.abs(dy) < DRAG_THRESHOLD) return;
      drag.active = true;
      node.setPointerCapture(event.pointerId);
      node.classList.add('is-reordering');
      drag.element.classList.add('is-dragging');
    }

    const shift = Math.round(dy / drag.pitch);
    drag.toIndex = Math.max(0, Math.min(drag.items.length - 1, drag.fromIndex + shift));
    drag.element.style.transform = `translateY(${dy}px)`;

    drag.items.forEach((item, index) => {
      if (item === drag!.element) return;
      const { fromIndex, toIndex, pitch } = drag!;
      let offset = 0;
      if (fromIndex < toIndex && index > fromIndex && index <= toIndex) offset = -pitch;
      if (fromIndex > toIndex && index < fromIndex && index >= toIndex) offset = pitch;
      item.style.transform = offset ? `translateY(${offset}px)` : '';
    });
  }

  function finish(commit: boolean) {
    if (!drag) return;
    const { active, id, toIndex, fromIndex, items, element, pointerId } = drag;
    drag = null;
    if (!active) return;

    items.forEach((item) => (item.style.transform = ''));
    element.classList.remove('is-dragging');
    node.classList.remove('is-reordering');
    if (node.hasPointerCapture(pointerId)) node.releasePointerCapture(pointerId);

    // The release would otherwise count as a click on the item. If no click
    // follows (released outside the list), drop the guard right away.
    node.addEventListener('click', swallowClick, { capture: true, once: true });
    setTimeout(() => node.removeEventListener('click', swallowClick, { capture: true }), 0);
    if (commit && toIndex !== fromIndex) options.onreorder(id, toIndex);
  }

  function swallowClick(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
  }

  const onPointerUp = () => finish(true);
  const onPointerCancel = () => finish(false);
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') finish(false);
  };

  node.addEventListener('pointerdown', onPointerDown);
  node.addEventListener('pointermove', onPointerMove);
  node.addEventListener('pointerup', onPointerUp);
  node.addEventListener('pointercancel', onPointerCancel);
  window.addEventListener('keydown', onKeyDown);

  return {
    update(next) {
      options = next;
    },
    destroy() {
      node.removeEventListener('pointerdown', onPointerDown);
      node.removeEventListener('pointermove', onPointerMove);
      node.removeEventListener('pointerup', onPointerUp);
      node.removeEventListener('pointercancel', onPointerCancel);
      window.removeEventListener('keydown', onKeyDown);
    },
  };
};
