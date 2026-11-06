---
name: monitorawake-platform-adapters
description: Implement and verify MonitorAwake integration with Windows and Linux for awake requests, desktop notifications, and capability detection. Use when native behavior or OS differences change, rather than for pure timer logic or mascot styling.
---

# MonitorAwake platform adapters

Apply this skill to MonitorAwake's OS boundary. Read repository instructions and inspect the Rust core's existing contract. Keep Windows and Linux implementations behind a small shared interface; avoid leaking native handles or desktop-specific policy into timer and UI code.

## Shared semantics

- Distinguish preventing automatic system sleep from keeping the display on. Do not equate either with preventing session lock, waking a suspended machine, or overriding explicit user suspension.
- Represent active requests as owned leases with reasons. Manual mode and concurrent tasks can each own a lease; releasing one owner must not release another owner's request. Aggregate the required mode across active owners.
- Expose available capabilities and actionable failures. Unsupported, denied, and temporarily unavailable are different outcomes; do not silently report an unavailable feature as active.
- Ensure handles and requests are released on normal completion and errors. Check what the OS actually releases on process exit before relying on it for crash cleanup.
- Use supported inhibition APIs rather than simulating keyboard or mouse activity. Normal use should not require elevation or permanent changes to machine power settings.

## Windows

Evaluate the documented execution-state or power-request APIs for the chosen lifecycle. `SetThreadExecutionState` requirements belong to the calling thread: if using it, create/update/clear the request on a dedicated long-lived thread, not arbitrary async worker threads. Account for continuous state and separately aggregate system/display requirements.

If using handle-based power requests, pair each acquired request with the documented clear/close behavior. Check return values and preserve useful OS error information. Do not claim these requests prevent screen locking or explicit sleep actions.

Primary references, to consult when implementing or changing the API usage:

- [SetThreadExecutionState](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate)
- [PowerCreateRequest](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-powercreaterequest)
- [PowerSetRequest](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-powersetrequest)

## Linux

Detect the relevant session services rather than assuming every distribution uses the same stack. Evaluate XDG desktop portal inhibition for session integration and logind inhibition where available and appropriate. Their semantics are not interchangeable: check which sleep, idle, and display behavior the selected backend actually supports.

Retain the portal request handle or logind file descriptor for the lifetime of the request, release it correctly, and handle service loss/reconnection. Portal operations can complete asynchronously; inspect the actual response before reporting success. Do not silently broaden inhibition to logout, lid switches, or shutdown.

Desktop notifications and visual window behavior also depend on the session. Record distribution, desktop, and X11/Wayland when testing relevant functionality. Avoid promising identical GNOME/KDE or Wayland/X11 behavior without verification.

Primary references:

- [XDG Inhibit portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Inhibit.html)
- [systemd inhibitor locks](https://github.com/systemd/systemd/blob/main/docs/INHIBITOR_LOCKS.md)

## Validation

Use fake adapters to test request aggregation, failure propagation, and release ordering without affecting the host. Compile supported platform configurations where the toolchain is available; keep host-independent code buildable when the other platform's native dependencies are absent.

For real host verification, use a bounded temporary request with guaranteed cleanup and inspect the OS-visible request when possible. Do not deliberately suspend the user's machine or modify global power policy as a routine test. Report separately compilation, automated behavior checks, and real desktop verification. Follow the repository's no-browser-without-explicit-request policy.

Explain the native lifecycle and relevant Rust ownership decisions briefly in Italian so the user can learn from the implementation.
