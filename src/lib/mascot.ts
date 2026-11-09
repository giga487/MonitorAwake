import type { Timer } from './api';

export type MascotKind = (typeof mascots)[number]['id'];
export type IdleAction = 'peck' | 'flutter' | 'speak';
export type MascotState =
  | 'waiting'
  | 'random-action'
  | 'waiting-for-notification'
  | 'notification-ready';
export type EggPhase = 'green' | 'yellow' | 'red';

export const mascots = [
  {
    id: 'chicken',
    name: 'Chicken',
    src: '/chicken-actions.png',
    layout: 'atlas',
    rendering: 'pixelated',
    description:
      'A curious little hen. Pecks, flutters and keeps your notifications warm.',
  },
  {
    id: 'frost_gryphon',
    name: 'Frost gryphon',
    src: '/frost-gryphon-pixel-actions.png',
    layout: 'atlas',
    rendering: 'pixelated',
    description:
      'A little guardian from snowy peaks. Curious, brave and always watching over your reminders.',
  },
  {
    id: 'ember_wyvern',
    name: 'Ember wyvern',
    src: '/ember-wyvern-pixel-actions.png',
    layout: 'atlas',
    rendering: 'pixelated',
    description:
      'A spirited little wind rider with crimson wings, iron details and a warm heart. Your reminders are under its protection.',
  },
  {
    id: 'elf',
    name: 'Woodland elf',
    src: '/elf.png',
    layout: 'single',
    rendering: 'pixelated',
    description:
      'A woodland lookout with a leaf shield. Leans from side to side and hops between reminders.',
  },
] as const;

export function mascotState(
  timers: readonly Timer[],
  now: number,
  action: IdleAction | null,
): MascotState {
  if (
    timers.some(
      (timer) =>
        timer.status === 'expired' ||
        (timer.status === 'running' && timer.deadline_ms <= now),
    )
  )
    return 'notification-ready';
  if (timers.some((timer) => timer.status === 'running'))
    return 'waiting-for-notification';
  return action ? 'random-action' : 'waiting';
}

export function notificationEggs(timers: readonly Timer[], now: number) {
  return timers
    .filter((timer) => timer.status === 'running' || timer.status === 'expired')
    .slice()
    .sort((a, b) => a.deadline_ms - b.deadline_ms || a.id.localeCompare(b.id))
    .map((timer) => {
      const remaining = Math.max(0, timer.deadline_ms - now);
      const ratio = remaining / Math.max(1, timer.duration_seconds * 1000);
      const phase: EggPhase =
        timer.status === 'expired' || ratio <= 0.2
          ? 'red'
          : ratio <= 0.5
            ? 'yellow'
            : 'green';
      const due = timer.status === 'expired' || remaining === 0;
      return {
        timer,
        phase,
        due,
        label: due
          ? 'Ready'
          : phase === 'red'
            ? 'Due soon'
            : phase === 'yellow'
              ? 'Halfway there'
              : 'Plenty of time',
      };
    });
}

export function selectRecurringPhrase(
  phrases: readonly string[],
  previous: string | null,
  random: () => number = Math.random,
): string | null {
  const available = phrases.filter((text) => text.trim() && text !== previous);
  const choices = available.length
    ? available
    : phrases.filter((text) => text.trim());
  return choices.length
    ? choices[
        Math.min(choices.length - 1, Math.floor(random() * choices.length))
      ]
    : null;
}

export function idleActionPlan(random: () => number = Math.random) {
  const roll = random();
  const action: IdleAction = roll < 0.5 ? 'peck' : 'flutter';
  return {
    action,
    delay: 10_000 + Math.floor(random() * 15_000),
    duration: action === 'peck' ? 1800 : 1400,
  };
}

export function speechDelay(
  minMinutes = 5,
  maxMinutes = 10,
  random: () => number = Math.random,
) {
  return Math.round(
    (minMinutes + random() * (maxMinutes - minMinutes)) * 60_000,
  );
}
