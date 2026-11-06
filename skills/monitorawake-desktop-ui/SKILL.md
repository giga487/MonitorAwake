---
name: monitorawake-desktop-ui
description: Build the MonitorAwake Tauri and Svelte desktop interface, including its floating mascot, command field, timer display, and settings. Use for desktop UI work and Rust-to-frontend integration, not for unrelated websites.
---

# MonitorAwake desktop UI

Apply this skill to the MonitorAwake desktop assistant. Read repository instructions and inspect existing frontend and Tauri configuration first. The intended stack is Tauri with Rust and Svelte/TypeScript; keep existing compatible conventions rather than regenerating the application.

## Interaction model

- Use a compact optional mascot above a rounded control bar. Provide awake/pause, remaining time, and settings access; reveal the text command field on deliberate interaction.
- Keep routine commands and timer feedback near the mascot. Put appearance, awake controls, task configuration, and activity history in the browser configuration page served by the embedded Rust HTTP server.
- Display explicit state from the Rust core. Animation and countdown rendering must not control timer expiry or hold the only copy of timer state.
- Show system-awake and display-awake as separate choices, along with unsupported or failed states. Do not indicate success before the backend confirms it.
- Support labelled controls, keyboard access, visible focus, reduced motion, and optional sound. Do not communicate errors or timer expiry only through mascot animation or color.

## Window prototype first

For the first UI increment, prove a small transparent, undecorated window with a simple local placeholder mascot, draggable region, clickable control, and an explicit way to exit. A polished asset or animation engine is not a prerequisite.

- Distinguish draggable areas from buttons and inputs so dragging does not swallow interaction.
- Transparency does not imply click-through. Keep window bounds close to visible content; implement selective hit testing only when required and supported.
- Make mascot visibility, closing settings, hiding the widget, and quitting the application separate actions. If closing a window leaves work running, provide a reliable way to reopen and quit, without assuming a tray exists on every Linux desktop.
- Avoid stealing focus on countdown updates or routine notifications.
- Treat always-on-top, positioning, transparency, tray availability, scaling, and multi-monitor behavior as platform capabilities to verify. Wayland can restrict window placement and stacking; record the actual desktop/session tested and provide a usable fallback.

## Frontend/backend boundary

Embed the configuration page's built assets in the application and serve them through a Rust HTTP server bound only to loopback. Use an OS-assigned available port and provide the current URL through the mascot. Open the browser only on an explicit user action. Authenticate API access with a per-run secret, validate Host and Origin, and do not enable wildcard CORS. The HTTP API and Tauri commands must call the same core service; neither interface owns a separate timer list. Keep configuration functional without internet or Node.js at runtime.

Scope Tauri capabilities to the operations each window needs. Validate incoming payloads in Rust, unsubscribe event listeners when views are disposed, and avoid giving the frontend unrestricted process execution. For details of version-sensitive Tauri APIs, consult current official documentation rather than copying older configuration.

## Validation and browser policy

Never open, launch, or control a browser, including the Codex in-app browser, Browser Use, Playwright windows, Chrome, or Edge, unless the user explicitly requests browser-based verification in the current message. If needed, ask and wait for that approval. Start development servers headlessly and provide only the local URL; disable automatic browser opening.

Use frontend type checks, builds, focused interaction/state tests, and Rust checks as applicable. Native desktop verification is distinct from browser verification: use available native tooling within the user's authorized scope and report what was actually observed. If visual verification was not performed, say so; a successful build does not prove transparent-window behavior. Do not create screenshot-test infrastructure solely for a small reversible style change.

Explain significant Rust/IPC choices briefly in Italian, relating them to the feature being implemented.
