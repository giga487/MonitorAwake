#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = monitorawake::run() {
        eprintln!("Could not start MonitorAwake: {error}");
        #[cfg(windows)]
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
            let text: Vec<u16> = error.encode_utf16().chain(Some(0)).collect();
            let title: Vec<u16> = "MonitorAwake".encode_utf16().chain(Some(0)).collect();
            // SAFETY: both strings are NUL-terminated and live until the synchronous call returns.
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    text.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
        std::process::exit(1);
    }
}
