---
name: monitorawake-rust-core
description: Develop the MonitorAwake Rust core for Italian commands, named timers, persistence, and simple configurable tasks. Use for core behavior and its tests, rather than mascot styling or native OS adapter implementation.
---

# MonitorAwake Rust core

Apply this skill to MonitorAwake work, not unrelated Rust projects. Inspect the repository instructions and existing code before choosing modules or dependencies. The product is a local desktop assistant for Windows and Linux; the user is learning Rust. Explain significant ownership, enum, trait, or concurrency decisions in Italian using the implementation at hand, without turning every change into a tutorial.

## Boundaries

- Keep command parsing, timer rules, and task state usable without Tauri or a WebView. The UI submits typed commands and renders state/events; it does not own scheduling.
- Model concrete states with enums and stable identifiers. Introduce traits for actual external boundaries such as clocks and OS services, rather than for every type.
- Keep system-awake requests distinct from timers. A timer keeps the system awake only when that option is selected.
- Use the repository's established async approach. For an initial implementation, add an async runtime only when the selected operations need it.

## Commands

Start with a small deterministic Italian vocabulary. Representative requests include `avvisami tra 30 minuti`, `timer pasta 12 minuti`, `quanto manca?`, `aggiungi 5 minuti al timer pasta`, and `annulla il timer pasta`.

- Parse into typed intents before executing effects. Reject invalid, zero, negative, excessive, or overflowing durations with a useful response.
- When multiple timers make a command ambiguous, return a clarification with candidate timers. Never select a destructive target arbitrarily.
- Unknown input returns supported examples; it is never passed through to a shell.
- Keep conversational text separate from domain results so UI wording can evolve.

## Timer lifecycle

- Store stable IDs, names, absolute UTC deadlines, and lifecycle state. Display times in the user's local timezone.
- Do not derive elapsed time from the number of UI ticks. Reconcile deadlines on startup and resume; a timer that expired during suspension is due on resume, not evidence that the app woke the computer.
- For running timers, explicitly choose how clock corrections affect deadlines. Use monotonic time for elapsed-time measurement where appropriate and account for suspend behavior on the target OS; a monotonic clock alone is not a persistence format.
- Define cancellation, extension, expiry, and acknowledgement transitions so repeated scheduler checks do not repeatedly alert. Document the crash window between persisting delivery state and sending an external notification; do not claim exactly-once delivery without support.
- Persist changes with atomic replacement where supported and a schema version. Report corrupt or unsupported configuration instead of silently overwriting it.

## Simple tasks

Implement only the task types requested for the current increment. Each executable task has an explicit program, arguments, working directory when needed, timeout, and an overlap policy. Prefer process APIs with separate arguments; shell syntax is an explicit task type, not the interpretation of arbitrary user text.

Associate each running task with its own awake lease when requested. Release it on success, failure, cancellation, and startup reconciliation. Define cancellation of child processes per OS instead of assuming terminating the parent terminates the whole tree. Bound captured output and retain a useful exit result.

## Validation

Use a controllable clock and fake external services for meaningful behavior tests, avoiding real waits. Cover the transitions touched by the change: ambiguity, extension/cancellation near expiry, restart with overdue timers, repeated expiry checks, and overlapping awake owners as applicable.

Run formatting checks, relevant Rust tests, and lint checks supported by the actual workspace. Report limitations rather than inventing a successful Linux run from Windows. Do not launch a browser as part of core tests; follow the repository browser policy.
