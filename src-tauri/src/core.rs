use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_SECONDS: u64 = 7 * 24 * 60 * 60;
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mascot {
    #[default]
    Chicken,
    Elf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default)]
    pub mascot: Mascot,
    pub animations: bool,
    pub notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mascot: Mascot::default(),
            animations: true,
            notifications: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Model {
    pub schema_version: u32,
    pub timers: Vec<Timer>,
    pub settings: Settings,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            schema_version: 1,
            timers: vec![],
            settings: Settings::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
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
        if self.schema_version != 1 {
            return Err(
                "Versione del file dati non supportata. Il file è stato conservato.".into(),
            );
        }
        if self.timers.len() > MAX_TIMERS {
            return Err("Il file dati contiene troppi timer.".into());
        }
        let mut ids = std::collections::HashSet::new();
        for timer in &self.timers {
            validate_name(&timer.name)?;
            validate_duration(timer.duration_seconds)?;
            if !ids.insert(timer.id) {
                return Err("Il file dati contiene identificatori duplicati.".into());
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
            return Err("Hai raggiunto 100 timer. Svuota prima lo storico.".into());
        }
        let deadline_ms = now
            .checked_add(seconds * 1000)
            .ok_or("Scadenza non valida")?;
        self.timers.push(Timer {
            id: Uuid::new_v4(),
            name: name.into(),
            deadline_ms,
            duration_seconds: seconds,
            status: TimerStatus::Running,
            keep_awake,
        });
        Ok(format!("Va bene, timer «{name}» avviato."))
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
            Action::AddTimer {
                name,
                seconds,
                keep_awake,
            } => self.add(name, *seconds, *keep_awake, now),
            Action::CancelTimer { id } => {
                let t = self.timer_mut(*id)?;
                if t.status != TimerStatus::Running || t.deadline_ms <= now {
                    return Err("Questo timer non è più in corso.".into());
                }
                t.status = TimerStatus::Cancelled;
                Ok(format!("Timer «{}» annullato.", t.name))
            }
            Action::AcknowledgeTimer { id } => {
                let t = self.timer_mut(*id)?;
                if t.status != TimerStatus::Expired {
                    return Err("Questo timer non è scaduto.".into());
                }
                t.status = TimerStatus::Acknowledged;
                Ok("Avviso chiuso.".into())
            }
            Action::ExtendTimer { id, seconds } => {
                validate_duration(*seconds)?;
                let t = self.timer_mut(*id)?;
                if t.status != TimerStatus::Running || t.deadline_ms <= now {
                    return Err("Il timer è già terminato. Creane uno nuovo.".into());
                }
                let duration = t
                    .duration_seconds
                    .checked_add(*seconds)
                    .ok_or("Durata eccessiva")?;
                validate_duration(duration)?;
                t.deadline_ms = t
                    .deadline_ms
                    .checked_add(seconds * 1000)
                    .ok_or("Scadenza non valida")?;
                t.duration_seconds = duration;
                Ok(format!("Tempo aggiunto al timer «{}».", t.name))
            }
            Action::UpdateSettings { settings } => {
                self.settings = settings.clone();
                Ok("Preferenze salvate.".into())
            }
            Action::ClearHistory => {
                self.timers
                    .retain(|t| matches!(t.status, TimerStatus::Running | TimerStatus::Expired));
                Ok("Storico svuotato. I timer e gli avvisi attivi sono conservati.".into())
            }
            _ => Err("Questa azione deve essere gestita dal servizio.".into()),
        }
    }

    fn timer_mut(&mut self, id: Uuid) -> Result<&mut Timer, String> {
        self.timers
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or("Timer non trovato.".into())
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
            [] => Err("Non trovo un timer attivo con questo nome.".into()),
            _ => Err(format!(
                "Quale timer? Usa i pulsanti oppure specifica un nome univoco: {}.",
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
            return Err("Il comando è troppo lungo.".into());
        }
        let text = input
            .trim()
            .trim_end_matches(['?', '.', '!'])
            .to_lowercase();
        if matches!(text.as_str(), "quanto manca" | "timer" | "lista timer") {
            let timers: Vec<_> = self
                .timers
                .iter()
                .filter(|t| t.status == TimerStatus::Running)
                .collect();
            let message = if timers.is_empty() {
                "Non ci sono timer in corso.".into()
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
        if let Some(duration) = text.strip_prefix("avvisami tra ") {
            return Ok(ParsedCommand::Action(Action::AddTimer {
                name: "Promemoria".into(),
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
            .strip_prefix("annulla il timer")
            .or_else(|| text.strip_prefix("annulla timer"))
        {
            let name = (!rest.trim().is_empty()).then_some(rest.trim());
            return Ok(ParsedCommand::Action(Action::CancelTimer {
                id: self.resolve_running(name, now)?.id,
            }));
        }
        if let Some(rest) = text.strip_prefix("aggiungi ") {
            let (duration, name) = rest.split_once(" al timer").ok_or_else(command_help)?;
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
    "Prova: «timer tè 5 minuti», «avvisami tra 30 minuti», «quanto manca?», «aggiungi 5 minuti al timer tè» o «annulla il timer tè».".into()
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() || name.chars().count() > 60 || name.chars().any(char::is_control) {
        Err("Scegli un nome da 1 a 60 caratteri, senza caratteri di controllo.".into())
    } else {
        Ok(())
    }
}

fn validate_duration(seconds: u64) -> Result<(), String> {
    if !(1..=MAX_SECONDS).contains(&seconds) {
        Err("La durata deve essere tra 1 secondo e 7 giorni.".into())
    } else {
        Ok(())
    }
}

fn parse_duration(input: &str) -> Result<u64, String> {
    let parts: Vec<_> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("Scrivi una durata come «30 minuti» o «10 secondi».".into());
    }
    let amount = parts[0]
        .parse::<u64>()
        .map_err(|_| "La durata deve essere un numero intero positivo.")?;
    let multiplier = match parts[1] {
        "secondo" | "secondi" | "s" => 1,
        "minuto" | "minuti" | "min" => 60,
        "ora" | "ore" | "h" => 3600,
        _ => return Err("Usa secondi, minuti oppure ore.".into()),
    };
    let seconds = amount.checked_mul(multiplier).ok_or("Durata eccessiva")?;
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
                .contains("Quale timer")
        );
        assert!(
            model
                .timers
                .iter()
                .all(|t| t.status == TimerStatus::Running)
        );
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
