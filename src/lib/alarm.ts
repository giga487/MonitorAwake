const MAX_DELAY_MS = 100 * 24 * 60 * 60 * 1000;

export function localAlarmValue(timestamp: number): string {
  const date = new Date(timestamp);
  const pad = (value: number) => String(value).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

// Reject normalized dates (including a missing hour at the spring DST change).
export function alarmDeadline(value: string, now: number): number | null {
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(value)) return null;
  const date = new Date(value);
  const timestamp = date.getTime();
  if (
    !Number.isSafeInteger(timestamp) ||
    localAlarmValue(timestamp) !== value ||
    timestamp <= now ||
    timestamp - now > MAX_DELAY_MS
  )
    return null;
  return timestamp;
}
