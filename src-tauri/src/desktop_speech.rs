use tauri::Manager;

#[tauri::command]
pub async fn set_speech_visible(app: tauri::AppHandle, visible: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("speech")
        .ok_or("Speech window unavailable")?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GetWindowLongW, IsWindowVisible, SW_HIDE, SW_SHOWNOACTIVATE,
            SetWindowLongW, ShowWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };
        let handle = window.hwnd().map_err(|e| e.to_string())?.0 as _;
        // SAFETY: the handle belongs to our fixed speech window. No pointers or
        // user-provided handles are accepted. ShowWindow's result is the previous
        // visibility, so verify the current visibility separately.
        unsafe {
            let style = GetWindowLongW(handle, GWL_EXSTYLE);
            SetWindowLongW(
                handle,
                GWL_EXSTYLE,
                style | WS_EX_NOACTIVATE as i32 | WS_EX_TOOLWINDOW as i32,
            );
            ShowWindow(handle, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
            if (IsWindowVisible(handle) != 0) != visible {
                return Err("Could not change speech bubble visibility.".into());
            }
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        if visible {
            window.show()
        } else {
            window.hide()
        }
        .map_err(|e| e.to_string())
    }
}
