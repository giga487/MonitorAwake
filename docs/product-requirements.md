# Product requirements

Direction agreed on November 10, 2026. This document distinguishes implemented functionality from work still requiring external setup or verification.

## Main features

| Feature | Requirement | Current state |
| --- | --- | --- |
| Awake mode | Keep the computer active; distinguish system and display capabilities. | Implemented, with different Windows/Linux limitations. |
| Timers and alarms | Duration timers and alarms at a specified time. | Timers and one-time local date/time alarms implemented. |
| Shared notes | Choose local or Firebase storage and share notes. | Storage selection, local cache, Account ID sharing and conflict resolution implemented. Production access requires published rules and verification. |

## Agreed constraints

- Local note storage remains available and is the default.
- Firebase initially applies only to notes. Awake mode, timers and alarms are not synchronized.
- Persistent data, backend configuration and cloud documents need explicit schema versions and migrations.
- App UI stays in English; personal phrases can use Italian or any preferred language.
- Firebase project configuration belongs to the local installation and must not be committed or bundled.
- Advance Firebase setup in small, verifiable steps.

Sharing supports the same account on multiple devices and different people using the same project. Only the owner changes membership; collaborators can edit or delete the shared note.

See [Local alarms](alarms.md), [Firebase setup](firebase-setup.md) and [Cloud notes](cloud-notes.md).
