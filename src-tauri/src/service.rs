use crate::{
    core::{Action, AwakeMode, Model, ParsedCommand, Settings, Timer},
    platform::Capabilities,
};
use serde::Serialize;
#[cfg(test)]
use std::fs;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub type SharedService = Arc<Mutex<Service>>;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub schema_version: u32,
    pub app_version: &'static str,
    pub notes: Vec<crate::notes::Note>,
    pub desktop_error: Option<String>,
    pub timers: Vec<Timer>,
    pub settings: Settings,
    pub manual_mode: AwakeMode,
    pub requested_mode: AwakeMode,
    pub applied_mode: AwakeMode,
    pub capabilities: Capabilities,
    pub backend_error: Option<String>,
    pub notification_error: Option<String>,
    pub now_ms: u64,
}

#[derive(Serialize)]
pub struct ActionResult {
    pub message: String,
    pub state: Snapshot,
}

pub struct Service {
    model: Model,
    path: PathBuf,
    persistence_error: Option<String>,
    manual_mode: AwakeMode,
    pub applied_mode: AwakeMode,
    pub capabilities: Capabilities,
    pub backend_error: Option<String>,
    pub notification_error: Option<String>,
    pub desktop_error: Option<String>,
    pub notes_revision: u64,
}

impl Service {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let model = crate::persistence::load(&path)?;
        Ok(Self {
            model,
            path,
            persistence_error: None,
            manual_mode: AwakeMode::Off,
            applied_mode: AwakeMode::Off,
            capabilities: Capabilities {
                system: false,
                display: false,
                backend: "Initializing".into(),
            },
            backend_error: None,
            notification_error: None,
            desktop_error: None,
            notes_revision: 0,
        })
    }

    fn save(&self, model: &Model) -> Result<(), String> {
        crate::persistence::save(&self.path, model)
    }
    pub fn desktop_notes(&self) -> Vec<crate::notes::Note> {
        self.model.notes.clone()
    }

    pub fn snapshot(&self) -> Snapshot {
        let now = now_ms();
        Snapshot {
            schema_version: self.model.schema_version,
            app_version: env!("CARGO_PKG_VERSION"),
            notes: self.model.notes.clone(),
            desktop_error: self.desktop_error.clone(),
            timers: self.model.timers.clone(),
            settings: self.model.settings.clone(),
            manual_mode: self.manual_mode,
            requested_mode: self.model.required_mode(self.manual_mode, now),
            applied_mode: self.applied_mode,
            capabilities: self.capabilities.clone(),
            backend_error: self
                .persistence_error
                .clone()
                .or_else(|| self.backend_error.clone()),
            notification_error: self.notification_error.clone(),
            now_ms: now,
        }
    }

    pub fn dispatch(&mut self, action: Action) -> Result<ActionResult, String> {
        let now = now_ms();
        let action = if let Action::Command { text } = action {
            match self.model.parse_command(&text, now)? {
                ParsedCommand::Action(action) => action,
                ParsedCommand::Reply(message) => {
                    return Ok(ActionResult {
                        message,
                        state: self.snapshot(),
                    });
                }
            }
        } else {
            action
        };
        let message = if let Action::SetAwake { mode } = action {
            self.check_mode(mode)?;
            self.manual_mode = mode;
            "Request updated; waiting for system confirmation.".into()
        } else {
            if matches!(
                action,
                Action::AddTimer {
                    keep_awake: true,
                    ..
                }
            ) {
                self.check_mode(AwakeMode::System)?;
            }
            // Commit only after the write succeeds: the UI never sees unsaved changes as saved.
            let mut next = self.model.clone();
            let message = next.apply(&action, now)?;
            self.save(&next)?;
            self.model = next;
            self.notes_revision = self.notes_revision.wrapping_add(1);
            message
        };
        Ok(ActionResult {
            message,
            state: self.snapshot(),
        })
    }

    fn check_mode(&self, mode: AwakeMode) -> Result<(), String> {
        match mode {
            AwakeMode::System if !self.capabilities.system => {
                Err("Keeping the system awake is unavailable.".into())
            }
            AwakeMode::Display if !self.capabilities.display => {
                Err("Keeping the display on is unavailable on this platform.".into())
            }
            _ => Ok(()),
        }
    }

    pub fn tick(&mut self) -> (Vec<Timer>, bool, AwakeMode) {
        let mut next = self.model.clone();
        let now = now_ms();
        let mut expired = next.expire(now);
        if !expired.is_empty() {
            match self.save(&next) {
                Ok(()) => {
                    self.model = next;
                    self.persistence_error = None;
                }
                Err(error) => {
                    self.persistence_error = Some(error);
                    expired.clear();
                }
            }
        }
        // Even when storage fails, an overdue timer must release its awake request.
        (
            expired,
            self.model.settings.notifications,
            self.model.required_mode(self.manual_mode, now),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Mascot;

    #[test]
    fn older_preferences_keep_the_chicken() {
        let settings: Settings =
            serde_json::from_str(r#"{"animations":false,"notifications":true}"#).unwrap();
        assert_eq!(settings.mascot, Mascot::Chicken);
        assert!(!settings.animations);
    }

    #[test]
    fn note_contents_visibility_and_geometry_survive_reload_and_failed_saves() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut service = Service::load(path.clone()).unwrap();
        service.dispatch(Action::CreateNote).unwrap();
        let id = service.snapshot().notes[0].id;
        service
            .dispatch(Action::UpdateNote {
                id,
                expected_revision: 0,
                title: "Saved note".into(),
                body: "Keep me".into(),
            })
            .unwrap();
        service
            .dispatch(Action::SetNoteVisible { id, visible: false })
            .unwrap();
        service
            .dispatch(Action::SetNoteBounds {
                id,
                bounds: crate::notes::NoteBounds {
                    x: -1400,
                    y: 200,
                    width: 350.0,
                    height: 280.0,
                },
            })
            .unwrap();
        let restored = Service::load(path).unwrap().snapshot();
        assert_eq!(restored.notes[0].body, "Keep me");
        assert!(!restored.notes[0].visible);
        assert_eq!(restored.notes[0].bounds.x, -1400);
        service.path = dir.path().join("missing/state.json");
        assert!(
            service
                .dispatch(Action::UpdateNote {
                    id,
                    expected_revision: 1,
                    title: "Unsaved".into(),
                    body: "Lost".into()
                })
                .is_err()
        );
        assert_eq!(service.snapshot().notes[0].body, "Keep me");
        assert_eq!(service.snapshot().notes[0].content_revision, 1);
    }

    #[test]
    fn saved_timer_and_preferences_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut service = Service::load(path.clone()).unwrap();
        service
            .dispatch(Action::AddTimer {
                name: "Tè".into(),
                seconds: 300,
                keep_awake: false,
            })
            .unwrap();
        service
            .dispatch(Action::UpdateSettings {
                settings: Settings {
                    mascot: Mascot::Elf,
                    animations: false,
                    notifications: false,
                    ..Settings::default()
                },
            })
            .unwrap();
        let restored = Service::load(path).unwrap().snapshot();
        assert_eq!(restored.timers.len(), 1);
        assert!(!restored.settings.animations);
        assert_eq!(restored.settings.mascot, crate::core::MascotKind::Chicken);
        assert_eq!(restored.manual_mode, AwakeMode::Off);
    }

    #[test]
    fn failed_save_does_not_commit_in_memory() {
        let dir = tempfile::tempdir().unwrap();
        let mut service = Service::load(dir.path().join("missing/state.json")).unwrap();
        assert!(
            service
                .dispatch(Action::AddTimer {
                    name: "Tè".into(),
                    seconds: 300,
                    keep_awake: false
                })
                .is_err()
        );
        assert!(service.snapshot().timers.is_empty());
    }

    #[test]
    fn corrupt_file_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        fs::write(&path, "not json").unwrap();
        assert!(Service::load(path.clone()).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "not json");
    }

    #[test]
    fn failed_expiry_save_does_not_keep_system_awake() {
        let dir = tempfile::tempdir().unwrap();
        let mut service = Service::load(dir.path().join("missing/state.json")).unwrap();
        service.model.add("Scaduto", 1, true, 0).unwrap();
        let (notifications, _, mode) = service.tick();
        assert_eq!(mode, AwakeMode::Off);
        assert!(notifications.is_empty());
        assert!(service.snapshot().backend_error.is_some());
        service.path = dir.path().join("state.json");
        assert_eq!(service.tick().0.len(), 1);
        assert!(service.snapshot().backend_error.is_none());
    }
}
