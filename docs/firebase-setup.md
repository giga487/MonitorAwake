# Firebase setup

Firebase is optional and applies only to notes. Awake mode, timers, alarms and reminders stay local. No Firebase project is bundled with MonitorAwake: each installation supplies its own configuration. Google Analytics and Firebase Hosting are not needed.

## 1. Create a project and register an app

1. Open the [Firebase console](https://console.firebase.google.com/) and create a project. Analytics is optional.
2. Record its unique **Project ID**. The public display name is separate and can be changed without changing the ID.
3. Select **Add app > Web** (`</>`), choose an app nickname such as **MonitorAwake Desktop**, and leave Hosting disabled.
4. Copy the web SDK configuration from **Project settings > General > Your apps**.

The web app registration provides client configuration; it does not create Firestore or upload notes. MonitorAwake already includes the npm Firebase SDK: end users do not need npm, another SDK installation, a hosted website, or service-account credentials.

## 2. Enable authentication

In **Authentication > Sign-in method**, enable **Google**, select a public project name and support email, then save. In **Authentication > Settings > Authorized domains**, add `127.0.0.1` if missing: the desktop sign-in handoff uses a loopback page.

Optionally enable **Email/Password** for the alternative sign-in and account-creation form. This is not required for Google sign-in.

## 3. Create Cloud Firestore and publish rules

Create a **Cloud Firestore Standard** database in **Production** mode with database ID **`(default)`**. A generic “Database” label in the console does not require an extra name. The current client uses the default database; a separately named database is not configurable in this version.

Choose the storage location before creating it. For European storage, select a European location, for example Milan `europe-west8`. App settings do not relocate an existing database.

Open the database's **Rules** tab, replace its contents with [firestore.rules](../firestore.rules), then select **Publish**. Keep these access rules; do not switch to public test rules. Production defaults deny access, so successful authentication alone does not allow note synchronization.

The rules require authenticated note membership, preserve the owner, restrict membership changes to that owner, validate schema/fields and require revision increments. Physical deletion is denied; clients use versioned deletion markers. [firestore.indexes.json](../firestore.indexes.json) contains optional indexing exemptions; the current membership query needs no custom compound index.

For CLI deployment by an authenticated project administrator:

```sh
npx firebase-tools@15.35.0 deploy --only firestore --project YOUR_PROJECT_ID
```

Replace the placeholder locally. Do not commit a personal project selection: `.firebaserc` is ignored. This command publishes rules/index configuration, not the app and not Hosting. Console publication of the supplied rules is sufficient to start using notes.

## 4. Configure the installed app

Right-click the mascot, open **Firebase account > Local Firebase configuration**, and enter JSON using the field names in [firebase.example.json](../firebase.example.json). Map the SDK values as follows:

| Firebase SDK field | MonitorAwake JSON field |
| --- | --- |
| — | `schema_version: 1` |
| `projectId` | `project_id` |
| `apiKey` | `api_key` |
| `authDomain` | `auth_domain` |
| `appId` | `app_id` |
| `storageBucket` | `storage_bucket` (optional) |
| `messagingSenderId` | `messaging_sender_id` (optional) |

Copy the **values**, not the JavaScript `import` or `initializeApp` statements. Keep the real Project ID; do not substitute the app nickname or public project name. Save the configuration in the app.

Alternatively, place your own `firebase.local.json` in the app's data directory while it is closed:

- Windows: `%LOCALAPPDATA%\MonitorAwake\firebase.local.json`.
- Linux: `$XDG_DATA_HOME/MonitorAwake/firebase.local.json`, normally `~/.local/share/MonitorAwake/firebase.local.json`.
- A custom `MONITORAWAKE_DATA_DIR` changes the data directory.

For development, copy `firebase.example.json` to the repository-root `firebase.local.json` as a local reference. **The repository file is not loaded into the installed app or imported into the frontend.** Copy it to the active profile or save through the app panel.

Personal values are excluded from Git and the distributed setup. They are runtime configuration, so changing projects does not require recompiling. The configuration has its own schema (1); Rust validates it, saves atomically, and keeps an exact backup when it changes. Corrupt or future-version files are preserved.

## 5. Sign in and enable synchronization

1. In the native mascot menu, open **Firebase account** and select **Sign in with Google**.
2. Complete the user-initiated system-browser flow and return to MonitorAwake. Alternatively, use email/password if enabled.
3. Open **My notes** and select **Use Firebase notes**.
4. Start with a test note and confirm **Notes synchronized**. You can inspect `notes` in the Firestore console.

Enabling Firebase also uploads existing local notes to the signed-in account. Signing in alone does not upload them; Local is the default storage mode. To test without uploading your current notes, use a separate `MONITORAWAKE_DATA_DIR` profile.

The native WebView keeps the Firebase session in IndexedDB, with localStorage fallback. Passwords and tokens are not stored in `state.json` or `firebase.local.json`. Firebase refreshes ID tokens as needed; tokens are not permanent and revoked sessions may require sign-in again. **Sign out** ends the local session.

The HTTP configuration page uses a different port after restart, so persistent sign-in belongs to the native desktop window. The browser handoff uses in-memory authentication and a random, single-use challenge valid for five minutes. The temporary Google credential stays in memory, is validated through Firebase by the desktop client, and is excluded from snapshots. Restarting during this handoff requires trying again.

## 6. Share a note

Both users must use the same Firebase project. The recipient copies their **Account ID** from **Firebase account**. The owner opens the note's **Cloud sync and sharing** section and adds that ID; an email address is not an Account ID.

The recipient finds the shared note in **My notes**. Up to 20 members, including the owner, are supported. Members can edit or delete the shared note; only the owner changes members. See [Cloud notes](cloud-notes.md) for conflicts, local copies and revocation.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Google reports an unauthorized domain | Enable Google and add `127.0.0.1` to authorized domains. |
| Email sign-in is unavailable | Enable Email/Password, or use Google. |
| Missing/invalid configuration | Check snake_case field mapping, schema 1 and values from the registered web app. |
| Cloud access failed / permission denied | Check database ID `(default)`, the matching project, published rules, and note membership. |
| Signed in but nothing uploads | Select **My notes > Use Firebase notes** in the running desktop app. |
| Changes remain queued | Check connectivity and the storage status. Changes stay saved locally. |
| Conflicting edits | Open the note and choose **Keep my version** or **Use cloud version**. |
| Switching accounts is refused | Close note editors and use **Cloud account links** to disconnect the previous cache first. |

Do not solve access errors by making database rules public. Do not put an administrator key or a copied login token in app configuration.

## Local emulator tests

Use a fictitious project, never the personal project, for automated tests. Java 21 and Node.js are required by the CLI used here:

```sh
npm ci
npx --yes firebase-tools@15.35.0 emulators:exec --project demo-monitorawake --only auth,firestore "npm run test:firebase && npm run test:firestore"
```

The CLI sets `FIREBASE_AUTH_EMULATOR_HOST` and `FIRESTORE_EMULATOR_HOST` for the test command. The repository's `firebase.json` binds Auth to `127.0.0.1:19099`, Firestore to `127.0.0.1:19080`, and disables emulator UI.

For a native app emulator test, use a dedicated data/WebView profile and fictitious Firebase configuration. Set process-local `MONITORAWAKE_AUTH_EMULATOR_URL=http://127.0.0.1:19099` and `MONITORAWAKE_FIRESTORE_EMULATOR_URL=http://127.0.0.1:19080`. Only loopback emulator addresses are accepted. Do not set these variables for production use.

Windows verification on November 10, 2026 covered emulator authentication/session restoration, SDK transactions and rules, queued local changes across restart and resumed synchronization. No browser was opened for testing. Real Google sign-in, the production database and desktop Linux remain separate checks.

The SDK is pinned through the npm lockfile. An override of `@grpc/grpc-js` to the corrected 1.14.7 series avoids a vulnerable transitive version. At the recorded audit, production dependencies had no reported vulnerabilities; two low-severity development-only `deepmerge` findings remained without an available fix.

References: [Firebase web setup](https://firebase.google.com/docs/web/setup), [Google authentication](https://firebase.google.com/docs/auth/web/google-signin), [Firestore databases](https://firebase.google.com/docs/firestore/manage-databases), [Firestore security rules](https://firebase.google.com/docs/firestore/security/get-started), [Firebase CLI](https://firebase.google.com/docs/cli#partial_deploys).
