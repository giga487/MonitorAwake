use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirebaseConfig {
    pub schema_version: u32,
    pub project_id: String,
    pub api_key: String,
    pub auth_domain: String,
    pub app_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_bucket: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messaging_sender_id: Option<String>,
}

impl FirebaseConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(
                "Unsupported Firebase configuration version. The original file has been preserved."
                    .into(),
            );
        }
        if self.auth_domain.is_empty()
            || self.auth_domain.len() > 253
            || !self
                .auth_domain
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        {
            return Err("Auth domain must be a hostname without a protocol or path.".into());
        }
        for (name, value) in [
            ("Project ID", &self.project_id),
            ("API key", &self.api_key),
            ("Auth domain", &self.auth_domain),
            ("App ID", &self.app_id),
        ] {
            if value.trim().is_empty() || value.len() > 1024 || value.chars().any(char::is_control)
            {
                return Err(format!("Enter a valid {name}."));
            }
        }
        for value in [&self.storage_bucket, &self.messaging_sender_id]
            .into_iter()
            .flatten()
        {
            if value.len() > 1024 || value.chars().any(char::is_control) {
                return Err("Invalid optional Firebase configuration field.".into());
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct FirebaseSetup {
    pub config: Option<FirebaseConfig>,
    pub auth_emulator_url: Option<String>,
    pub firestore_emulator_url: Option<String>,
}

pub fn emulator_url(value: &str) -> Result<String, String> {
    let port = value
        .strip_prefix("http://127.0.0.1:")
        .and_then(|port| port.parse::<u16>().ok())
        .filter(|port| *port > 0)
        .ok_or("The Auth emulator must use http://127.0.0.1:<port>.")?;
    Ok(format!("http://127.0.0.1:{port}"))
}

pub fn load(path: &Path) -> Result<Option<FirebaseConfig>, String> {
    if !path.exists() {
        return Ok(None);
    }
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 {
        return Err("Firebase configuration is too large. The file has been preserved.".into());
    }
    let bytes = fs::read(path).map_err(|_| "Cannot read the local Firebase configuration.")?;
    let config: FirebaseConfig = serde_json::from_slice(&bytes).map_err(
        |_| "Invalid or incomplete Firebase configuration. The original file has been preserved.",
    )?;
    config.validate()?;
    Ok(Some(config))
}

pub fn save(path: &Path, config: &FirebaseConfig) -> Result<(), String> {
    config.validate()?;
    let previous = load(path)?;
    if previous.as_ref() == Some(config) {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or("Invalid Firebase configuration directory.")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut temp, config).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    if previous.is_some() {
        let mut backup = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        backup
            .write_all(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        backup.as_file().sync_all().map_err(|e| e.to_string())?;
        backup
            .persist_noclobber(parent.join(format!(
                "firebase.local.json.backup-v1-{}.json",
                uuid::Uuid::new_v4()
            )))
            .map_err(|e| e.to_string())?;
    }
    temp.persist(path)
        .map_err(|e| format!("Cannot save Firebase configuration: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> FirebaseConfig {
        FirebaseConfig {
            schema_version: 1,
            project_id: "demo-notes".into(),
            api_key: "test-key".into(),
            auth_domain: "demo-notes.firebaseapp.com".into(),
            app_id: "test-app".into(),
            storage_bucket: None,
            messaging_sender_id: None,
        }
    }
    #[test]
    fn config_is_separate_and_changes_have_exact_backups() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("firebase.local.json");
        assert!(load(&path).unwrap().is_none());
        let mut value = config();
        save(&path, &value).unwrap();
        let original = fs::read(&path).unwrap();
        save(&path, &value).unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        value.project_id = "demo-other".into();
        save(&path, &value).unwrap();
        assert_eq!(load(&path).unwrap(), Some(value));
        let backup = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p != &path)
            .unwrap();
        assert_eq!(fs::read(backup).unwrap(), original);
    }
    #[test]
    fn invalid_files_and_future_versions_are_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("firebase.local.json");
        for original in ["broken", r#"{"schema_version":99}"#] {
            fs::write(&path, original).unwrap();
            assert!(save(&path, &config()).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
        }
        let mut invalid = config();
        invalid.api_key.clear();
        assert!(invalid.validate().is_err());
        for url in [
            "http://remote.example:9099",
            "https://127.0.0.1:9099",
            "http://127.0.0.1:0",
            "http://127.0.0.1:9099/path",
        ] {
            assert!(emulator_url(url).is_err());
        }
        assert_eq!(
            emulator_url("http://127.0.0.1:9099").unwrap(),
            "http://127.0.0.1:9099"
        );
    }
}
