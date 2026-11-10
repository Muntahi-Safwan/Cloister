import { describe, expect, it } from 'vitest';

import { moveTo, reorderVisible } from './order';

describe('order', () => {
  it('moves an item to an index', () => {
    expect(moveTo(['a', 'b', 'c'], 'a', 2)).toEqual(['b', 'c', 'a']);
    expect(moveTo(['a', 'b', 'c'], 'c', 0)).toEqual(['c', 'a', 'b']);
    expect(moveTo(['a', 'b', 'c'], 'b', 99)).toEqual(['a', 'c', 'b']);
  });

  it('keeps hidden items in their slots', () => {
    // "h" is hidden during focus; moving "c" first must not move "h".
    expect(reorderVisible(['a', 'h', 'b', 'c'], ['a', 'b', 'c'], 'c', 0)).toEqual(['c', 'h', 'a', 'b']);
  });

  it('returns null when nothing changes or the id is not visible', () => {
    expect(reorderVisible(['a', 'b'], ['a', 'b'], 'a', 0)).toBeNull();
    expect(reorderVisible(['a', 'b'], ['a'], 'b', 0)).toBeNull();
  });
});
