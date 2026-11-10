import { deleteApp, type FirebaseApp } from 'firebase/app';
import {
  initializeAuth,
  indexedDBLocalPersistence,
  browserLocalPersistence,
  connectAuthEmulator,
  signOut,
  type Auth,
  type Persistence,
} from 'firebase/auth';
import { initializeNotesFirebase } from './firebase.ts';

const sessions = new Map<
  FirebaseApp,
  { ready: Promise<Auth>; emulatorUrl?: string }
>();

export function initializeNotesAuth(
  config: unknown,
  options: {
    emulatorUrl?: string;
    persistence?: Persistence | Persistence[];
  } = {},
): Promise<Auth> {
  if (
    options.emulatorUrl &&
    !/^http:\/\/127\.0\.0\.1:[1-9]\d{0,4}$/.test(options.emulatorUrl)
  ) {
    return Promise.reject(
      new Error('Auth emulator must use a loopback HTTP address.'),
    );
  }
  const app = initializeNotesFirebase(config);
  const existing = sessions.get(app);
  if (existing) {
    if (existing.emulatorUrl !== options.emulatorUrl)
      return Promise.reject(
        new Error('Disconnect before changing Auth environment.'),
      );
    return existing.ready;
  }
  const auth = initializeAuth(app, {
    persistence: options.persistence ?? [
      indexedDBLocalPersistence,
      browserLocalPersistence,
    ],
  });
  if (options.emulatorUrl) connectAuthEmulator(auth, options.emulatorUrl);
  const ready = auth.authStateReady().then(() => auth);
  sessions.set(app, { ready, emulatorUrl: options.emulatorUrl });
  return ready;
}

export async function disconnectNotesFirebase(): Promise<void> {
  for (const [app, session] of sessions) {
    const auth = await session.ready;
    await signOut(auth);
    await deleteApp(app);
    sessions.delete(app);
  }
}

export function firebaseAuthError(error: unknown): string {
  const code =
    error && typeof error === 'object' && 'code' in error
      ? String(error.code)
      : '';
  switch (code) {
    case 'auth/unauthorized-domain':
      return 'Add 127.0.0.1 to Authentication → Settings → Authorized domains in Firebase.';
    case 'auth/popup-blocked':
      return 'Allow the Google sign-in popup in your browser, then try again.';
    case 'auth/popup-closed-by-user':
    case 'auth/cancelled-popup-request':
      return 'Google sign-in was cancelled. You can try again.';
    case 'auth/operation-not-allowed':
    case 'auth/configuration-not-found':
      return 'Enable Email/Password in Firebase Authentication for this project.';
    case 'auth/invalid-credential':
    case 'auth/wrong-password':
    case 'auth/user-not-found':
      return 'Email or password is incorrect.';
    case 'auth/email-already-in-use':
      return 'This email already has an account. Use Sign in.';
    case 'auth/weak-password':
    case 'auth/password-does-not-meet-requirements':
      return 'Choose a stronger password that meets the project password policy.';
    case 'auth/invalid-email':
      return 'Enter a valid email address.';
    case 'auth/too-many-requests':
      return 'Too many attempts. Please try again later.';
    case 'auth/network-request-failed':
      return 'Cannot reach Firebase. Check your connection and try again.';
    case 'auth/user-disabled':
    case 'auth/user-token-expired':
    case 'auth/invalid-user-token':
      return 'Your session is no longer valid. Sign in again or check your account in Firebase.';
    case 'auth/invalid-api-key':
    case 'auth/app-not-authorized':
      return 'Check the Firebase configuration for this installation.';
    default:
      return code ? `Firebase sign-in failed (${code}).` : String(error);
  }
}
