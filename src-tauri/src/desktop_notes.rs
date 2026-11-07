use crate::{AppRuntime, core::Action, notes::Note};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    time::Duration,
};
use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

#[derive(Default)]
pub struct QuitCoordinator(pub Mutex<QuitRequest>);
#[derive(Default)]
pub struct QuitRequest {
    token: String,
    pending: HashSet<String>,
    failed: bool,
}

#[tauri::command]
pub fn note_flush_done(
    window: tauri::WebviewWindow,
    token: String,
    saved: bool,
    requests: tauri::State<'_, QuitCoordinator>,
) {
    if let Ok(mut request) = requests.0.lock()
        && request.token == token
        && request.pending.remove(window.label())
    {
        request.failed |= !saved;
    }
}

pub async fn graceful_quit(app: tauri::AppHandle) -> Result<(), String> {
    let windows: Vec<_> = app
        .webview_windows()
        .into_iter()
        .filter(|(label, _)| label.starts_with("note-"))
        .collect();
    let token = Uuid::new_v4().to_string();
    {
        let state = app.state::<QuitCoordinator>();
        let mut request = state
            .0
            .lock()
            .map_err(|_| "Cannot prepare notes for exit")?;
        if !request.pending.is_empty() {
            return Err("Notes are already being saved. Please wait.".into());
        }
        *request = QuitRequest {
            token: token.clone(),
            pending: windows.iter().map(|(label, _)| label.clone()).collect(),
            failed: false,
        };
    }
    for (label, window) in windows {
        if let Err(error) = window.emit_to(label.as_str(), "save-before-quit", &token) {
            if let Ok(mut request) = app.state::<QuitCoordinator>().0.lock() {
                request.pending.clear();
            }
            return Err(error.to_string());
        }
    }
    for _ in 0..100 {
        let done = {
            let state = app.state::<QuitCoordinator>();
            let request = state.0.lock().map_err(|_| "Cannot check note saves")?;
            if request.failed {
                break;
            }
            request.pending.is_empty()
        };
        if done {
            app.exit(0);
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    if let Ok(mut request) = app.state::<QuitCoordinator>().0.lock() {
        request.pending.clear();
    }
    Err(
        "Some notes could not be saved. The app has stayed open; check the notes and try again."
            .into(),
    )
}

pub fn start(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_revision = None;
        let mut cache: HashMap<Uuid, Note> = HashMap::new();
        loop {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let Some(runtime) = app.try_state::<AppRuntime>() else {
                break;
            };
            if runtime.stopped.load(std::sync::atomic::Ordering::Acquire) {
                break;
            }
            let update = runtime.service.lock().ok().and_then(|service| {
                (last_revision != Some(service.notes_revision))
                    .then(|| (service.notes_revision, service.desktop_notes()))
            });
            let Some((revision, notes)) = update else {
                continue;
            };
            let result = sync(&app, &notes, &mut cache, last_revision.is_some());
            if let Ok(mut service) = runtime.service.lock() {
                service.desktop_error = result.as_ref().err().cloned();
            }
            if result.is_ok() {
                last_revision = Some(revision);
            }
        }
    });
}

fn sync(
    app: &tauri::AppHandle,
    notes: &[Note],
    cache: &mut HashMap<Uuid, Note>,
    user_change: bool,
) -> Result<(), String> {
    let removed: Vec<_> = cache
        .keys()
        .filter(|id| !notes.iter().any(|note| note.id == **id))
        .copied()
        .collect();
    for id in removed {
        if let Some(window) = app.get_webview_window(&format!("note-{id}")) {
            window.destroy().map_err(|e| e.to_string())?;
        }
        cache.remove(&id);
    }
    for note in notes {
        let old = cache.get(&note.id);
        if old == Some(note) {
            continue;
        }
        let label = format!("note-{}", note.id);
        let mut window = app.get_webview_window(&label);
        let creating = window.is_none() && note.visible;
        if creating {
            window = Some(
                WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
                    .title("MonitorAwake — Note")
                    .inner_size(note.bounds.width, note.bounds.height)
                    .min_inner_size(240.0, 200.0)
                    .max_inner_size(1600.0, 1600.0)
                    .decorations(false)
                    .resizable(true)
                    .transparent(false)
                    .always_on_top(note.pinned)
                    .visible(false)
                    .focused(false)
                    .build()
                    .map_err(|e| format!("Cannot open note: {e}"))?,
            );
        }
        if let Some(window) = window {
            // Bounds updates from the editor are observations of an already moved native
            // window. Reapplying them here would snap it back during a continuous drag.
            if creating || old.is_none() {
                let monitors = window.available_monitors().unwrap_or_default();
                let monitor = monitors
                    .iter()
                    .find(|monitor| {
                        let area = monitor.work_area();
                        note.bounds.x >= area.position.x
                            && note.bounds.y >= area.position.y
                            && i64::from(note.bounds.x)
                                < i64::from(area.position.x) + i64::from(area.size.width)
                            && i64::from(note.bounds.y)
                                < i64::from(area.position.y) + i64::from(area.size.height)
                    })
                    .or_else(|| monitors.first());
                let mut x = note.bounds.x;
                let mut y = note.bounds.y;
                if let Some(monitor) = monitor {
                    let area = monitor.work_area();
                    let scale = monitor.scale_factor();
                    x = x.clamp(
                        area.position.x,
                        (area.position.x + area.size.width as i32
                            - (note.bounds.width * scale) as i32)
                            .max(area.position.x),
                    );
                    y = y.clamp(
                        area.position.y,
                        (area.position.y + area.size.height as i32
                            - (note.bounds.height * scale) as i32)
                            .max(area.position.y),
                    );
                }
                // Wayland can reject positioning; the compositor still provides a usable window.
                let _ = window.set_position(PhysicalPosition::new(x, y));
                window
                    .set_size(LogicalSize::new(note.bounds.width, note.bounds.height))
                    .map_err(|e| e.to_string())?;
            }
            if creating || old.is_none_or(|previous| previous.pinned != note.pinned) {
                window
                    .set_always_on_top(note.pinned)
                    .map_err(|e| e.to_string())?;
            }
            if note.visible && (creating || old.is_none_or(|previous| !previous.visible)) {
                window.show().map_err(|e| e.to_string())?;
                if user_change {
                    let _ = window.set_focus();
                }
            }
            if !note.visible {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
        cache.insert(note.id, note.clone());
    }
    Ok(())
}

pub fn close_note(app: &tauri::AppHandle, label: &str) {
    if let Some(id) = label
        .strip_prefix("note-")
        .and_then(|id| Uuid::parse_str(id).ok())
        && let Some(runtime) = app.try_state::<AppRuntime>()
        && let Ok(mut service) = runtime.service.lock()
        && let Err(error) = service.dispatch(Action::SetNoteVisible { id, visible: false })
    {
        service.desktop_error = Some(error);
    }
}
