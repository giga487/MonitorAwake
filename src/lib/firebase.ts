import {
  getApps,
  initializeApp,
  type FirebaseApp,
  type FirebaseOptions,
} from 'firebase/app';

export interface FirebaseLocalConfig {
  schema_version: 1;
  project_id: string;
  api_key: string;
  auth_domain: string;
  app_id: string;
  storage_bucket?: string;
  messaging_sender_id?: string;
}

const APP_NAME = 'monitorawake-notes';

// Accept runtime configuration; never import the user's JSON into a Vite bundle.
export function firebaseOptions(value: unknown): FirebaseOptions {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(
      'Invalid Firebase configuration. The original file has not been changed.',
    );
  }
  const config = value as Record<string, unknown>;
  if (config.schema_version !== 1) {
    throw new Error(
      'Unsupported Firebase configuration version. The original file has not been changed.',
    );
  }
  for (const field of ['project_id', 'api_key', 'auth_domain', 'app_id']) {
    if (typeof config[field] !== 'string' || !config[field].trim()) {
      throw new Error(
        `Firebase configuration is incomplete: ${field} is required.`,
      );
    }
  }
  for (const field of ['storage_bucket', 'messaging_sender_id']) {
    if (config[field] !== undefined && typeof config[field] !== 'string') {
      throw new Error(`Invalid Firebase configuration field: ${field}.`);
    }
  }
  const validated = config as unknown as FirebaseLocalConfig;
  return {
    projectId: validated.project_id.trim(),
    apiKey: validated.api_key.trim(),
    authDomain: validated.auth_domain.trim(),
    appId: validated.app_id.trim(),
    ...(validated.storage_bucket
      ? { storageBucket: validated.storage_bucket.trim() }
      : {}),
    ...(validated.messaging_sender_id
      ? { messagingSenderId: validated.messaging_sender_id.trim() }
      : {}),
  };
}

// Initialization only: no Auth, Firestore, Analytics or automatic startup hook yet.
export function initializeNotesFirebase(config: unknown): FirebaseApp {
  const options = firebaseOptions(config);
  const existing = getApps().find((app) => app.name === APP_NAME);
  if (existing) {
    if (JSON.stringify(existing.options) !== JSON.stringify(options)) {
      throw new Error(
        'Disconnect the existing Firebase client before changing its configuration.',
      );
    }
    return existing;
  }
  return initializeApp(options, {
    name: APP_NAME,
    automaticDataCollectionEnabled: false,
  });
}
