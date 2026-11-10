# Desktop notes and recurring phrases

MonitorAwake has a neutral identity. Its app icon is `public/app-icon.svg` (note and clock); Chicken is a selectable mascot, not the app's data-format brand.

## Notes

**New note** in the mascot menu creates an independent window. **My notes** lists notes and lets you open, show all or hide them. The HTTP configuration page provides the same section and editing through the local server.

Notes have a title, plain text, four colors and an always-on-top pin. Drag the title bar and resize using the window edges. Closing hides a note without deleting it. The trash button requires separate confirmation. Only notes left visible are restored as windows at startup.

Autosave runs after a short typing pause. **Saving…**, **Unsaved** and **Saved** show the local save state; Firebase mode says **Saved locally**, with synchronization status shown separately. A stale editor never silently overwrites newer text: reload the saved version or explicitly choose **Save my version**. Geometry and appearance save separately from text revisions, avoiding conflicts when moving a window.

Before explicit app exit, note windows receive a save request. If a window does not acknowledge or a write fails, the app stays open and shows an error. Forced termination can still lose changes that have not finished saving.

**Add a reminder** links a timer to a note. The Rust core owns its deadline, and it appears as an egg like other timers. Notes without reminders have no eggs. Hiding does not cancel reminders; deletion cancels the linked timer or acknowledges an expired reminder. History cleanup also removes obsolete links.

Native windows and HTTP APIs share one Rust service. Restored coordinates are clamped to available screens. On Linux, especially Wayland, pinning and positioning depend on the compositor; if placement is unavailable, the window still opens. Compilation does not verify desktop Linux rendering.

## Optional cloud storage

Local is the default. Configure a project and account, publish the supplied rules, then select **My notes > Use Firebase notes**. See [Firebase setup](firebase-setup.md) and [Cloud notes](cloud-notes.md).

Cloud mode shares title, body and color. Window layout, visibility, pinning and reminders remain device-local. Shared-note deletion affects all members. Cloud revision conflicts are separate from stale local-editor conflicts and offer **Keep my version** / **Use cloud version**.

## Recurring phrases

In **Preferences > Recurring phrases**, save up to 50 phrases of 180 characters, one per line, or disable speech bubbles. Phrases are chosen locally without a language model. User phrases can be in Italian or any preferred language.

The bubble lasts six seconds and avoids immediately repeating the last phrase when alternatives exist. Speech uses its own configurable pause: 5–10 minutes by default. It continues during brooding without changing the pose or hiding eggs. Speech pauses for ready notifications, a hidden mascot, or disabled/reduced animation.

The bubble uses a separate transparent window, does not take focus, and lets clicks pass through. It does not resize or move the mascot. An open menu takes priority.

**Preview phrase** immediately shows a configured phrase, including when animations are disabled. On Windows, a dedicated Rust adapter uses `SW_SHOWNOACTIVATE` to show the bubble without interrupting typing.

## Verification

Widget tests cover the save queue shared by autosave/close/quit and phrase selection. Rust tests cover conflicts, visibility, reminders, migrations and protected files. `scripts/smoke-server.py` exercises actual APIs in an isolated profile, including restart and backups. Report native visual checks separately.

Windows verification on November 7, 2026: 20 Rust tests, 11 frontend tests, Svelte, Clippy and HTTP checks passed. Native checks covered typing/autosave, menu creation, confirmed deletion, position, pinning, hide-on-close, coordinated exit saving and startup restoration. Note and bubble captures were inspected; configured text, no focus, click-through and automatic dismissal were verified. Later speech/brooding checks are recorded in [Mascots](mascot.md). Desktop Linux was not verified.
