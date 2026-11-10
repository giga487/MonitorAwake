import test from 'node:test';
import assert from 'node:assert/strict';
import { deleteApp, getApps } from 'firebase/app';
import {
  firebaseOptions,
  initializeNotesFirebase,
} from '../src/lib/firebase.ts';

const config = {
  schema_version: 1,
  project_id: 'demo-notes',
  api_key: 'test-api-key',
  auth_domain: 'demo-notes.firebaseapp.com',
  app_id: '1:123:web:test',
};

test('runtime configuration maps to SDK options without changing the source', () => {
  const original = structuredClone(config);
  assert.deepEqual(firebaseOptions(config), {
    projectId: 'demo-notes',
    apiKey: 'test-api-key',
    authDomain: 'demo-notes.firebaseapp.com',
    appId: '1:123:web:test',
  });
  assert.deepEqual(config, original);
});

test('future and incomplete configurations are rejected before initializing a client', () => {
  const before = getApps().length;
  for (const value of [
    null,
    [],
    {},
    { ...config, schema_version: 2 },
    { ...config, api_key: '' },
    { schema_version: 1, project_id: 'demo-notes' },
    { ...config, storage_bucket: 123 },
  ]) {
    assert.throws(() => initializeNotesFirebase(value));
  }
  assert.equal(getApps().length, before);
});

test('SDK initialization reuses the same client and refuses silent project switches', async () => {
  const app = initializeNotesFirebase(config);
  try {
    assert.strictEqual(initializeNotesFirebase(config), app);
    assert.equal(app.options.projectId, 'demo-notes');
    assert.equal(app.automaticDataCollectionEnabled, false);
    assert.throws(() =>
      initializeNotesFirebase({ ...config, project_id: 'demo-other-notes' }),
    );
    assert.equal(app.options.projectId, 'demo-notes');
  } finally {
    await deleteApp(app);
  }
});
