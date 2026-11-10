use crate::core::{MascotKind, Model, SCHEMA_VERSION, Settings, Timer};
use serde::Deserialize;
use std::{fs, io::Write, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyV1 {
    schema_version: u32,
    timers: Vec<Timer>,
    settings: Settings,
}

fn migrate_v2_to_v3(mut model: Model) -> Result<Model, String> {
    if model.schema_version != 2 || model.settings.mascot != MascotKind::Chicken {
        return Err(
            "Invalid schema 2 mascot settings. The original file has been preserved.".into(),
        );
    }
    model.schema_version = 3;
    Ok(model)
}

fn migrate_v3_to_v4(mut model: Model) -> Result<Model, String> {
    if model.schema_version != 3 || model.settings.mascot == MascotKind::EmberWyvern {
        return Err(
            "Invalid schema 3 mascot settings. The original file has been preserved.".into(),
        );
    }
    model.schema_version = 4;
    Ok(model)
}

fn migrate_v4_to_v5(mut model: Model) -> Result<Model, String> {
    if model.schema_version != 4 {
        return Err("Invalid schema 4. The original file has been preserved.".into());
    }
    model.schema_version = 5;
    Ok(model)
}

fn migrate_v5_to_v6(mut model: Model) -> Result<Model, String> {
    if model.schema_version != 5 || model.settings.mascot == MascotKind::Elf {
        return Err(
            "Invalid schema 5 mascot settings. The original file has been preserved.".into(),
        );
    }
    model.schema_version = 6;
    Ok(model)
}

fn migrate_v6_to_v7(mut model: Model) -> Result<Model, String> {
    if model.schema_version != 6
        || model
            .timers
            .iter()
            .any(|timer| timer.kind != crate::core::TimerKind::Timer)
    {
        return Err("Invalid schema 6 timer kind. The original file has been preserved.".into());
    }
    model.schema_version = 7;
    Ok(model)
}

pub fn save(path: &Path, model: &Model) -> Result<(), String> {
    model.validate()?;
    let parent = path.parent().ok_or("Invalid data directory")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("Saving: {e}"))?;
    serde_json::to_writer_pretty(&mut temporary, model).map_err(|e| format!("Saving: {e}"))?;
    if temporary
        .as_file()
        .metadata()
        .map_err(|e| e.to_string())?
        .len()
        > 8 * 1024 * 1024
    {
        return Err("The data file would exceed 8 MiB. Original data preserved.".into());
    }
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary
        .persist(path)
        .map_err(|e| format!("Replacing data file: {e}"))?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Model, String> {
    if !path.exists() {
        return Ok(Model::default());
    }
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
        return Err("The data file is too large. No data has been changed.".into());
    }
    let bytes = fs::read(path).map_err(|e| format!("Reading data: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid data file, preserved: {e}"))?;
    let version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid data schema version. The original file has been preserved.")?;
    let model = match version {
        1 => {
            let legacy: LegacyV1 = serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 1; original preserved: {e}"))?;
            if legacy.schema_version != 1 { return Err("Invalid legacy schema.".into()); }
            if legacy.settings.mascot == MascotKind::Elf {
                // The elf prototype wrote schema 1 before the two mascot catalogs merged.
                Model { timers: legacy.timers, settings: legacy.settings, ..Model::default() }
            } else {
                migrate_v3_to_v4(migrate_v2_to_v3(Model { schema_version: 2, timers: legacy.timers, settings: legacy.settings, notes: vec![], note_sync: Default::default() })?)?
            }
        }
        2 => migrate_v3_to_v4(migrate_v2_to_v3(serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 2; original preserved: {e}"))?)?)?,
        3 => migrate_v3_to_v4(serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 3; original preserved: {e}"))?)?,
        4 => serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 4; original preserved: {e}"))?,
        5 => serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 5; original preserved: {e}"))?,
        6 => serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 6; original preserved: {e}"))?,
        7 => serde_json::from_value(value).map_err(|e| format!("Cannot migrate schema 7; original preserved: {e}"))?,
        v if v == u64::from(SCHEMA_VERSION) => serde_json::from_value(value).map_err(|e| format!("Invalid data file, preserved: {e}"))?,
        _ => return Err("Unsupported data schema. Use a newer compatible version of MonitorAwake. The original file has been preserved.".into()),
    };
    let model = if model.schema_version == 4 {
        migrate_v4_to_v5(model)?
    } else {
        model
    };
    let model = if model.schema_version == 5 {
        migrate_v5_to_v6(model)?
    } else {
        model
    };
    let model = if model.schema_version == 6 {
        migrate_v6_to_v7(model)?
    } else {
        model
    };
    let model = if model.schema_version == 7 {
        if model.settings.notes_storage != crate::note_sync::NotesStorage::Local
            || model.note_sync.scope.is_some()
            || !model.note_sync.entries.is_empty()
        {
            return Err("Invalid schema 7 cloud metadata. Original data preserved.".into());
        }
        Model {
            schema_version: 8,
            ..model
        }
    } else {
        model
    };
    model.validate()?;
    if version != u64::from(SCHEMA_VERSION) {
        let parent = path.parent().ok_or("Invalid data directory")?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid data filename")?;
        let backup = parent.join(format!(
            "{filename}.backup-v{version}-{}.json",
            uuid::Uuid::new_v4()
        ));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|e| format!("Migration backup: {e}"))?;
        temporary
            .write_all(&bytes)
            .map_err(|e| format!("Migration backup: {e}"))?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary
            .persist_noclobber(backup)
            .map_err(|e| format!("Migration backup: {e}"))?;
        save(path, &model)?;
    }
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn v7_migration_keeps_all_notes_and_defaults_to_local_without_uploads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut before = Model::default();
        before.apply(&crate::core::Action::CreateNote, 0).unwrap();
        let mut value = serde_json::to_value(&before).unwrap();
        value["schema_version"] = serde_json::json!(7);
        value.as_object_mut().unwrap().remove("note_sync");
        value["settings"]
            .as_object_mut()
            .unwrap()
            .remove("notes_storage");
        let original = serde_json::to_vec(&value).unwrap();
        fs::write(&path, &original).unwrap();
        let after = load(&path).unwrap();
        assert_eq!(after.schema_version, SCHEMA_VERSION);
        assert_eq!(after.notes, before.notes);
        assert_eq!(
            after.settings.notes_storage,
            crate::note_sync::NotesStorage::Local
        );
        assert!(after.note_sync.scope.is_none());
        load(&path).unwrap();
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p != &path)
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
    }
    #[test]
    fn v6_migration_preserves_timers_and_notes_and_defaults_timer_kind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut before = Model::default();
        before.apply(&crate::core::Action::CreateNote, 0).unwrap();
        before
            .apply(
                &crate::core::Action::SetNoteReminder {
                    id: before.notes[0].id,
                    seconds: 60,
                },
                0,
            )
            .unwrap();
        before.settings.phrases = vec!["La mia frase.".into()];
        before.schema_version = 6;
        let mut legacy = serde_json::to_value(&before).unwrap();
        legacy["timers"][0].as_object_mut().unwrap().remove("kind");
        let original = serde_json::to_vec(&legacy).unwrap();
        fs::write(&path, &original).unwrap();
        let mut after = load(&path).unwrap();
        assert_eq!(after.schema_version, SCHEMA_VERSION);
        assert_eq!(after.notes, before.notes);
        assert_eq!(after.settings.phrases, before.settings.phrases);
        assert_eq!(after.timers[0].id, before.timers[0].id);
        assert_eq!(after.timers[0].kind, crate::core::TimerKind::Timer);
        after
            .apply(
                &crate::core::Action::AddAlarm {
                    name: "Saved alarm".into(),
                    deadline_ms: 90_123,
                    keep_awake: false,
                },
                0,
            )
            .unwrap();
        save(&path, &after).unwrap();
        let restored = load(&path).unwrap();
        assert_eq!(restored.timers[1].kind, crate::core::TimerKind::Alarm);
        assert_eq!(restored.timers[1].deadline_ms, 90_123);
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p != &path)
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
    }
    #[test]
    fn migration_preserves_exact_backup_and_only_runs_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let original = br#"{"schema_version":1,"timers":[{"id":"9b40e03d-b37b-415d-9003-09e746df46ca","name":"Preserved timer","deadline_ms":12345000,"duration_seconds":300,"status":"running","keep_awake":true}],"settings":{"animations":false,"notifications":true}}"#;
        fs::write(&path, original).unwrap();
        let model = load(&path).unwrap();
        assert_eq!(model.schema_version, SCHEMA_VERSION);
        assert!(!model.settings.animations);
        assert!(model.notes.is_empty());
        assert_eq!(model.timers[0].name, "Preserved timer");
        assert_eq!(model.timers[0].deadline_ms, 12_345_000);
        assert!(model.timers[0].keep_awake);
        let files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|p| p != &path)
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(fs::read(&files[0]).unwrap(), original);
        load(&path).unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
    #[test]
    fn v2_migration_keeps_notes_personal_phrases_and_reminders() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut before = Model::default();
        before.apply(&crate::core::Action::CreateNote, 0).unwrap();
        let id = before.notes[0].id;
        before
            .apply(
                &crate::core::Action::UpdateNote {
                    id,
                    expected_revision: 0,
                    title: "Personal note".into(),
                    body: "Keep this text".into(),
                },
                0,
            )
            .unwrap();
        before
            .apply(&crate::core::Action::SetNoteReminder { id, seconds: 60 }, 0)
            .unwrap();
        before.settings.phrases = vec!["Una piccola abitudine alla volta.".into()];
        before.schema_version = 2;
        let original = serde_json::to_vec_pretty(&before).unwrap();
        fs::write(&path, &original).unwrap();
        let mut after = load(&path).unwrap();
        assert_eq!(after.schema_version, SCHEMA_VERSION);
        assert_eq!(after.notes, before.notes);
        assert_eq!(after.settings.phrases, before.settings.phrases);
        assert_eq!(after.timers[0].id, before.timers[0].id);
        after.settings.mascot = MascotKind::FrostGryphon;
        save(&path, &after).unwrap();
        assert_eq!(
            load(&path).unwrap().settings.mascot,
            MascotKind::FrostGryphon
        );
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|candidate| candidate != &path)
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
    }

    #[test]
    fn v3_migration_preserves_gryphon_and_accepts_wyvern_afterwards() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut before = Model::default();
        before.apply(&crate::core::Action::CreateNote, 0).unwrap();
        before.settings.mascot = MascotKind::FrostGryphon;
        before.settings.phrases = vec!["Un passo alla volta.".into()];
        before.schema_version = 3;
        let original = serde_json::to_vec_pretty(&before).unwrap();
        fs::write(&path, &original).unwrap();
        let mut after = load(&path).unwrap();
        assert_eq!(after.schema_version, SCHEMA_VERSION);
        assert_eq!(
            serde_json::to_value(&after.settings).unwrap(),
            serde_json::to_value(&before.settings).unwrap()
        );
        assert_eq!(after.notes, before.notes);
        after.settings.mascot = MascotKind::EmberWyvern;
        save(&path, &after).unwrap();
        assert_eq!(
            load(&path).unwrap().settings.mascot,
            MascotKind::EmberWyvern
        );
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|candidate| candidate != &path)
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
    }

    #[test]
    fn v4_migration_adds_speech_interval_and_preserves_personal_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut before = Model::default();
        before.apply(&crate::core::Action::CreateNote, 0).unwrap();
        let note_id = before.notes[0].id;
        before
            .apply(
                &crate::core::Action::SetNoteReminder {
                    id: note_id,
                    seconds: 60,
                },
                0,
            )
            .unwrap();
        before.settings.mascot = MascotKind::EmberWyvern;
        before.settings.phrases = vec!["Una frase personale.".into()];
        before.schema_version = 4;
        let mut legacy = serde_json::to_value(&before).unwrap();
        let settings = legacy["settings"].as_object_mut().unwrap();
        settings.remove("speech_min_minutes");
        settings.remove("speech_max_minutes");
        let original = serde_json::to_vec_pretty(&legacy).unwrap();
        fs::write(&path, &original).unwrap();
        let mut after = load(&path).unwrap();
        assert_eq!(after.schema_version, SCHEMA_VERSION);
        assert_eq!(after.settings.speech_min_minutes, 5);
        assert_eq!(after.settings.speech_max_minutes, 10);
        assert_eq!(after.settings.phrases, before.settings.phrases);
        assert_eq!(after.settings.mascot, MascotKind::EmberWyvern);
        assert_eq!(after.notes, before.notes);
        assert_eq!(after.timers[0].id, before.timers[0].id);
        after.settings.speech_min_minutes = 7;
        after.settings.speech_max_minutes = 7;
        save(&path, &after).unwrap();
        let restored = load(&path).unwrap();
        assert_eq!(restored.settings.speech_min_minutes, 7);
        assert_eq!(restored.settings.speech_max_minutes, 7);
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p != &path)
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
    }

    #[test]
    fn speech_interval_rejects_invalid_bounds() {
        let mut settings = Settings::default();
        for (min, max) in [(0, 10), (10, 5), (1, 1441)] {
            settings.speech_min_minutes = min;
            settings.speech_max_minutes = max;
            assert!(settings.validate().is_err());
        }
        for (min, max) in [(1, 1), (5, 10), (1440, 1440)] {
            settings.speech_min_minutes = min;
            settings.speech_max_minutes = max;
            assert!(settings.validate().is_ok());
        }
    }

    #[test]
    fn migrates_elf_prototype_and_released_v5_profiles() {
        for (version, mascot) in [(1, MascotKind::Elf), (5, MascotKind::EmberWyvern)] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("state.json");
            let mut profile = serde_json::json!({
                "schema_version": version, "timers": [],
                "settings": { "animations": false, "notifications": true, "mascot": mascot }
            });
            if version == 5 {
                profile["notes"] = serde_json::json!([]);
            }
            let original = serde_json::to_vec(&profile).unwrap();
            fs::write(&path, &original).unwrap();
            let migrated = load(&path).unwrap();
            assert_eq!(migrated.schema_version, SCHEMA_VERSION);
            assert_eq!(migrated.settings.mascot, mascot);
            assert!(!migrated.settings.animations);
            assert_eq!(load(&path).unwrap().settings.mascot, mascot);
            let backups: Vec<_> = fs::read_dir(dir.path())
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|p| p != &path)
                .collect();
            assert_eq!(backups.len(), 1);
            assert_eq!(fs::read(&backups[0]).unwrap(), original);
        }
    }

    #[test]
    fn future_and_corrupt_files_are_never_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        for original in [
            r#"{"schema_version":999,"future_data":"keep me"}"#,
            "broken json",
            r#"{"schema_version":2,"timers":[],"settings":{}}"#,
        ] {
            fs::write(&path, original).unwrap();
            assert!(load(&path).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
