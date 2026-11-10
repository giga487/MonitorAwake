import {
  initializeFirestore,
  connectFirestoreEmulator,
  runTransaction,
  doc,
  type Firestore,
} from 'firebase/firestore';
import type { FirebaseApp } from 'firebase/app';
import type { CloudNote, CloudOperation } from './api';

const clients = new WeakMap<FirebaseApp, Firestore>();
export function notesFirestore(
  app: FirebaseApp,
  emulatorUrl?: string,
): Firestore {
  const existing = clients.get(app);
  if (existing) return existing;
  // Rust owns the durable offline queue; transactions never silently overwrite offline changes.
  const client = initializeFirestore(app, {
    experimentalAutoDetectLongPolling: true,
  });
  if (emulatorUrl) {
    if (!/^http:\/\/127\.0\.0\.1:[1-9]\d{0,4}$/.test(emulatorUrl))
      throw new Error('Invalid Firestore emulator address.');
    connectFirestoreEmulator(
      client,
      '127.0.0.1',
      Number(new URL(emulatorUrl).port),
    );
  }
  clients.set(app, client);
  return client;
}
export function sameCloudContent(left: CloudNote, right: CloudNote): boolean {
  return (
    left.id === right.id &&
    left.title === right.title &&
    left.body === right.body &&
    left.color === right.color &&
    left.owner_uid === right.owner_uid &&
    left.deleted === right.deleted &&
    JSON.stringify(left.member_uids) === JSON.stringify(right.member_uids)
  );
}
export function validCloudNote(value: unknown, id: string): value is CloudNote {
  if (!value || typeof value !== 'object') return false;
  const note = value as CloudNote;
  return (
    note.schema_version === 1 &&
    note.id === id &&
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(
      id,
    ) &&
    typeof note.title === 'string' &&
    [...note.title].length <= 80 &&
    typeof note.body === 'string' &&
    [...note.body].length <= 10000 &&
    ['yellow', 'sage', 'blue', 'rose'].includes(note.color) &&
    typeof note.owner_uid === 'string' &&
    Array.isArray(note.member_uids) &&
    note.member_uids.length > 0 &&
    note.member_uids.length <= 20 &&
    note.member_uids.every(
      (uid) => typeof uid === 'string' && uid.length > 0 && uid.length <= 128,
    ) &&
    note.member_uids.includes(note.owner_uid) &&
    new Set(note.member_uids).size === note.member_uids.length &&
    Number.isSafeInteger(note.revision) &&
    note.revision > 0 &&
    typeof note.deleted === 'boolean'
  );
}
export async function writeCloudNote(
  client: Firestore,
  operation: CloudOperation,
): Promise<{ conflict: boolean; document: CloudNote }> {
  const reference = doc(client, 'notes', operation.document.id);
  return runTransaction(client, async (transaction) => {
    const snapshot = await transaction.get(reference);
    if (snapshot.exists()) {
      const remote = snapshot.data();
      if (!validCloudNote(remote, snapshot.id))
        throw new Error(
          'Unsupported cloud note schema. No data was overwritten.',
        );
      if (sameCloudContent(remote, operation.document))
        return { conflict: false, document: remote };
      if (operation.expected_revision !== remote.revision)
        return { conflict: true, document: remote };
    } else if (operation.expected_revision !== null) {
      throw new Error('The cloud note was removed. Your local copy was kept.');
    }
    transaction.set(reference, operation.document);
    return { conflict: false, document: operation.document };
  });
}
