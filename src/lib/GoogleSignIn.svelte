<script lang="ts">
  import { onMount } from 'svelte';
  import {
    GoogleAuthProvider,
    signInWithPopup,
    browserPopupRedirectResolver,
    inMemoryPersistence,
    signOut,
    type Auth,
  } from 'firebase/auth';
  import {
    getFirebaseSetup,
    completeGoogleLogin,
    initializeSession,
  } from './api';
  import { initializeNotesAuth, firebaseAuthError } from './firebase-auth';
  let { challenge }: { challenge: string } = $props();
  let auth = $state.raw<Auth | null>(null);
  let busy = $state(false);
  let done = $state(false);
  let error = $state('');
  onMount(() => {
    // Child onMount runs before App's onMount: initialize the local API session here.
    if (!initializeSession()) {
      error = 'Reopen Google sign-in from the desktop app.';
      return;
    }
    let stopped = false;
    void getFirebaseSetup()
      .then(async (setup) => {
        if (!setup.config)
          throw new Error(
            'Save the Firebase configuration in the desktop app first.',
          );
        const next = await initializeNotesAuth(setup.config, {
          persistence: inMemoryPersistence,
        });
        if (!stopped) auth = next;
      })
      .catch((reason) => {
        if (!stopped) error = firebaseAuthError(reason);
      });
    return () => {
      stopped = true;
    };
  });
  async function login() {
    if (!auth || busy || done) return;
    busy = true;
    error = '';
    try {
      const provider = new GoogleAuthProvider();
      provider.setCustomParameters({ prompt: 'select_account' });
      const result = await signInWithPopup(
        auth,
        provider,
        browserPopupRedirectResolver,
      );
      const credential = GoogleAuthProvider.credentialFromResult(result);
      if (!credential?.idToken)
        throw new Error(
          'Google did not return a sign-in credential. Please try again.',
        );
      await completeGoogleLogin(challenge, credential.idToken);
      done = true;
    } catch (reason) {
      error = firebaseAuthError(reason);
    } finally {
      if (auth) await signOut(auth).catch(() => {});
      busy = false;
    }
  }
</script>

<main class="google-login">
  <section class="panel">
    <h1>Sign in to MonitorAwake</h1>
    {#if done}<p role="status">
        Sign-in sent to the desktop app. Return to MonitorAwake to check your
        account. You can close this tab.
      </p>
    {:else}<p>Continue with your Google account to connect this device.</p>
      <button class="primary" disabled={!auth || busy} onclick={login}
        >Continue with Google</button
      >
    {/if}
    {#if error}<p role="alert">{error}</p>{/if}
  </section>
</main>

<style>
  .google-login {
    max-width: 560px;
    margin: 12vh auto;
    padding: 20px;
  }
  h1 {
    font-size: 24px;
    margin-bottom: 18px;
  }
  p {
    line-height: 1.7;
    margin: 16px 0;
  }
  [role='alert'] {
    color: #8b4038;
  }
</style>
