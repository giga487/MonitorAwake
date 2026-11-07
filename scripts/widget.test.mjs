import test from 'node:test';
import assert from 'node:assert/strict';
import { createSaveQueue } from '../src/lib/save-queue.ts';
import { menuPosition, notificationSeconds } from '../src/lib/widget.ts';
import {
  mascotState,
  notificationEggs,
  idleActionPlan,
  selectRecurringPhrase,
} from '../src/lib/mascot.ts';

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

const timer = (id, overrides = {}) => ({
  id,
  name: `Notification ${id}`,
  deadline_ms: 100_000,
  duration_seconds: 100,
  status: 'running',
  keep_awake: false,
  ...overrides,
});

test('mascot actions yield immediately to pending and ready notifications', () => {
  assert.equal(mascotState([], 0, null), 'waiting');
  assert.equal(mascotState([], 0, 'peck'), 'random-action');
  assert.equal(
    mascotState([timer('a')], 0, 'flutter'),
    'waiting-for-notification',
  );
  assert.equal(
    mascotState([timer('a')], 100_000, 'flutter'),
    'notification-ready',
  );
  assert.equal(
    mascotState([timer('a', { status: 'expired' })], 0, null),
    'notification-ready',
  );
  assert.equal(
    mascotState([timer('a', { status: 'cancelled' })], 0, null),
    'waiting',
  );
});

test('each egg changes from green to yellow to red with its own deadline', () => {
  const timers = [timer('a')];
  assert.equal(notificationEggs(timers, 0)[0].phase, 'green');
  assert.equal(notificationEggs(timers, 49_999)[0].phase, 'green');
  assert.equal(notificationEggs(timers, 50_000)[0].phase, 'yellow');
  assert.equal(notificationEggs(timers, 79_999)[0].phase, 'yellow');
  assert.equal(notificationEggs(timers, 80_000)[0].phase, 'red');
  assert.equal(notificationEggs(timers, 100_000)[0].due, true);
  const extended = timer('a', { deadline_ms: 200_000, duration_seconds: 200 });
  assert.equal(notificationEggs([extended], 80_000)[0].phase, 'green');
});

test('expired eggs remain cracked until dismissed; cancellation removes the egg', () => {
  const timers = [
    timer('b', { deadline_ms: 200_000 }),
    timer('a', { status: 'expired' }),
    timer('c', { status: 'cancelled' }),
    timer('d', { status: 'acknowledged' }),
  ];
  const eggs = notificationEggs(timers, 0);
  assert.deepEqual(
    eggs.map((egg) => egg.timer.id),
    ['a', 'b'],
  );
  assert.equal(eggs[0].due, true);
  assert.equal(eggs[0].phase, 'red');
  assert.deepEqual(
    timers.map((item) => item.id),
    ['b', 'a', 'c', 'd'],
  );
  assert.equal(
    notificationEggs(
      Array.from({ length: 100 }, (_, index) => timer(String(index))),
      0,
    ).length,
    100,
  );
});

test('idle actions are bounded and include both pecking and fluttering', () => {
  assert.deepEqual(
    idleActionPlan(() => 0),
    { action: 'peck', delay: 10_000, duration: 1800 },
  );
  assert.deepEqual(
    idleActionPlan(() => 0.999),
    { action: 'flutter', delay: 24_985, duration: 1400 },
  );
});

test('speech is a third optional action and does not immediately repeat phrases', () => {
  assert.equal(idleActionPlan(() => 0.99, true).action, 'speak');
  assert.equal(idleActionPlan(() => 0.99, false).action, 'flutter');
  assert.equal(selectRecurringPhrase([], null), null);
  assert.equal(
    selectRecurringPhrase(['A', 'B'], 'A', () => 0),
    'B',
  );
  assert.equal(
    selectRecurringPhrase(['A'], 'A', () => 0),
    'A',
  );
  assert.equal(
    selectRecurringPhrase(['', '   ', 'B'], null, () => 0),
    'B',
  );
});

test('autosave, Close and Quit share a drain and save edits made during a write', async () => {
  let dirty = true,
    writes = 0,
    release;
  const flush = createSaveQueue(
    async () => {
      writes++;
      dirty = false;
      if (writes === 1) await new Promise((resolve) => (release = resolve));
      return true;
    },
    () => dirty,
  );
  const first = flush(),
    second = flush();
  assert.equal(first, second);
  assert.equal(writes, 1);
  dirty = true;
  release();
  assert.equal(await first, true);
  assert.equal(writes, 2);
  assert.equal(dirty, false);
});

test('a failed autosave keeps changes dirty and a later explicit retry can save them', async () => {
  let dirty = true,
    fail = true;
  const flush = createSaveQueue(
    async () => {
      if (fail) return false;
      dirty = false;
      return true;
    },
    () => dirty,
  );
  assert.equal(await flush(), false);
  assert.equal(dirty, true);
  fail = false;
  assert.equal(await flush(), true);
  assert.equal(dirty, false);
});
