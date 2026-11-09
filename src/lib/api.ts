import { invoke, isTauri } from '@tauri-apps/api/core';

export type AwakeMode = 'off' | 'system' | 'display';
export const mascots = [
  { id: 'chicken', name: 'Gallina robot', src: '/mascot.png' },
  { id: 'elf', name: 'Elfo dei boschi', src: '/elf.png' },
] as const;
export interface Timer {
  id: string;
  name: string;
  deadline_ms: number;
  duration_seconds: number;
  status: 'running' | 'expired' | 'acknowledged' | 'cancelled';
  keep_awake: boolean;
}
export interface Settings {
  mascot: (typeof mascots)[number]['id'];
  animations: boolean;
  notifications: boolean;
}
export interface Snapshot {
  timers: Timer[];
  settings: Settings;
  manual_mode: AwakeMode;
  requested_mode: AwakeMode;
  applied_mode: AwakeMode;
  capabilities: { system: boolean; display: boolean; backend: string };
  backend_error: string | null;
  notification_error: string | null;
  now_ms: number;
}
export type Action =
  | { type: 'add_timer'; name: string; seconds: number; keep_awake: boolean }
  | { type: 'cancel_timer' | 'acknowledge_timer'; id: string }
  | { type: 'extend_timer'; id: string; seconds: number }
  | { type: 'set_awake'; mode: AwakeMode }
  | { type: 'update_settings'; settings: Settings }
  | { type: 'clear_history' }
  | { type: 'command'; text: string };

export const desktop = isTauri();
let token = '';

export function initializeSession(): boolean {
  if (desktop) return true;
  const supplied = new URLSearchParams(location.hash.slice(1)).get('token');
  if (supplied) {
    sessionStorage.setItem('monitorawake-session', supplied);
    history.replaceState(null, '', location.pathname);
  }
  token = sessionStorage.getItem('monitorawake-session') ?? '';
  return Boolean(token);
}

async function request<T>(path: string, action?: Action): Promise<T> {
  const response = await fetch(path, {
    method: action ? 'POST' : 'GET',
    headers: {
      Authorization: `Bearer ${token}`,
      ...(action ? { 'Content-Type': 'application/json' } : {}),
    },
    body: action ? JSON.stringify(action) : undefined,
    cache: 'no-store',
    signal: AbortSignal.timeout(5000),
  });
  if (!response.ok) {
    const result = await response
      .json()
      .catch(() => ({ error: 'Richiesta non valida.' }));
    throw new Error(result.error ?? 'Il server non ha accettato la richiesta.');
  }
  return response.json();
}

export const getState = () =>
  desktop ? invoke<Snapshot>('get_state') : request<Snapshot>('/api/state');
export const dispatch = (action: Action) =>
  desktop
    ? invoke<{ message: string; state: Snapshot }>('dispatch', { action })
    : request<{ message: string; state: Snapshot }>('/api/action', action);

export function remaining(timer: Timer, now: number): string {
  const seconds = Math.max(0, Math.ceil((timer.deadline_ms - now) / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${hours ? `${hours}:` : ''}${String(minutes).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
}

export const modes: { value: AwakeMode; label: string; description: string }[] =
  [
    {
      value: 'off',
      label: 'Automatico',
      description: 'Segui le impostazioni del sistema',
    },
    {
      value: 'system',
      label: 'Sistema sveglio',
      description: 'Lo schermo può spegnersi',
    },
    {
      value: 'display',
      label: 'Anche lo schermo',
      description: 'Mantieni visibile quello che fai',
    },
  ];
