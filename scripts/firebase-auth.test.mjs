import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  signOut,
  inMemoryPersistence,
  signInWithCredential,
  GoogleAuthProvider,
} from 'firebase/auth';
import {
  initializeNotesAuth,
  disconnectNotesFirebase,
  firebaseAuthError,
} from '../src/lib/firebase-auth.ts';

test('authentication errors describe next actions without exposing server details', () => {
  assert.match(
    firebaseAuthError({
      code: 'auth/operation-not-allowed',
      message: 'internal detail',
    }),
    /Enable Email\/Password/,
  );
  assert.equal(
    firebaseAuthError({ code: 'auth/invalid-credential' }),
    'Email or password is incorrect.',
  );
  assert.match(
    firebaseAuthError({ code: 'auth/network-request-failed' }),
    /connection/,
  );
  assert.match(
    firebaseAuthError({ code: 'auth/user-token-expired' }),
    /Sign in again/,
  );
});

test(
  'Auth emulator: create account, reuse client, refresh token, sign out and sign in again',
  {
    skip: !process.env.FIREBASE_AUTH_EMULATOR_HOST,
  },
  async () => {
    const address = process.env.FIREBASE_AUTH_EMULATOR_HOST;
    assert.match(address, /^127\.0\.0\.1:\d+$/);
    const auth = await initializeNotesAuth(
      {
        schema_version: 1,
        project_id: 'demo-monitorawake',
        api_key: 'demo-api-key',
        auth_domain: 'demo-monitorawake.firebaseapp.com',
        app_id: '1:123:web:auth-test',
      },
      {
        emulatorUrl: `http://${address}`,
        persistence: inMemoryPersistence,
      },
    );
    const email = `sdk-${crypto.randomUUID()}@example.test`;
    const password = `Test-${crypto.randomUUID()}!`;
    try {
      const created = await createUserWithEmailAndPassword(
        auth,
        email,
        password,
      );
      const id = created.user.uid;
      const token = await created.user.getIdToken(true);
      assert.ok(token.length > 20);
      assert.equal(auth.currentUser.uid, id);
      await signOut(auth);
      assert.equal(auth.currentUser, null);
      await assert.rejects(
        signInWithEmailAndPassword(auth, email, 'wrong-password'),
      );
      const signedIn = await signInWithEmailAndPassword(auth, email, password);
      assert.equal(signedIn.user.uid, id);
      assert.ok((await signedIn.user.getIdToken(true)).length > 20);
      await signOut(auth);
      const header = Buffer.from(
        JSON.stringify({ alg: 'none', typ: 'JWT' }),
      ).toString('base64url');
      const payload = Buffer.from(
        JSON.stringify({
          sub: crypto.randomUUID(),
          email: `google-${crypto.randomUUID()}@example.test`,
          email_verified: true,
          iss: 'https://accounts.google.com',
          aud: 'test-client',
          iat: Math.floor(Date.now() / 1000),
          exp: Math.floor(Date.now() / 1000) + 3600,
        }),
      ).toString('base64url');
      const googleUser = await signInWithCredential(
        auth,
        GoogleAuthProvider.credential(`${header}.${payload}.`),
      );
      assert.ok(
        googleUser.user.providerData.some(
          (provider) => provider.providerId === 'google.com',
        ),
      );
      assert.ok((await googleUser.user.getIdToken(true)).length > 20);
    } finally {
      await disconnectNotesFirebase();
    }
  },
);
