# Local and Firebase notes

Notes are local by default. Configure and sign in through **Firebase account**, then select **My notes > Use Firebase notes** to explicitly enable synchronization, including existing local notes. Awake mode, timers and alarms remain local. Firebase Hosting is not required.

Follow the [Firebase setup guide](firebase-setup.md) for project creation, Google authentication, the default Firestore database, rule publication and local SDK configuration. Personal project values stay in the ignored `firebase.local.json`; they are not bundled in releases.

## Sharing and conflicts

A note has one owner and up to 20 members in total. The owner opens **Cloud sync and sharing** and adds the other user's **Account ID**, available in their Firebase account panel. Both users must use the same project. An email address is not an Account ID.

Members can edit the note's content or delete the shared note; only the owner can change membership. Deleting a shared note affects everyone.

Concurrent edits to the same base revision preserve both local and cloud versions. The editor offers **Keep my version** and **Use cloud version**. Revocation stops further updates but preserves an already downloaded local copy; it cannot erase copies retroactively.

## Persistence and recovery

Rust atomically saves note content, cloud metadata and pending changes in `state.json`, including while offline. The SDK uses an in-memory cache; the Rust model owns durable storage. On reconnection, Firestore transactions compare revisions. An upload committed before a crash is recognized without duplicating the note.

**Use local notes** pauses synchronization while retaining links for later resumption. To switch accounts/projects, close note editors and use the disconnect option under **Cloud account links**. It retains local copies with new IDs without modifying the previous cloud. The app refuses to upload a cache bound to another account automatically.

Title, body, color and membership are shared. Position, size, visibility, pinning and reminders belong to each device. Received notes appear in **My notes** without automatically opening windows. Limits: 100 local notes, 80-character titles and 10,000-character bodies. Errors and limits pause synchronization while preserving local data.

## Versions and access control

Local state uses schema 8. Firebase configuration and cloud documents each use schema 1. Documents at `notes/{uuid}` contain owner, members, revision and a deletion flag. Deletion is logical; this version does not purge historical cloud documents.

Rules validate fields, limits, membership, ownership and revision increments, and deny all other collections. Do not use public rules. Passwords and tokens never enter note documents. Authentication sessions belong to the WebView profile and are managed by the SDK.

The main desktop window runs synchronization. Opening only the standalone HTTP configuration page does not start that worker.

## Verification

Rust tests cover queued edits, conflicts, deletion and recovery. Auth/Firestore emulator tests cover transactions and rules for an owner, collaborator and outsider; these tests do not deploy production rules.

Windows verification on November 10, 2026: 42 Rust tests, 16 widget tests, 5 Auth/config tests and 2 Firestore tests passed; Svelte, Clippy and isolated HTTP checks passed. Native WebView tests with a fictitious emulator account verified initial upload, paused local edits, restart and upload on resumption, remote updates and deletion markers. Physical network disconnection was not simulated. Personal notes were not used.

Real Google login, production database access and desktop Linux behavior require separate verification.
