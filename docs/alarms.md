# Local alarms

The November 10, 2026 increment adds one-time alarms at a specific date/time alongside duration timers. Choose **Alarm** in the mascot menu or **Set an alarm for a date and time** in configuration. Enter a name, local date and time, then select **Set alarm**. **My timers** shows the local date/time, countdown, cancellation and a five-minute extension.

Dates are interpreted in the computer's local timezone at creation and converted to UTC. They must be in the future and within 100 days. Nonexistent times during the spring daylight-saving transition are rejected; a repeated time in autumn selects the first occurrence, matching JavaScript local-time conversion. Changing timezone later changes the displayed local time, not the saved deadline.

Rust validates the deadline again. `TimerKind::Alarm` distinguishes alarms from `TimerKind::Timer` while sharing scheduling and acknowledgement rules. Alarms use the same eggs and optional awake requests; unsupported awake capability is rejected. Name, UUID, kind and deadline are persistent. The service publishes the resulting state only after a successful save.

The app must be running to show an alert. An alarm that expires while closed or suspended is recovered on next startup/resume. This does not wake the computer in hardware. Recurrence and dedicated alarm sounds are outside this increment.

Schema 7 introduced `timers[].kind`, defaulting to `timer` for older data. v6 → v7 preserves timers, notes, mascots and phrases with an exact backup before atomic replacement. Current state uses schema 8; Firebase does not synchronize alarms.

Windows verification for the alarm increment: 30 Rust tests, 16 frontend tests, Svelte, Clippy and HTTP checks passed. A native isolated profile migrated from v6 verified menu creation, saved date/kind, restart, recovery of an overdue alarm and acknowledgement. Native form/list captures were inspected and an NSIS installer generated. No browser was opened; desktop Linux was not verified.
