# MonitorAwake

## Browser policy

- Never open, launch, or control the Codex in-app browser, Browser Use, Playwright browser windows, Chrome, Edge, or any other browser unless the user explicitly asks for browser-based verification in the current message.
- Start frontend and web development servers headlessly. Do not use launch profiles or commands that automatically open a browser; provide the local URL.
- If browser-based visual verification is needed, ask first and wait for explicit approval.

## Product direction

MonitorAwake is a small desktop assistant for Windows and Linux. Its main features are keeping the system awake, named timers, notifications, and simple configurable tasks. A compact mascot and command field are the everyday interface; an embedded Rust HTTP server serves the Svelte configuration page on loopback. Open configuration in the user's browser only in response to their action, never automatically at startup.

The chosen direction is a Rust core with Tauri and a Svelte/TypeScript interface. Keep the core independent of the GUI. Start with deterministic commands in Italian and local operation; a language model or cloud service is not required. The first implementation milestone is a transparent, draggable, clickable mascot prototype, to assess desktop integration before expanding the application.

The user wants to learn Rust. Explain significant Rust choices briefly in Italian, using the code being changed as the example. Prefer small working increments over speculative frameworks.

## Project skills

Read the relevant skill when its task applies. These repository files are the maintained source; personal installed copies must be kept in sync when the skills are updated.

- [monitorawake-rust-core](skills/monitorawake-rust-core/SKILL.md): command parsing, timers, state, persistence, and task execution.
- [monitorawake-desktop-ui](skills/monitorawake-desktop-ui/SKILL.md): Tauri windows, mascot rendering, controls, settings, and frontend integration.
- [monitorawake-platform-adapters](skills/monitorawake-platform-adapters/SKILL.md): Windows/Linux awake requests, notifications, and platform capability differences.

For changes spanning boundaries, read the relevant skills without assuming a fixed directory structure that has not yet been implemented. Report what was actually checked on each operating system; compilation alone does not verify desktop behavior.
