<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    createUserWithEmailAndPassword,
    signInWithEmailAndPassword,
    signOut,
    signInWithCredential,
    GoogleAuthProvider,
    onAuthStateChanged,
    type Auth,
    type User,
  } from 'firebase/auth';
  import { desktop, getFirebaseSetup, saveFirebaseConfig } from './api';
  import { firebaseOptions, type FirebaseLocalConfig } from './firebase';
  import {
    initializeNotesAuth,
    disconnectNotesFirebase,
    firebaseAuthError,
  } from './firebase-auth';

  let loading = $state(true);
  let busy = $state(false);
  let error = $state('');
  let message = $state('');
  let configText = $state('');
  let configured = $state(false);
  let email = $state('');
  let password = $state('');
  let emailLogin = $state(false);
  let googleWaiting = $state(false);
  let googleAttempt = 0;
  let auth = $state.raw<Auth | null>(null);
  let user = $state.raw<User | null>(null);
  let stopped = false;
  let unsubscribe: (() => void) | undefined;

  async function load() {
    loading = true;
    try {
      const setup = await getFirebaseSetup();
      if (stopped) return;
      configured = !!setup.config;
      configText = setup.config ? JSON.stringify(setup.config, null, 2) : '';
      if (desktop && setup.config) {
        const next = await initializeNotesAuth(setup.config, {
          emulatorUrl: setup.auth_emulator_url ?? undefined,
        });
        if (stopped) return;
        auth = next;
        unsubscribe?.();
        unsubscribe = onAuthStateChanged(
          next,
          (value) => {
            user = value;
          },
          (reason) => {
            error = firebaseAuthError(reason);
          },
        );
      }
    } catch (reason) {
      if (!stopped) error = firebaseAuthError(reason);
    } finally {
      if (!stopped) loading = false;
    }
  }
  onMount(() => {
    void load();
    return () => {
      stopped = true;
      googleAttempt++;
      unsubscribe?.();
      password = '';
    };
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    error = '';
    message = '';
    try {
      const config = JSON.parse(configText) as FirebaseLocalConfig;
      firebaseOptions(config);
      await saveFirebaseConfig(config);
      unsubscribe?.();
      unsubscribe = undefined;
      await disconnectNotesFirebase();
      auth = null;
      user = null;
      password = '';
      await load();
      if (!error) message = 'Configuration saved on this computer.';
    } catch (reason) {
      error = firebaseAuthError(reason);
    } finally {
      busy = false;
    }
  }
  async function login(create = false) {
    if (!auth || busy) return;
    busy = true;
    error = '';
    message = '';
    try {
      const result = create
        ? await createUserWithEmailAndPassword(auth, email.trim(), password)
        : await signInWithEmailAndPassword(auth, email.trim(), password);
      user = result.user;
      message = create
        ? 'Account created. You are signed in.'
        : 'Signed in. This device will remember your account.';
    } catch (reason) {
      error = firebaseAuthError(reason);
    } finally {
      password = '';
      busy = false;
    }
  }
  async function logout() {
    if (!auth || busy) return;
    busy = true;
    error = '';
    try {
      await signOut(auth);
      user = null;
      password = '';
      message = 'Signed out on this device.';
    } catch (reason) {
      error = firebaseAuthError(reason);
    } finally {
      busy = false;
    }
  }

  async function googleLogin() {
    if (!auth || busy) return;
    const attempt = ++googleAttempt;
    busy = true;
    googleWaiting = true;
    error = '';
    message = 'Finish Google sign-in in your browser.';
    try {
      const challenge = await invoke<string>('begin_google_login');
      while (!stopped && googleAttempt === attempt) {
        const credential = await invoke<{ id_token: string } | null>(
          'take_google_login',
          { challenge },
        );
        if (credential) {
          googleWaiting = false;
          const result = await signInWithCredential(
            auth,
            GoogleAuthProvider.credential(credential.id_token),
          );
          user = result.user;
          message =
            'Signed in with Google. This device will remember your account.';
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 800));
      }
    } catch (reason) {
      if (!stopped && googleAttempt === attempt)
        error = firebaseAuthError(reason);
    } finally {
      if (!stopped && googleAttempt === attempt) {
        busy = false;
        googleWaiting = false;
      }
    }
  }
  function cancelGoogleLogin() {
    googleAttempt++;
    googleWaiting = false;
    busy = false;
    message = 'Google sign-in cancelled on this device.';
  }
</script>

<section class="firebase-account" aria-label="Firebase account">
  <h2>Firebase account</h2>
  <p class="hint">
    Connect your account, then choose Firebase storage in My notes to
    synchronize.
  </p>
  {#if loading}<p role="status">Restoring account…</p>
  {:else if !desktop}<p class="hint">
      Sign in from the mascot menu → Firebase account. The desktop app remembers
      your account across restarts.
    </p>
  {:else if user}
    <p>Signed in as <strong>{user.email ?? 'Firebase user'}</strong></p>
    <label for="firebase-account-id">Your account ID for sharing</label><input
      id="firebase-account-id"
      readonly
      value={user.uid}
      onclick={(event) => event.currentTarget.select()}
    />
    <p class="hint">
      This device remembers your session and renews it automatically.
    </p>
    <button class="secondary" disabled={busy} onclick={logout}>Sign out</button>
  {:else if auth}
    <button class="primary" disabled={busy} onclick={googleLogin}
      >Sign in with Google</button
    >
    {#if googleWaiting}<button class="text-button" onclick={cancelGoogleLogin}
        >Cancel sign-in</button
      >{/if}
    <button
      class="text-button"
      disabled={busy}
      onclick={() => (emailLogin = !emailLogin)}>Use email and password</button
    >
    {#if emailLogin}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void login();
        }}
      >
        <label for="firebase-email">Email</label>
        <input
          id="firebase-email"
          type="email"
          bind:value={email}
          autocomplete="username"
          required
        />
        <label for="firebase-password">Password</label>
        <input
          id="firebase-password"
          type="password"
          bind:value={password}
          autocomplete="current-password"
          required
        />
        <p class="hint">
          Use your Firebase account password. Sign in once to stay connected.
        </p>
        <div class="actions">
          <button class="primary" disabled={busy || !email.trim() || !password}
            >Sign in</button
          >
          <button
            class="text-button"
            type="button"
            disabled={busy || !email.trim() || !password}
            onclick={() => login(true)}>Create account</button
          >
        </div>
      </form>
    {/if}
  {:else}<p class="hint">
      Add this installation’s Firebase configuration below to get started.
    </p>{/if}
  {#if error}<p class="account-error" role="alert">{error}</p>{/if}
  <p class="hint" aria-live="polite">{message}</p>
  <details open={!configured}>
    <summary>Local Firebase configuration</summary>
    <form onsubmit={save}>
      <label for="firebase-config">Configuration JSON</label>
      <textarea
        id="firebase-config"
        bind:value={configText}
        rows="7"
        maxlength="16000"
        spellcheck="false"
        placeholder="Paste firebase.local.json here"
        required></textarea>
      <p class="hint">
        Saved in your local app profile. Changing this configuration signs this
        device out.
      </p>
      <button class="secondary" disabled={busy || loading || !configText.trim()}
        >Save configuration</button
      >
    </form>
  </details>
</section>

<style>
  .firebase-account {
    font-size: 13px;
  }
  h2 {
    font-size: 16px;
    margin: 10px 0;
  }
  form {
    display: grid;
    gap: 8px;
    margin: 12px 0;
  }
  label {
    font-size: 12px;
  }
  input,
  textarea {
    min-width: 0;
    width: 100%;
    box-sizing: border-box;
    border: 1px solid #dbe4d1;
    border-radius: 9px;
    padding: 10px;
    color: #303b34;
    background: white;
    font: inherit;
  }
  textarea {
    font-family: monospace;
    resize: vertical;
  }
  .hint {
    font-size: 11px;
    line-height: 1.6;
    color: #657269;
    margin: 8px 0;
  }
  .hint:empty {
    display: none;
  }
  .actions {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
  }
  .account-error {
    font-size: 12px;
    color: #8b4038;
    overflow-wrap: anywhere;
  }
  details {
    margin-top: 20px;
    border-top: 1px solid #e6ecdd;
    padding-top: 14px;
  }
  summary {
    cursor: pointer;
    font-size: 12px;
  }
  strong {
    overflow-wrap: anywhere;
  }
</style>
