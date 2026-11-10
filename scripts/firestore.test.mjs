import test from 'node:test';
import assert from 'node:assert/strict';
import { initializeApp, deleteApp } from 'firebase/app';
import {
  initializeAuth,
  inMemoryPersistence,
  connectAuthEmulator,
  createUserWithEmailAndPassword,
} from 'firebase/auth';
import {
  doc,
  getDoc,
  setDoc,
  deleteDoc,
  getDocs,
  collection,
  query,
  where,
  terminate,
  setLogLevel,
} from 'firebase/firestore';
import {
  notesFirestore,
  writeCloudNote,
  validCloudNote,
} from '../src/lib/firebase-cloud.ts';

setLogLevel('silent');

test('cloud document validation rejects unsupported schemas and malformed members', () => {
  assert.equal(
    validCloudNote({ schema_version: 99 }, crypto.randomUUID()),
    false,
  );
  assert.equal(validCloudNote({}, 'not-a-uuid'), false);
});

test(
  'Firestore emulator: owner/editor permissions, revisions, conflicts, sharing revocation and deletion',
  {
    skip:
      !process.env.FIREBASE_AUTH_EMULATOR_HOST ||
      !process.env.FIRESTORE_EMULATOR_HOST,
  },
  async () => {
    const authHost = process.env.FIREBASE_AUTH_EMULATOR_HOST,
      firestoreHost = process.env.FIRESTORE_EMULATOR_HOST;
    assert.match(authHost, /^127\.0\.0\.1:\d+$/);
    assert.match(firestoreHost, /^127\.0\.0\.1:\d+$/);
    const actors = [];
    for (const label of ['owner', 'editor', 'outsider']) {
      const app = initializeApp(
        {
          projectId: 'demo-monitorawake',
          apiKey: 'demo-api-key',
          appId: 'test-cloud-app',
        },
        `${label}-${crypto.randomUUID()}`,
      );
      const auth = initializeAuth(app, { persistence: inMemoryPersistence });
      connectAuthEmulator(auth, `http://${authHost}`, {
        disableWarnings: true,
      });
      const { user } = await createUserWithEmailAndPassword(
        auth,
        `${label}-${crypto.randomUUID()}@example.test`,
        'Emulator-only-password!',
      );
      actors.push({
        app,
        auth,
        user,
        db: notesFirestore(app, `http://${firestoreHost}`),
      });
    }
    const [owner, editor, outsider] = actors;
    const id = crypto.randomUUID();
    let note = {
      schema_version: 1,
      id,
      title: 'Shared thought',
      body: 'Private to owner',
      color: 'yellow',
      owner_uid: owner.user.uid,
      member_uids: [owner.user.uid],
      revision: 1,
      deleted: false,
    };
    try {
      assert.equal(
        (
          await writeCloudNote(owner.db, {
            document: note,
            expected_revision: null,
          })
        ).conflict,
        false,
      );
      assert.equal(
        (await getDoc(doc(owner.db, 'notes', id))).data().body,
        'Private to owner',
      );
      await assert.rejects(getDoc(doc(outsider.db, 'notes', id)));
      await assert.rejects(getDocs(collection(owner.db, 'notes')));
      assert.equal(
        (
          await getDocs(
            query(
              collection(owner.db, 'notes'),
              where('member_uids', 'array-contains', owner.user.uid),
            ),
          )
        ).docs.some((item) => item.id === id),
        true,
      );
      await assert.rejects(
        setDoc(doc(owner.db, 'notes', id), {
          ...note,
          revision: 2,
          schema_version: 99,
        }),
      );
      await assert.rejects(
        setDoc(doc(owner.db, 'notes', id), {
          ...note,
          revision: 2,
          bounds: { x: 1, y: 1 },
        }),
      );
      note = {
        ...note,
        revision: 2,
        member_uids: [owner.user.uid, editor.user.uid],
      };
      await writeCloudNote(owner.db, { document: note, expected_revision: 1 });
      assert.equal(
        (await getDoc(doc(editor.db, 'notes', id))).data().revision,
        2,
      );
      await assert.rejects(
        setDoc(doc(editor.db, 'notes', id), {
          ...note,
          revision: 3,
          owner_uid: editor.user.uid,
        }),
      );
      await assert.rejects(
        setDoc(doc(editor.db, 'notes', id), {
          ...note,
          revision: 3,
          member_uids: [...note.member_uids, outsider.user.uid],
        }),
      );
      const stale = { ...note, revision: 3, body: 'Stale edit' };
      note = { ...note, revision: 3, body: 'Editor changed this' };
      await writeCloudNote(editor.db, { document: note, expected_revision: 2 });
      const conflict = await writeCloudNote(owner.db, {
        document: stale,
        expected_revision: 2,
      });
      assert.equal(conflict.conflict, true);
      assert.equal(conflict.document.body, 'Editor changed this');
      assert.equal(
        (
          await writeCloudNote(editor.db, {
            document: note,
            expected_revision: 2,
          })
        ).conflict,
        false,
      );
      note = { ...note, revision: 4, member_uids: [owner.user.uid] };
      await writeCloudNote(owner.db, { document: note, expected_revision: 3 });
      await assert.rejects(getDoc(doc(editor.db, 'notes', id)));
      await assert.rejects(deleteDoc(doc(owner.db, 'notes', id)));
      note = { ...note, revision: 5, deleted: true };
      await writeCloudNote(owner.db, { document: note, expected_revision: 4 });
      assert.equal(
        (await getDoc(doc(owner.db, 'notes', id))).data().deleted,
        true,
      );
    } finally {
      for (const actor of actors) {
        await terminate(actor.db);
        await deleteApp(actor.app);
      }
    }
  },
);
