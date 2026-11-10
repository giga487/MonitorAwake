# Persistent data and migrations

All persistent data must belong to a versioned schema. Data format versions are independent of application release versions.

## Current state

`state.json` uses global `schema_version: 8`, declared in `core::SCHEMA_VERSION`. It covers timers, notes, cloud synchronization metadata and preferences, including mascots and recurring phrases. The configuration page exposes app and schema versions in local details.

`persistence::load` reads the version before deserializing. It supports the migration chain v1 → v2 → v3 → v4 → v5 → v6 → v7 → v8, preserving existing data and explicitly initializing new fields. It validates the migrated model, writes an exact backup named `state.json.backup-v<original-version>-<uuid>.json` without replacing earlier backups, then atomically replaces the main file using a synchronized temporary file.

Reopening a current-format file does not migrate again. Corrupt files and unsupported future versions are preserved; no downgrade is attempted. The special v1 elf prototype can migrate directly to v6 before following later steps.

Serde defaults handle explicitly supported omissions, such as an older profile's missing mascot or phrase preferences. A default alone does not make arbitrary future changes backward compatible: older clients could erase fields on save.

A note's `content_revision` protects against stale text editors. It is separate from the schema version and cloud document revision. Color, geometry and visibility do not increment the text revision.

## Rules for future changes

- Keep one backend constant for the current local format and document each distributed version.
- Version each independent persistent store. A global version covers entities kept in the same file; split stores need their own versions.
- Document references and compatibility between stores. Avoid partial writes that leave dangling note/reminder links.
- Detect old, future and corrupt formats before current-model deserialization.
- Migrate through explicit steps rather than scattering conversions through the frontend.
- Preserve an exact, distinct backup before migration. If creating the backup fails, leave the original untouched.
- Convert and validate in memory, save with atomic replacement, then commit the service's in-memory state only after success.
- Preserve unreadable files and show useful errors instead of replacing them with an empty profile.
- Reject writing future versions and explain that a newer app is required.
- Consider compatibility on both reads and rewrites; ignoring unknown fields can lose data.
- Show schema versions diagnostically without requiring users to select routine migrations.

## Notes and reminders

Notes share the timer store. Adding/removing a linked reminder is one validated model change saved before it becomes visible. Missing reminders and reminders shared by multiple notes are rejected. History cleanup removes references to timers that no longer exist.

Each note stores UUID, title, body, color, visibility, pinning, geometry, content revision and an optional reminder ID. Coordinates are global physical pixels; width/height are logical pixels. Restoration clamps positions to an available monitor's work area; Wayland may let the compositor decide placement.

Hiding preserves content and reminders. Limits: 100 local notes, 80-character titles, 10,000-character bodies, 50 phrases of 180 characters, and an 8 MiB state file. The app identity stays neutral: Chicken is a mascot preference, not a data format identifier.

## Schema history

| Schema | Change and migration behavior |
| --- | --- |
| 1 | Original timer/preferences prototype. |
| 2 | Desktop notes and recurring phrases. Migrates old state, preserving timers/preferences and initializing missing fields. |
| 3 | Adds `frost_gryphon` to the mascot enum. v2 → v3 preserves selection, notes, timers and personal phrases. |
| 4 | Adds `ember_wyvern`. v3 → v4 preserves existing data. Expanding an enum requires a version change so old clients cannot rewrite unsupported values. |
| 5 | Adds `speech_min_minutes` and `speech_max_minutes`, integers from 1 to 1440 with minimum ≤ maximum. Defaults are 5 and 10. v4 → v5 preserves data with exact backup and atomic replacement. |
| 6 | Adds `elf`. Migration preserves the selected mascot. The v1 elf prototype has a dedicated path preserving its timers/preferences and exact original backup. |
| 7 | Adds `timers[].kind`: `timer` or `alarm`. Existing records default to `timer`; v6 → v7 preserves deadlines and note links. |
| 8 | Adds `settings.notes_storage` (default `local`) and `note_sync`: bound account/project, cloud base revisions, desired members, conflicts and revoked access. v7 → v8 retains notes without uploading and initializes an empty cache. |

Pending cloud operations are derived from local notes versus their saved cloud bases, so they survive restart. Firebase configuration and cloud note documents each use independent schema 1. See [Cloud notes](cloud-notes.md).

## Validation

Keep fixtures for released formats. Cover migration, reopening without duplicate work, text/ID preservation, optional settings, reminder references, exact backups, corrupt/future files and failed writes. Verify that errors leave the original unchanged. Rust and HTTP tests cover these behaviors; each future format needs its own explicit migration and previous-format fixture.
