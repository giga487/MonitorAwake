import test from 'node:test';
import assert from 'node:assert/strict';
import { menuPosition, notificationSeconds } from '../src/lib/widget.ts';

test('popup stays on screen without changing the mascot anchor', () => {
  const workArea = {
    position: { x: 0, y: 0 },
    size: { width: 1920, height: 1040 },
  };
  const mascotSize = { width: 160, height: 160 };
  const menuSize = { width: 320, height: 440 };
  for (const anchor of [
    { x: 1744, y: 880 },
    { x: 0, y: 880 },
    { x: 0, y: 0 },
    { x: 1760, y: 0 },
  ]) {
    const original = { ...anchor };
    const popup = menuPosition(anchor, mascotSize, menuSize, workArea, 6);
    assert.deepEqual(anchor, original);
    assert.ok(popup.x >= 0 && popup.x + menuSize.width <= 1920);
    assert.ok(popup.y >= 0 && popup.y + menuSize.height <= 1040);
    assert.ok(
      popup.y + menuSize.height <= anchor.y ||
        popup.y >= anchor.y + mascotSize.height,
    );
  }
});

test('popup placement uses physical coordinates on a scaled secondary monitor', () => {
  const area = {
    position: { x: -2560, y: -200 },
    size: { width: 2560, height: 1400 },
  };
  const result = menuPosition(
    { x: -240, y: 960 },
    { width: 240, height: 240 },
    { width: 480, height: 660 },
    area,
    9,
  );
  assert.deepEqual(result, { x: -480, y: 291 });
});

test('notification units convert the selected amount to seconds', () => {
  assert.equal(notificationSeconds(1, 'seconds'), 1);
  assert.equal(notificationSeconds(5, 'minutes'), 300);
  assert.equal(notificationSeconds(2, 'hours'), 7200);
  assert.equal(notificationSeconds(100, 'days'), 8_640_000);
});

test('notification cannot start with empty, zero, fractional or out-of-range values', () => {
  for (const value of [undefined, NaN, Infinity, -1, 0, 1.5, 101]) {
    assert.equal(notificationSeconds(value, 'minutes'), null);
  }
});
