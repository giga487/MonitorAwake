export const durationUnits = [
  { value: 'seconds', label: 'Seconds', seconds: 1 },
  { value: 'minutes', label: 'Minutes', seconds: 60 },
  { value: 'hours', label: 'Hours', seconds: 3600 },
  { value: 'days', label: 'Days', seconds: 86400 },
] as const;

export type DurationUnit = (typeof durationUnits)[number]['value'];

export function notificationSeconds(
  amount: number | undefined,
  unit: DurationUnit,
): number | null {
  if (
    amount === undefined ||
    !Number.isInteger(amount) ||
    amount <= 0 ||
    amount > 100
  )
    return null;
  const multiplier = durationUnits.find(
    (candidate) => candidate.value === unit,
  )?.seconds;
  return multiplier ? amount * multiplier : null;
}

type Point = { x: number; y: number };
type Size = { width: number; height: number };

// Position only the popup. The mascot's physical coordinates never change.
export function menuPosition(
  mascot: Point,
  mascotSize: Size,
  menuSize: Size,
  workArea: { position: Point; size: Size } | null,
  gap: number,
): Point {
  let x = mascot.x + (mascotSize.width - menuSize.width) / 2;
  let y = mascot.y - menuSize.height - gap;
  if (workArea) {
    const area = workArea;
    if (y < area.position.y) y = mascot.y + mascotSize.height + gap;
    x = Math.max(
      area.position.x,
      Math.min(x, area.position.x + area.size.width - menuSize.width),
    );
    y = Math.max(
      area.position.y,
      Math.min(y, area.position.y + area.size.height - menuSize.height),
    );
  }
  return { x: Math.round(x), y: Math.round(y) };
}
