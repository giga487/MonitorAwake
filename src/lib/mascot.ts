import type { Timer } from './api';

export type MascotKind = 'chicken' | 'frost_gryphon' | 'ember_wyvern';
export type IdleAction = 'peck' | 'flutter' | 'speak';
export type MascotState =
  | 'waiting'
  | 'random-action'
  | 'waiting-for-notification'
  | 'notification-ready';
export type EggPhase = 'green' | 'yellow' | 'red';

export const mascots: {
  id: MascotKind;
  name: string;
  atlas: string;
  rendering: 'pixelated' | 'auto';
  description: string;
}[] = [
  {
    id: 'chicken',
    name: 'Chicken',
    atlas: '/chicken-actions.png',
    rendering: 'pixelated',
    description:
      'A curious little hen. Pecks, flutters and keeps your notifications warm.',
  },
  {
    id: 'frost_gryphon',
    name: 'Frost gryphon',
    atlas: '/frost-gryphon-actions.png',
    rendering: 'auto',
    description:
      'A little guardian from snowy peaks. Curious, brave and always watching over your reminders.',
  },
  {
    id: 'ember_wyvern',
    name: 'Ember wyvern',
    atlas: '/ember-wyvern-actions.png',
    rendering: 'auto',
    description:
      'A spirited little wind rider with crimson wings, iron details and a warm heart. Your reminders are under its protection.',
  },
];

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

export function idleActionPlan(
  random: () => number = Math.random,
  speech = false,
) {
  const roll = random();
  const action: IdleAction = speech
    ? roll < 1 / 3
      ? 'peck'
      : roll < 2 / 3
        ? 'flutter'
        : 'speak'
    : roll < 0.5
      ? 'peck'
      : 'flutter';
  return {
    action,
    delay: 10_000 + Math.floor(random() * 15_000),
    duration: action === 'peck' ? 1800 : action === 'flutter' ? 1400 : 6000,
  };
}
