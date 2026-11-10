<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onAuthStateChanged, type Auth, type User } from 'firebase/auth';
  import {
    collection,
    query,
    where,
    onSnapshot,
    type Firestore,
  } from 'firebase/firestore';
  import {
    getState,
    dispatch,
    getFirebaseSetup,
    type CloudScope,
    type CloudNote,
    type CloudOperation,
  } from './api';
  import {
    initializeNotesAuth,
    disconnectNotesFirebase,
  } from './firebase-auth';
  import {
    notesFirestore,
    validCloudNote,
    writeCloudNote,
  } from './firebase-cloud';
  onMount(() => {
    let stopped = false,
      generation = 0,
      fingerprint = '',
      auth: Auth | null = null,
      user: User | null = null,
      client: Firestore | null = null;
    let authUnsubscribe: (() => void) | undefined,
      cloudUnsubscribe: (() => void) | undefined;
    let timer: ReturnType<typeof setTimeout>;
    let remote = new Map<string, CloudNote>(),
      removed = new Set<string>(),
      receivedServer = false;
    let cloudError = '';
    let retryAt = 0;
    const report = (message: string, pending = 0) =>
      invoke('set_note_sync_status', {
        status: { message: message.slice(0, 1000), pending },
      });
    function stopCloud() {
      generation++;
      cloudUnsubscribe?.();
      cloudUnsubscribe = undefined;
      remote.clear();
      removed.clear();
      receivedServer = false;
      cloudError = '';
      retryAt = 0;
    }
    async function tick() {
      try {
        let state = await getState();
        if (stopped) return;
        if (state.settings.notes_storage !== 'firebase') {
          stopCloud();
          await report('Notes are stored locally.');
          return;
        }
        const setup = await getFirebaseSetup();
        if (!setup.config) {
          stopCloud();
          await report('Configure Firebase from the mascot account menu.');
          return;
        }
        const nextFingerprint = JSON.stringify(setup);
        if (nextFingerprint !== fingerprint || !auth) {
          stopCloud();
          authUnsubscribe?.();
          if (auth) await disconnectNotesFirebase();
          auth = await initializeNotesAuth(setup.config, {
            emulatorUrl: setup.auth_emulator_url ?? undefined,
          });
          client = notesFirestore(
            auth.app,
            setup.firestore_emulator_url ?? undefined,
          );
          user = auth.currentUser;
          fingerprint = nextFingerprint;
          authUnsubscribe = onAuthStateChanged(auth, (next) => {
            if (user?.uid !== next?.uid) {
              stopCloud();
              user = next;
            }
          });
        }
        if (!user || !client) {
          stopCloud();
          await report('Sign in from the mascot menu → Firebase account.');
          return;
        }
        const scope: CloudScope = {
          project_id: setup.config.project_id,
          uid: user.uid,
        };
        if (JSON.stringify(state.note_sync.scope) !== JSON.stringify(scope)) {
          state = (await dispatch({ type: 'bind_notes_cloud', scope })).state;
        }
        if (!cloudUnsubscribe) {
          if (Date.now() < retryAt) {
            await report(cloudError || 'Waiting to reconnect…');
            return;
          }
          const current = generation;
          cloudUnsubscribe = onSnapshot(
            query(
              collection(client, 'notes'),
              where('member_uids', 'array-contains', user.uid),
            ),
            { includeMetadataChanges: true },
            (snapshot) => {
              if (
                stopped ||
                current !== generation ||
                snapshot.metadata.fromCache
              )
                return;
              receivedServer = true;
              cloudError = '';
              const next = new Map<string, CloudNote>();
              for (const item of snapshot.docs) {
                const value = item.data();
                if (!validCloudNote(value, item.id)) {
                  cloudError =
                    'Unsupported cloud note data. Your local notes were kept.';
                  continue;
                }
                next.set(item.id, value);
              }
              for (const id of remote.keys())
                if (!next.has(id)) removed.add(id);
              remote = next;
            },
            (reason) => {
              cloudError = `Cloud access failed (${reason.code}). Check the database and its security rules.`;
              cloudUnsubscribe = undefined;
              retryAt = Date.now() + 15000;
            },
          );
        }
        if (cloudError) {
          await report(cloudError);
          return;
        }
        const current = generation;
        for (const id of [...removed]) {
          await dispatch({ type: 'cloud_note_access_removed', scope, id });
          removed.delete(id);
        }
        if (receivedServer) {
          for (const document of [...remote.values()]) {
            const entry = state.note_sync.entries.find(
              (entry) => entry.id === document.id,
            );
            if (entry && entry.base.revision >= document.revision) continue;
            if (
              document.deleted &&
              !entry &&
              !state.notes.some((note) => note.id === document.id)
            )
              continue;
            state = (
              await dispatch({ type: 'apply_cloud_note', scope, document })
            ).state;
          }
        }
        const operations = await invoke<CloudOperation[]>(
          'note_cloud_operations',
          { scope },
        );
        if (!navigator.onLine) {
          await report(
            'Offline. Changes are saved on this computer and waiting to sync.',
            operations.length,
          );
          return;
        }
        await report(
          operations.length
            ? 'Syncing notes…'
            : 'Connected. Checking shared notes…',
          operations.length,
        );
        for (const operation of operations.slice(0, 10)) {
          if (stopped || current !== generation) return;
          try {
            const result = await writeCloudNote(client, operation);
            if (stopped || current !== generation) return;
            await dispatch({
              type: result.conflict
                ? 'apply_cloud_note'
                : 'acknowledge_cloud_note',
              scope,
              document: result.document,
            });
            if (
              (remote.get(result.document.id)?.revision ?? 0) <=
              result.document.revision
            )
              remote.set(result.document.id, result.document);
          } catch (reason) {
            const code =
              reason && typeof reason === 'object' && 'code' in reason
                ? String(reason.code)
                : '';
            if (
              code === 'permission-denied' &&
              operation.expected_revision !== null
            )
              await dispatch({
                type: 'cloud_note_access_removed',
                scope,
                id: operation.document.id,
              });
            throw reason;
          }
        }
        const latest = await getState();
        const conflicts = latest.note_sync.entries.filter(
          (entry) => entry.conflict,
        ).length;
        const pending = (
          await invoke<CloudOperation[]>('note_cloud_operations', { scope })
        ).length;
        await report(
          conflicts
            ? 'Conflicting changes need your review.'
            : pending
              ? 'Changes queued for synchronization.'
              : receivedServer
                ? 'Notes synchronized.'
                : 'Waiting for the cloud connection…',
          pending,
        );
      } catch (reason) {
        if (!stopped)
          await report(`Synchronization paused: ${String(reason)}`).catch(
            () => {},
          );
      } finally {
        if (!stopped) timer = setTimeout(tick, 2000);
      }
    }
    void tick();
    return () => {
      stopped = true;
      clearTimeout(timer);
      stopCloud();
      authUnsubscribe?.();
    };
  });
</script>
