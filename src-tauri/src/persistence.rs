use crate::core::{Model, SCHEMA_VERSION, Settings, Timer};
use serde::Deserialize;
use std::{fs, io::Write, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyV1 {
    schema_version: u32,
    timers: Vec<Timer>,
    settings: Settings,
}

pub fn save(path: &Path, model: &Model) -> Result<(), String> {
    model.validate()?;
    let parent = path.parent().ok_or("Invalid data directory")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("Saving: {e}"))?;
    serde_json::to_writer_pretty(&mut temporary, model).map_err(|e| format!("Saving: {e}"))?;
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
            Model { schema_version: SCHEMA_VERSION, timers: legacy.timers, settings: legacy.settings, notes: vec![] }
        }
        v if v == u64::from(SCHEMA_VERSION) => serde_json::from_value(value).map_err(|e| format!("Invalid data file, preserved: {e}"))?,
        _ => return Err("Unsupported data schema. Use a newer compatible version of MonitorAwake. The original file has been preserved.".into()),
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
