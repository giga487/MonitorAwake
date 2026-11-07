use crate::core::{Action, Model, TimerStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_NOTES: usize = 100;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoteColor {
    #[default]
    Yellow,
    Sage,
    Blue,
    Rose,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notes_hide_without_deleting_and_conflicting_content_is_rejected() {
        let mut model = Model::default();
        model.apply(&Action::CreateNote, 0).unwrap();
        let id = model.notes[0].id;
        model
            .apply(
                &Action::UpdateNote {
                    id,
                    expected_revision: 0,
                    title: "Thought".into(),
                    body: "Keep this\nSecond line".into(),
                },
                0,
            )
            .unwrap();
        model
            .apply(&Action::SetNoteVisible { id, visible: false }, 0)
            .unwrap();
        assert_eq!(model.notes[0].body, "Keep this\nSecond line");
        assert!(!model.notes[0].visible);
        assert!(
            model
                .apply(
                    &Action::UpdateNote {
                        id,
                        expected_revision: 0,
                        title: "Stale".into(),
                        body: "Overwrite".into()
                    },
                    0
                )
                .is_err()
        );
        assert_eq!(model.notes[0].title, "Thought");
        model
            .apply(
                &Action::SetNoteAppearance {
                    id,
                    color: NoteColor::Blue,
                    pinned: true,
                },
                0,
            )
            .unwrap();
        assert_eq!(model.notes[0].content_revision, 1);
        assert!(model.notes[0].pinned);
        model
            .apply(&Action::SetAllNotesVisible { visible: true }, 0)
            .unwrap();
        assert!(model.notes[0].visible);
        model.validate().unwrap();
    }
    #[test]
    fn deleting_a_note_cancels_its_reminder_and_history_does_not_leave_dangling_links() {
        let mut model = Model::default();
        model.apply(&Action::CreateNote, 0).unwrap();
        let id = model.notes[0].id;
        model
            .apply(&Action::SetNoteReminder { id, seconds: 60 }, 0)
            .unwrap();
        let timer = model.notes[0].reminder_id.unwrap();
        assert!(
            model
                .apply(&Action::SetNoteReminder { id, seconds: 30 }, 0)
                .is_err()
        );
        model.apply(&Action::CancelTimer { id: timer }, 0).unwrap();
        model.apply(&Action::ClearHistory, 0).unwrap();
        assert_eq!(model.notes[0].reminder_id, None);
        model.validate().unwrap();
        model
            .apply(&Action::SetNoteReminder { id, seconds: 60 }, 0)
            .unwrap();
        model.apply(&Action::DeleteNote { id }, 0).unwrap();
        assert!(model.notes.is_empty());
        assert_eq!(model.timers[0].status, TimerStatus::Cancelled);
        model.validate().unwrap();
    }
    #[test]
    fn note_and_phrase_limits_reject_invalid_payloads() {
        assert!(validate_text(&"x".repeat(81), "").is_err());
        assert!(validate_text("ok", &"x".repeat(10_001)).is_err());
        assert!(
            NoteBounds {
                width: f64::NAN,
                ..NoteBounds::default()
            }
            .validate()
            .is_err()
        );
        let mut settings = crate::core::Settings {
            phrases: vec!["x".repeat(181)],
            ..crate::core::Settings::default()
        };
        assert!(settings.validate().is_err());
        settings.phrases = vec![];
        assert!(settings.validate().is_ok());
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoteBounds {
    // Global physical pixels for position; logical pixels for size.
    pub x: i32,
    pub y: i32,
    pub width: f64,
    pub height: f64,
}
impl Default for NoteBounds {
    fn default() -> Self {
        Self {
            x: 120,
            y: 120,
            width: 300.0,
            height: 320.0,
        }
    }
}
impl NoteBounds {
    pub fn validate(&self) -> Result<(), String> {
        if !self.width.is_finite()
            || !self.height.is_finite()
            || !(240.0..=1600.0).contains(&self.width)
            || !(200.0..=1600.0).contains(&self.height)
        {
            return Err("Note size must be between 240 × 200 and 1600 × 1600.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub color: NoteColor,
    pub visible: bool,
    pub pinned: bool,
    pub bounds: NoteBounds,
    pub content_revision: u64,
    pub reminder_id: Option<Uuid>,
}
pub fn validate_text(title: &str, body: &str) -> Result<(), String> {
    if title.chars().count() > 80
        || title.chars().any(char::is_control)
        || body.chars().count() > 10_000
        || body
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err("Use up to 80 characters for the title and 10,000 for the note.".into());
    }
    Ok(())
}

impl Model {
    pub fn apply_note(&mut self, action: &Action, now: u64) -> Result<String, String> {
        match action {
            Action::CreateNote => {
                if self.notes.len() >= MAX_NOTES {
                    return Err("You have reached 100 notes. Delete an unused note first.".into());
                }
                let offset = (self.notes.len() % 10) as i32 * 24;
                self.notes.push(Note {
                    id: Uuid::new_v4(),
                    title: "Untitled note".into(),
                    body: String::new(),
                    color: NoteColor::Yellow,
                    visible: true,
                    pinned: false,
                    bounds: NoteBounds {
                        x: 120 + offset,
                        y: 120 + offset,
                        ..NoteBounds::default()
                    },
                    content_revision: 0,
                    reminder_id: None,
                });
                Ok("Note created.".into())
            }
            Action::UpdateNote {
                id,
                expected_revision,
                title,
                body,
            } => {
                validate_text(title, body)?;
                let note = self.note_mut(*id)?;
                if note.content_revision != *expected_revision {
                    return Err(
                        "This note changed elsewhere. Reload it or save your version explicitly."
                            .into(),
                    );
                }
                note.content_revision = note
                    .content_revision
                    .checked_add(1)
                    .ok_or("Note revision overflow")?;
                note.title.clone_from(title);
                note.body.clone_from(body);
                Ok("Note saved.".into())
            }
            Action::SetNoteAppearance { id, color, pinned } => {
                let note = self.note_mut(*id)?;
                note.color = *color;
                note.pinned = *pinned;
                Ok("Note appearance saved.".into())
            }
            Action::SetNoteVisible { id, visible } => {
                self.note_mut(*id)?.visible = *visible;
                Ok("Note visibility saved.".into())
            }
            Action::SetAllNotesVisible { visible } => {
                for note in &mut self.notes {
                    note.visible = *visible;
                }
                Ok("Note visibility saved.".into())
            }
            Action::SetNoteBounds { id, bounds } => {
                bounds.validate()?;
                self.note_mut(*id)?.bounds = bounds.clone();
                Ok("Note position saved.".into())
            }
            Action::DeleteNote { id } => {
                let linked = self.note_mut(*id)?.reminder_id;
                self.notes.retain(|note| note.id != *id);
                if let Some(timer) = self
                    .timers
                    .iter_mut()
                    .find(|timer| Some(timer.id) == linked)
                {
                    timer.status = match timer.status {
                        TimerStatus::Running => TimerStatus::Cancelled,
                        TimerStatus::Expired => TimerStatus::Acknowledged,
                        other => other,
                    };
                }
                Ok("Note deleted.".into())
            }
            Action::SetNoteReminder { id, seconds } => {
                let note = self
                    .notes
                    .iter()
                    .find(|note| note.id == *id)
                    .ok_or("Note not found.")?;
                if self.timers.iter().any(|timer| {
                    Some(timer.id) == note.reminder_id
                        && matches!(timer.status, TimerStatus::Running | TimerStatus::Expired)
                }) {
                    return Err("Dismiss or cancel this note's existing reminder first.".into());
                }
                let title: String = if note.title.trim().is_empty() {
                    "Note reminder".into()
                } else {
                    note.title.trim().chars().take(60).collect()
                };
                self.add(&title, *seconds, false, now)?;
                let timer_id = self.timers.last().ok_or("Timer not created")?.id;
                self.note_mut(*id)?.reminder_id = Some(timer_id);
                Ok("Note reminder started.".into())
            }
            _ => Err("Not a note action.".into()),
        }
    }
    fn note_mut(&mut self, id: Uuid) -> Result<&mut Note, String> {
        self.notes
            .iter_mut()
            .find(|note| note.id == id)
            .ok_or("Note not found.".into())
    }
}
