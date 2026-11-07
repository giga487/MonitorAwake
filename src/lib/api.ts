import { invoke, isTauri } from '@tauri-apps/api/core';

export type AwakeMode = 'off' | 'system' | 'display';
export interface Timer {
  id: string;
  name: string;
  deadline_ms: number;
  duration_seconds: number;
  status: 'running' | 'expired' | 'acknowledged' | 'cancelled';
  keep_awake: boolean;
}
export interface Settings {
  animations: boolean;
  notifications: boolean;
  mascot: 'chicken';
  phrases: string[];
  speech_enabled: boolean;
}
export type NoteColor = 'yellow' | 'sage' | 'blue' | 'rose';
export interface NoteBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}
export interface Note {
  id: string;
  title: string;
  body: string;
  color: NoteColor;
  visible: boolean;
  pinned: boolean;
  bounds: NoteBounds;
  content_revision: number;
  reminder_id: string | null;
}
export interface Snapshot {
  schema_version: number;
  app_version: string;
  notes: Note[];
  desktop_error: string | null;
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
  | { type: 'create_note' }
  | {
      type: 'update_note';
      id: string;
      expected_revision: number;
      title: string;
      body: string;
    }
  | {
      type: 'set_note_appearance';
      id: string;
      color: NoteColor;
      pinned: boolean;
    }
  | { type: 'set_note_visible'; id: string; visible: boolean }
  | { type: 'set_all_notes_visible'; visible: boolean }
  | { type: 'set_note_bounds'; id: string; bounds: NoteBounds }
  | { type: 'delete_note'; id: string }
  | { type: 'set_note_reminder'; id: string; seconds: number }
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
      .catch(() => ({ error: 'Invalid request.' }));
    throw new Error(result.error ?? 'The server did not accept the request.');
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
      label: 'Automatic',
      description: 'Follow system power settings',
    },
    {
      value: 'system',
      label: 'System awake',
      description: 'The display can turn off',
    },
    {
      value: 'display',
      label: 'Keep display on too',
      description: 'Keep your work on screen',
    },
  ];
