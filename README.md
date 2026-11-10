# MonitorAwake

[Download the latest release](https://github.com/giga487/MonitorAwake/releases/latest) · [Release and installer workflow](docs/releases.md)

A small desktop assistant built with Rust, Tauri and Svelte. Keep your computer awake, create named timers and alarms, and manage desktop sticky notes beside an animated mascot. Local features work without an account; Firebase note synchronization and sharing are optional.

Application version: **0.2.0**. Local data schema: **8**. Firebase configuration and cloud note documents each use schema **1**. Application and data versions are independent.

## Install and get started

On Windows, run the x64 setup from GitHub Releases or build it below. Close a running copy before replacing it. Local builds are unsigned. The installer can install WebView2 if missing; downloading that runtime may require a connection.

The mascot starts above the taskbar. Drag it with the left mouse button; right-click to open the menu in a separate window without moving the mascot. Press Esc, click outside, or switch applications to dismiss the menu. Keyboard: focus the mascot with Tab, then press Enter, Space or Shift+F10. Closing the menu hides it; **Quit** stops the app and its awake requests.

- **Notification:** enter a message, select **Next**, choose an integer amount from 1 to 100 and seconds, minutes, hours or days, then select **Start notification**. Maximum duration: 100 days.
- **Alarm:** choose a name and future local date/time. Find alarms and timers in **My timers**. See [Local alarms](docs/alarms.md).
- **New note / My notes:** create and reopen sticky notes with autosave, four colors, pinning, resizing and optional reminders. Closing hides a note; deletion requires confirmation. See [Desktop notes](docs/notes.md).
- **Preferences:** configure awake behavior, animations, your mascot and recurring phrases. Personal phrases can be in any language. Default speech pauses are 5–10 minutes, including while brooding.
- **Firebase account:** configure your own project and sign in. **My notes > Use Firebase notes** explicitly enables synchronization. Follow the [Firebase setup guide](docs/firebase-setup.md).

The interface and notifications are in English. Text commands include `timer tea 5 minutes`, `remind me in 30 minutes`, `how much time is left?`, `add 5 minutes to timer tea` and `cancel timer tea`. Italian commands remain supported; saved names and personal phrases are not translated. The command field never executes arbitrary shell commands.

Chicken, Frost gryphon, Ember wyvern and Woodland elf are selectable mascots. Each pending notification has an egg: green above 50% remaining, yellow down to 20%, then red. Ready eggs remain cracked until acknowledged. Animations can be disabled and respect reduced motion. See [Mascots and assets](docs/mascot.md).

## Configure Firebase notes

Firebase is optional. Awake, timers, alarms and note reminders remain local.

1. Create a Firebase project and register a **Web** app. Hosting and Analytics are not required.
2. Enable **Authentication > Sign-in method > Google**. Add `127.0.0.1` to authorized domains if missing.
3. Create a **Cloud Firestore Standard** database with ID **`(default)`**, in Production mode. Choose a European location before creation if required.
4. Publish [firestore.rules](firestore.rules) in the database's **Rules** tab.
5. Open **Firebase account > Local Firebase configuration** and enter your SDK values using [firebase.example.json](firebase.example.json) as the field-name template.
6. Sign in, then select **My notes > Use Firebase notes**. This also uploads existing local notes to that account.

Project configuration belongs to the individual installation. It is stored in `firebase.local.json`, excluded from Git and the installer. No default Firebase project is bundled. The [full setup guide](docs/firebase-setup.md) explains field mapping, file locations, Google sign-in and troubleshooting; [Cloud notes](docs/cloud-notes.md) covers sharing, offline changes and conflicts.

## Windows development

Requirements: Rust stable with Cargo, rustfmt and Clippy; Node.js 22.12+ or a later compatible LTS; Microsoft C++ Build Tools x64; Windows SDK; and WebView2 for the desktop UI.

```powershell
. .\scripts\windows-env.ps1 -BootstrapSdk
npm.cmd ci
npm.cmd run desktop:dev
```

The script configures the current PowerShell session without changing the permanent PATH. If needed, `-BootstrapSdk` downloads official Microsoft SDK packages from NuGet into `%LOCALAPPDATA%\MonitorAwake\build-tools`; an installed MSVC x64 compiler is still required. Later runs reuse the cache.

`desktop:dev` builds the embedded assets, starts Vite headlessly and launches the native app. Mascot edits support hot reload; rebuild assets and restart to update the embedded HTTP page. No browser opens automatically.

To generate the executable and Windows installer:

```powershell
.\scripts\build-windows.ps1
```

Add `-BootstrapSdk` if required. The script installs frontend dependencies when absent, stops on errors, and does not launch the result. Outputs:

- `target/release/monitorawake.exe`
- `target/release/bundle/nsis/MonitorAwake_0.2.0_x64-setup.exe`

## Linux development

Install Rust, Node.js and your distribution's Tauri prerequisites. On Ubuntu/Debian:

```sh
sudo apt install build-essential libwebkit2gtk-4.1-dev libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
npm ci
npm run desktop:dev
npm run desktop:build -- --bundles deb
```

The mascot requires a graphical session. Wayland placement, dragging and always-on-top behavior depend on the compositor. The initial Linux awake adapter uses logind and requires system-bus access and permission to inhibit `sleep`; display inhibition is not implemented. It does not bypass system policy. Depending on logind policy, inhibition may also affect explicit sleep requests. Actual desktop Linux behavior was not verified in the latest Windows increment.

## Local configuration server

Select the mascot menu's settings gear to open configuration in your browser. **URL** lets you copy the private link. Closing the page does not stop the app; quitting the app stops its server.

The Rust server listens only on `127.0.0.1` and chooses an available port. Its private link contains a random per-run key in `#token=...`; the page moves it into session storage and removes it from the address bar. APIs require the key, validate Host and Origin, and do not enable wildcard CORS. Do not share the link. Reopen configuration from the mascot after restarting.

To run the core without Tauri:

```sh
npm run server
```

This prints the private URL without opening a browser. Notifications go to the terminal in this mode. Ctrl+C stops the server and releases awake requests. Only one instance is allowed per data directory. The standalone HTTP page does not run the desktop Firebase synchronization worker.

## Data and behavior

- Windows data: `%LOCALAPPDATA%\MonitorAwake\state.json`.
- Linux data: `$XDG_DATA_HOME/MonitorAwake`, normally `~/.local/share/MonitorAwake`.
- `MONITORAWAKE_DATA_DIR` selects an isolated test profile.
- Saves use temporary files and atomic replacement. Corrupt files and unsupported future formats are preserved. Migrations create exact backups first; see [Persistence](docs/persistence.md).
- Timers use absolute UTC deadlines. Clock corrections affect remaining time. The app cannot wake a suspended computer; overdue timers are recovered on startup/resume.
- Manual awake mode starts disabled each session. Each timer with awake enabled owns an independent request until expiry or cancellation.
- Expiry is persisted before external notification delivery. A crash between these steps may miss that notification, while the in-app alert remains after restart. Exactly-once external delivery is not guaranteed.
- Windows Tauri notifications require an installed app for their intended behavior; the mascot and configuration page remain useful during development.

## Checks

Build frontend assets before Rust checks because they are embedded into the executable.

```sh
npm run check
npm run test:widget
npm run test:firebase
npm run test:firestore
npm run build
cargo fmt --all -- --check
cargo test --workspace --no-default-features --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Firebase integration cases require local emulators; otherwise they are skipped. See the [emulator instructions](docs/firebase-setup.md#local-emulator-tests). CI runs a separate emulator job with a fictitious project.

For an actual HTTP test with an isolated profile:

```sh
cargo build --no-default-features --bin monitorawake-server
python scripts/smoke-server.py target/debug/monitorawake-server.exe
# Linux: target/debug/monitorawake-server
```

The script covers authentication, Host/Origin checks, assets, expiry, restart, token rotation, persistence and single-instance behavior. It stops its process on failure and retains the test profile for diagnosis.

Windows verification on November 10, 2026: 42 Rust tests, 16 widget tests, 5 Firebase Auth/config tests and 2 Firestore tests passed, along with Svelte, Clippy and HTTP checks. Native Windows emulator tests verified upload, queued edits across restart, resumed synchronization, remote changes and deletion. Real Google sign-in and the production database remain to be verified by the user. Desktop Linux was not tested for this increment. Compilation alone does not verify desktop behavior. Follow the browser policy in `AGENTS.md`.

## Code map

| File | Responsibility |
| --- | --- |
| `src-tauri/src/core.rs` | Domain types, timer transitions and deterministic commands |
| `src-tauri/src/service.rs` | Shared state and transactional persistence |
| `src-tauri/src/note_sync.rs` | Cloud note state, queued changes and conflicts |
| `src-tauri/src/firebase_config.rs` | Local runtime Firebase configuration |
| `src-tauri/src/http.rs` | Axum server, authenticated APIs and embedded assets |
| `src-tauri/src/platform.rs` | Windows/Linux adapters and awake request lifetimes |
| `src-tauri/src/lib.rs` | Desktop lifecycle, scheduler and Tauri commands |
| `src/App.svelte` | Mascot and configuration interface |
| `src/lib/api.ts` | Tauri/HTTP transport to the same Rust service |
| `src/lib/NotesCloudSync.svelte` | Desktop Firebase synchronization worker |

Maintained project skills live in `skills/`. Synchronize installed personal copies in `~/.codex/skills/` when changing those skills.
