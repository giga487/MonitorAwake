use crate::notes::{Note, NoteBounds, NoteColor};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 2;

pub const MAX_SECONDS: u64 = 100 * 24 * 60 * 60;
pub const MAX_TIMERS: usize = 100;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AwakeMode {
    #[default]
    Off,
    System,
    Display,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimerStatus {
    Running,
    Expired,
    Acknowledged,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timer {
    pub id: Uuid,
    pub name: String,
    pub deadline_ms: u64,
    pub duration_seconds: u64,
    pub status: TimerStatus,
    pub keep_awake: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub animations: bool,
    pub notifications: bool,
    #[serde(default)]
    pub mascot: MascotKind,
    #[serde(default = "default_phrases")]
    pub phrases: Vec<String>,
    #[serde(default = "enabled")]
    pub speech_enabled: bool,
}
fn enabled() -> bool {
    true
}
pub fn default_phrases() -> Vec<String> {
    [
        "One thing at a time.",
        "A small break can hatch a good idea.",
        "I am keeping an eye on the time.",
        "Got a thought? Give it a note.",
        "Stretch your wings for a moment.",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.phrases.len() > 50
            || self.phrases.iter().any(|text| {
                text.trim().is_empty()
                    || text.chars().count() > 180
                    || text.chars().any(char::is_control)
            })
        {
            return Err(
                "Use up to 50 phrases, each containing 1–180 characters on one line.".into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MascotKind {
    #[default]
    Chicken,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            animations: true,
            notifications: true,
            mascot: MascotKind::Chicken,
            phrases: default_phrases(),
            speech_enabled: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub schema_version: u32,
    pub timers: Vec<Timer>,
    pub settings: Settings,
    pub notes: Vec<Note>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            timers: vec![],
            settings: Settings::default(),
            notes: vec![],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    CreateNote,
    UpdateNote {
        id: Uuid,
        expected_revision: u64,
        title: String,
        body: String,
    },
    SetNoteAppearance {
        id: Uuid,
        color: NoteColor,
        pinned: bool,
    },
    SetNoteVisible {
        id: Uuid,
        visible: bool,
    },
    SetAllNotesVisible {
        visible: bool,
    },
    SetNoteBounds {
        id: Uuid,
        bounds: NoteBounds,
    },
    DeleteNote {
        id: Uuid,
    },
    SetNoteReminder {
        id: Uuid,
        seconds: u64,
    },
    AddTimer {
        name: String,
        seconds: u64,
        keep_awake: bool,
    },
    CancelTimer {
        id: Uuid,
    },
    AcknowledgeTimer {
        id: Uuid,
    },
    ExtendTimer {
        id: Uuid,
        seconds: u64,
    },
    SetAwake {
        mode: AwakeMode,
    },
    UpdateSettings {
        settings: Settings,
    },
    ClearHistory,
    Command {
        text: String,
    },
}

impl Model {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err("Unsupported data file version. The file has been preserved.".into());
        }
        self.settings.validate()?;
        if self.notes.len() > crate::notes::MAX_NOTES {
            return Err("The data file contains too many notes.".into());
        }
        if self.timers.len() > MAX_TIMERS {
            return Err("The data file contains too many timers.".into());
        }
        let mut ids = std::collections::HashSet::new();
        for timer in &self.timers {
            validate_name(&timer.name)?;
            validate_duration(timer.duration_seconds)?;
            if !ids.insert(timer.id) {
                return Err("The data file contains duplicate IDs.".into());
            }
        }
        let mut note_reminders = std::collections::HashSet::new();
        for note in &self.notes {
            crate::notes::validate_text(&note.title, &note.body)?;
            note.bounds.validate()?;
            if !ids.insert(note.id) {
                return Err("The data file contains duplicate IDs.".into());
            }
            if note
                .reminder_id
                .is_some_and(|id| !self.timers.iter().any(|timer| timer.id == id))
            {
                return Err("A note references a missing reminder.".into());
            }
            if let Some(id) = note.reminder_id
                && !note_reminders.insert(id)
            {
                return Err("Multiple notes reference the same reminder.".into());
            }
        }
        Ok(())
    }

    pub fn add(
        &mut self,
        name: &str,
        seconds: u64,
        keep_awake: bool,
        now: u64,
    ) -> Result<String, String> {
        let name = name.trim();
        validate_name(name)?;
        validate_duration(seconds)?;
        if self.timers.len() >= MAX_TIMERS {
            return Err("You have reached 100 timers. Clear the history first.".into());
        }
        let deadline_ms = now.checked_add(seconds * 1000).ok_or("Invalid deadline")?;
        self.timers.push(Timer {
            id: Uuid::new_v4(),
            name: name.into(),
            deadline_ms,
            duration_seconds: seconds,
            status: TimerStatus::Running,
            keep_awake,
        });
        Ok(format!("Timer «{name}» started."))
    }

    pub fn expire(&mut self, now: u64) -> Vec<Timer> {
        let mut expired = vec![];
        for timer in &mut self.timers {
            if timer.status == TimerStatus::Running && timer.deadline_ms <= now {
                timer.status = TimerStatus::Expired;
                expired.push(timer.clone());
            }
        }
        expired
    }

    pub fn required_mode(&self, manual: AwakeMode, now: u64) -> AwakeMode {
        if manual != AwakeMode::Off {
            manual
        } else if self
            .timers
            .iter()
            .any(|t| t.status == TimerStatus::Running && t.keep_awake && t.deadline_ms > now)
        {
            AwakeMode::System
        } else {
            AwakeMode::Off
        }
    }

    pub fn apply(&mut self, action: &Action, now: u64) -> Result<String, String> {
        match action {
            Action::CreateNote
            | Action::UpdateNote { .. }
            | Action::SetNoteAppearance { .. }
            | Action::SetNoteVisible { .. }
            | Action::SetAllNotesVisible { .. }
            | Action::SetNoteBounds { .. }
            | Action::DeleteNote { .. }
            | Action::SetNoteReminder { .. } => self.apply_note(action, now),
            Action::AddTimer {
                name,
                seconds,
                keep_awake,
            } => self.add(name, *seconds, *keep_awake, now),
            Action::CancelTimer { id } => {
                let t = self.timer_mut(*id)?;
                if t.status != TimerStatus::Running || t.deadline_ms <= now {
                    return Err("This timer is no longer running.".into());
                }
                t.status = TimerStatus::Cancelled;
                Ok(format!("Timer «{}» cancelled.", t.name))
            }
            Action::AcknowledgeTimer { id } => {
                let t = self.timer_mut(*id)?;
                if t.status != TimerStatus::Expired {
                    return Err("This timer has not expired.".into());
                }
                t.status = TimerStatus::Acknowledged;
                Ok("Alert dismissed.".into())
            }
            Action::ExtendTimer { id, seconds } => {
                validate_duration(*seconds)?;
                let t = self.timer_mut(*id)?;
                if t.status != TimerStatus::Running || t.deadline_ms <= now {
                    return Err("This timer has already ended. Create a new one.".into());
                }
                let duration = t
                    .duration_seconds
                    .checked_add(*seconds)
                    .ok_or("Duration is too large")?;
                validate_duration(duration)?;
                t.deadline_ms = t
                    .deadline_ms
                    .checked_add(seconds * 1000)
                    .ok_or("Invalid deadline")?;
                t.duration_seconds = duration;
                Ok(format!("Time added to timer «{}».", t.name))
            }
            Action::UpdateSettings { settings } => {
                settings.validate()?;
                self.settings = settings.clone();
                Ok("Preferences saved.".into())
            }
            Action::ClearHistory => {
                self.timers
                    .retain(|t| matches!(t.status, TimerStatus::Running | TimerStatus::Expired));
                for note in &mut self.notes {
                    if note
                        .reminder_id
                        .is_some_and(|id| !self.timers.iter().any(|timer| timer.id == id))
                    {
                        note.reminder_id = None;
                    }
                }
                Ok("History cleared. Active timers and alerts have been kept.".into())
            }
            _ => Err("This action must be handled by the service.".into()),
        }
    }

    fn timer_mut(&mut self, id: Uuid) -> Result<&mut Timer, String> {
        self.timers
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or("Timer not found.".into())
    }

    fn resolve_running(&self, name: Option<&str>, now: u64) -> Result<&Timer, String> {
        let candidates: Vec<_> = self
            .timers
            .iter()
            .filter(|t| {
                t.status == TimerStatus::Running
                    && t.deadline_ms > now
                    && name.is_none_or(|n| t.name.to_lowercase() == n.to_lowercase())
            })
            .collect();
        match candidates.as_slice() {
            [timer] => Ok(timer),
            [] => Err("No active timer found with this name.".into()),
            _ => Err(format!(
                "Which timer? Use the buttons or specify a unique name: {}.",
                candidates
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    pub fn parse_command(&self, input: &str, now: u64) -> Result<ParsedCommand, String> {
        if input.len() > 300 {
            return Err("The command is too long.".into());
        }
        let text = input
            .trim()
            .trim_end_matches(['?', '.', '!'])
            .to_lowercase();
        if matches!(
            text.as_str(),
            "how much time is left"
                | "time left"
                | "list timers"
                | "timers"
                | "quanto manca"
                | "timer"
                | "lista timer"
        ) {
            let timers: Vec<_> = self
                .timers
                .iter()
                .filter(|t| t.status == TimerStatus::Running)
                .collect();
            let message = if timers.is_empty() {
                "There are no running timers.".into()
            } else {
                timers
                    .iter()
                    .map(|t| {
                        let seconds = t.deadline_ms.saturating_sub(now).div_ceil(1000);
                        format!("{}: {} min {} s", t.name, seconds / 60, seconds % 60)
                    })
                    .collect::<Vec<_>>()
                    .join(" · ")
            };
            return Ok(ParsedCommand::Reply(message));
        }
        if let Some(duration) = text
            .strip_prefix("remind me in ")
            .or_else(|| text.strip_prefix("avvisami tra "))
        {
            return Ok(ParsedCommand::Action(Action::AddTimer {
                name: "Reminder".into(),
                seconds: parse_duration(duration)?,
                keep_awake: false,
            }));
        }
        if let Some(rest) = text.strip_prefix("timer ") {
            let words: Vec<_> = rest.split_whitespace().collect();
            if words.len() < 2 {
                return Err(command_help());
            }
            let duration_start = words.len() - 2;
            let seconds = parse_duration(&words[duration_start..].join(" "))?;
            let name = if duration_start == 0 {
                "Timer".into()
            } else {
                words[..duration_start].join(" ")
            };
            return Ok(ParsedCommand::Action(Action::AddTimer {
                name,
                seconds,
                keep_awake: false,
            }));
        }
        if let Some(rest) = text
            .strip_prefix("cancel timer")
            .or_else(|| text.strip_prefix("annulla il timer"))
            .or_else(|| text.strip_prefix("annulla timer"))
        {
            if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
                return Err(command_help());
            }
            let name = (!rest.trim().is_empty()).then_some(rest.trim());
            return Ok(ParsedCommand::Action(Action::CancelTimer {
                id: self.resolve_running(name, now)?.id,
            }));
        }
        let extension = text
            .strip_prefix("add ")
            .map(|rest| (rest, " to timer"))
            .or_else(|| {
                text.strip_prefix("aggiungi ")
                    .map(|rest| (rest, " al timer"))
            });
        if let Some((rest, separator)) = extension {
            let (duration, name) = rest.split_once(separator).ok_or_else(command_help)?;
            if !name.is_empty() && !name.starts_with(char::is_whitespace) {
                return Err(command_help());
            }
            let name = (!name.trim().is_empty()).then_some(name.trim());
            return Ok(ParsedCommand::Action(Action::ExtendTimer {
                id: self.resolve_running(name, now)?.id,
                seconds: parse_duration(duration)?,
            }));
        }
        Err(command_help())
    }
}

pub enum ParsedCommand {
    Action(Action),
    Reply(String),
}

fn command_help() -> String {
    "Try: «timer tea 5 minutes», «remind me in 30 minutes», «how much time is left?», «add 5 minutes to timer tea» or «cancel timer tea».".into()
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() || name.chars().count() > 60 || name.chars().any(char::is_control) {
        Err("Choose a name with 1 to 60 characters and no control characters.".into())
    } else {
        Ok(())
    }
}

fn validate_duration(seconds: u64) -> Result<(), String> {
    if !(1..=MAX_SECONDS).contains(&seconds) {
        Err("Duration must be between 1 second and 100 days.".into())
    } else {
        Ok(())
    }
}

fn parse_duration(input: &str) -> Result<u64, String> {
    let parts: Vec<_> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("Enter a duration such as «30 minutes» or «10 seconds».".into());
    }
    let amount = parts[0]
        .parse::<u64>()
        .map_err(|_| "Duration must be a positive whole number.")?;
    let multiplier = match parts[1] {
        "second" | "seconds" | "sec" | "secondo" | "secondi" | "s" => 1,
        "minute" | "minutes" | "minuto" | "minuti" | "min" => 60,
        "hour" | "hours" | "ora" | "ore" | "h" => 3600,
        "day" | "days" | "giorno" | "giorni" | "d" => 86400,
        _ => return Err("Use seconds, minutes, hours or days.".into()),
    };
    let seconds = amount
        .checked_mul(multiplier)
        .ok_or("Duration is too large")?;
    validate_duration(seconds)?;
    Ok(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expiry_survives_reload_and_is_emitted_once() {
        let mut model = Model::default();
        model.add("Tè", 30, true, 1000).unwrap();
        let json = serde_json::to_string(&model).unwrap();
        let mut restored: Model = serde_json::from_str(&json).unwrap();
        assert!(restored.expire(30_999).is_empty());
        assert_eq!(restored.expire(31_000).len(), 1);
        assert!(restored.expire(90_000).is_empty());
        assert_eq!(
            restored.required_mode(AwakeMode::Off, 90_000),
            AwakeMode::Off
        );
    }

    #[test]
    fn awake_owners_are_independent() {
        let mut model = Model::default();
        model.add("Uno", 30, true, 0).unwrap();
        model.add("Due", 60, true, 0).unwrap();
        model.expire(30_000);
        assert_eq!(
            model.required_mode(AwakeMode::Off, 30_000),
            AwakeMode::System
        );
        assert_eq!(
            model.required_mode(AwakeMode::Display, 30_000),
            AwakeMode::Display
        );
        model.expire(60_000);
        assert_eq!(model.required_mode(AwakeMode::Off, 60_000), AwakeMode::Off);
    }

    #[test]
    fn ambiguous_cancel_does_not_change_any_timer() {
        let mut model = Model::default();
        model.add("Uno", 30, false, 0).unwrap();
        model.add("Due", 60, false, 0).unwrap();
        assert!(
            model
                .parse_command("annulla il timer", 0)
                .unwrap_err_text()
                .contains("Which timer")
        );
        assert!(
            model
                .timers
                .iter()
                .all(|t| t.status == TimerStatus::Running)
        );
    }

    #[test]
    fn old_preferences_default_to_chicken_and_unknown_mascots_are_rejected() {
        let settings: Settings =
            serde_json::from_str(r#"{"animations":false,"notifications":true}"#).unwrap();
        assert_eq!(settings.mascot, MascotKind::Chicken);
        assert!(!settings.animations);
        assert_eq!(
            serde_json::to_value(&settings).unwrap()["mascot"],
            "chicken"
        );
        assert!(
            serde_json::from_str::<Settings>(
                r#"{"animations":true,"notifications":true,"mascot":"unknown"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn supports_a_hundred_days_without_changing_expiry_rules() {
        let mut model = Model::default();
        model
            .add("Long reminder", MAX_SECONDS, false, 1000)
            .unwrap();
        assert!(model.add("Too long", MAX_SECONDS + 1, false, 1000).is_err());
        let json = serde_json::to_string(&model).unwrap();
        let mut restored: Model = serde_json::from_str(&json).unwrap();
        restored.validate().unwrap();
        let deadline = 1000 + MAX_SECONDS * 1000;
        assert!(restored.expire(deadline - 1).is_empty());
        assert_eq!(restored.expire(deadline).len(), 1);
        assert!(restored.expire(deadline + 1).is_empty());
    }

    #[test]
    fn parses_english_timer_commands_and_duration_units() {
        let model = Model::default();
        for (text, expected_name, expected_seconds) in [
            ("remind me in 30 minutes", "Reminder", 1800),
            ("Timer tea 5 minutes!", "tea", 300),
            ("timer 1 second", "Timer", 1),
            ("timer short break 10 seconds", "short break", 10),
            ("timer lunch 1 minute", "lunch", 60),
            ("timer work 1 hour", "work", 3600),
            ("timer work 2 hours", "work", 7200),
            ("timer holiday 100 days", "holiday", 8_640_000),
        ] {
            match model.parse_command(text, 0).unwrap() {
                ParsedCommand::Action(Action::AddTimer {
                    name,
                    seconds,
                    keep_awake,
                }) => {
                    assert_eq!(name, expected_name, "{text}");
                    assert_eq!(seconds, expected_seconds, "{text}");
                    assert!(!keep_awake);
                }
                _ => panic!("expected an add-timer action for {text}"),
            }
        }
        for text in [
            "timer tea 0 minutes",
            "timer tea -5 minutes",
            "timer tea 18446744073709551615 hours",
            "timer tea 2401 hours",
            "timer tea 101 days",
            "remind me in five minutes",
        ] {
            assert!(model.parse_command(text, 0).is_err(), "{text}");
        }
    }

    #[test]
    fn english_commands_resolve_targets_and_preserve_timer_rules() {
        let mut model = Model::default();
        model.add("Tea break", 60, false, 0).unwrap();
        model.add("Lunch", 120, false, 0).unwrap();
        let id = model.timers[0].id;
        for text in ["cancel timer", "add 5 minutes to timer"] {
            assert!(
                model
                    .parse_command(text, 0)
                    .unwrap_err_text()
                    .contains("Which timer")
            );
        }
        for text in ["cancel timerfoo", "add 5 minutes to timerfoo"] {
            assert!(
                model
                    .parse_command(text, 0)
                    .unwrap_err_text()
                    .starts_with("Try:")
            );
        }
        for text in [
            "how much time is left?",
            "time left",
            "list timers",
            "timers",
        ] {
            match model.parse_command(text, 0).unwrap() {
                ParsedCommand::Reply(message) => assert!(message.contains("Tea break: 1 min 0 s")),
                _ => panic!("expected a timer list"),
            }
        }
        match model
            .parse_command("add 5 minutes to timer tea break", 0)
            .unwrap()
        {
            ParsedCommand::Action(action @ Action::ExtendTimer { .. }) => {
                model.apply(&action, 0).unwrap();
                assert_eq!(model.timers[0].deadline_ms, 360_000);
            }
            _ => panic!("expected an extension"),
        }
        match model.parse_command("cancel timer TEA BREAK", 0).unwrap() {
            ParsedCommand::Action(action @ Action::CancelTimer { id: target }) => {
                assert_eq!(target, id);
                assert!(model.apply(&action, 0).unwrap().contains("cancelled"));
                assert_eq!(model.timers[0].status, TimerStatus::Cancelled);
            }
            _ => panic!("expected cancellation"),
        }
        assert_eq!(model.timers[1].status, TimerStatus::Running);
    }

    #[test]
    fn parses_italian_and_rejects_bad_durations() {
        let model = Model::default();
        for text in [
            "avvisami tra 30 minuti",
            "timer tè 5 minuti",
            "timer 10 secondi",
        ] {
            assert!(matches!(
                model.parse_command(text, 0),
                Ok(ParsedCommand::Action(Action::AddTimer { .. }))
            ));
        }
        for text in [
            "timer tè 0 minuti",
            "timer tè -5 minuti",
            "timer tè 18446744073709551615 ore",
            "rm file",
        ] {
            assert!(model.parse_command(text, 0).is_err());
        }
    }

    #[test]
    fn extension_at_deadline_cannot_resurrect_timer() {
        let mut model = Model::default();
        model.add("Tè", 1, false, 0).unwrap();
        let id = model.timers[0].id;
        assert!(
            model
                .apply(&Action::ExtendTimer { id, seconds: 5 }, 1000)
                .is_err()
        );
        assert_eq!(model.timers[0].deadline_ms, 1000);
    }

    trait ErrorText {
        fn unwrap_err_text(self) -> String;
    }
    impl ErrorText for Result<ParsedCommand, String> {
        fn unwrap_err_text(self) -> String {
            match self {
                Err(e) => e,
                Ok(_) => panic!("expected error"),
            }
        }
    }
}
