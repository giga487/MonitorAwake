use crate::{
    core::{Action, Model, TimerStatus},
    notes::{MAX_NOTES, Note, NoteBounds, NoteColor},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotesStorage {
    #[default]
    Local,
    Firebase,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudScope {
    pub project_id: String,
    pub uid: String,
}
impl CloudScope {
    pub fn validate(&self) -> Result<(), String> {
        if self.project_id.is_empty()
            || self.project_id.len() > 128
            || self.uid.is_empty()
            || self.uid.len() > 128
        {
            return Err("Invalid cloud account.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudNote {
    pub schema_version: u32,
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub color: NoteColor,
    pub owner_uid: String,
    pub member_uids: Vec<String>,
    pub revision: u64,
    pub deleted: bool,
}
impl CloudNote {
    pub fn validate(&self) -> Result<(), String> {
        crate::notes::validate_text(&self.title, &self.body)?;
        if self.schema_version != 1
            || self.revision == 0
            || self.revision > 9_007_199_254_740_991
            || self.owner_uid.is_empty()
            || self.owner_uid.len() > 128
            || self.member_uids.is_empty()
            || self.member_uids.len() > 20
            || !self.member_uids.contains(&self.owner_uid)
            || self
                .member_uids
                .iter()
                .any(|uid| uid.is_empty() || uid.len() > 128)
            || self
                .member_uids
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != self.member_uids.len()
        {
            return Err("Invalid or unsupported shared note document.".into());
        }
        Ok(())
    }
    pub fn same_content(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.body == other.body
            && self.color == other.color
            && self.owner_uid == other.owner_uid
            && self.member_uids == other.member_uids
            && self.deleted == other.deleted
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudEntry {
    pub id: Uuid,
    pub base: CloudNote,
    pub member_uids: Vec<String>,
    pub conflict: Option<CloudNote>,
    pub access_removed: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteSync {
    pub scope: Option<CloudScope>,
    pub entries: Vec<CloudEntry>,
}
#[derive(Serialize)]
pub struct CloudOperation {
    pub document: CloudNote,
    pub expected_revision: Option<u64>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncStatus {
    pub message: String,
    pub pending: usize,
}

impl Model {
    pub fn validate_note_sync(&self) -> Result<(), String> {
        if self.note_sync.scope.is_none() && !self.note_sync.entries.is_empty() {
            return Err("Cloud cache requires an account scope.".into());
        }
        if let Some(scope) = &self.note_sync.scope {
            scope.validate()?;
        }
        if self.note_sync.entries.len() > 200 {
            return Err("Too many cached cloud note entries.".into());
        }
        let mut ids = std::collections::HashSet::new();
        for entry in &self.note_sync.entries {
            entry.base.validate()?;
            if entry.id != entry.base.id || !ids.insert(entry.id) {
                return Err("Invalid cloud note cache identifiers.".into());
            }
            let mut desired = entry.base.clone();
            desired.member_uids = entry.member_uids.clone();
            desired.validate()?;
            if let Some(conflict) = &entry.conflict {
                conflict.validate()?;
                if conflict.id != entry.id {
                    return Err("Invalid note conflict.".into());
                }
            }
        }
        Ok(())
    }
    fn check_scope(&self, scope: &CloudScope) -> Result<(), String> {
        if self.settings.notes_storage != NotesStorage::Firebase
            || self.note_sync.scope.as_ref() != Some(scope)
        {
            return Err("Cloud account or storage mode changed. Synchronization stopped.".into());
        }
        Ok(())
    }
    pub fn cloud_operations(&self, scope: &CloudScope) -> Result<Vec<CloudOperation>, String> {
        self.check_scope(scope)?;
        let mut result = vec![];
        for note in &self.notes {
            let entry = self
                .note_sync
                .entries
                .iter()
                .find(|entry| entry.id == note.id);
            if entry.is_some_and(|entry| entry.conflict.is_some() || entry.access_removed) {
                continue;
            }
            let base = entry.map(|entry| &entry.base);
            let document = CloudNote {
                schema_version: 1,
                id: note.id,
                title: note.title.clone(),
                body: note.body.clone(),
                color: note.color,
                owner_uid: base.map_or_else(|| scope.uid.clone(), |base| base.owner_uid.clone()),
                member_uids: entry.map_or_else(
                    || vec![scope.uid.clone()],
                    |entry| entry.member_uids.clone(),
                ),
                revision: base.map_or(1, |base| base.revision.saturating_add(1)),
                deleted: false,
            };
            if base.is_none_or(|base| !base.same_content(&document)) {
                result.push(CloudOperation {
                    document,
                    expected_revision: base.map(|base| base.revision),
                });
            }
        }
        for entry in &self.note_sync.entries {
            if !self.notes.iter().any(|note| note.id == entry.id)
                && !entry.base.deleted
                && entry.conflict.is_none()
                && !entry.access_removed
            {
                let mut document = entry.base.clone();
                document.deleted = true;
                document.revision += 1;
                result.push(CloudOperation {
                    expected_revision: Some(entry.base.revision),
                    document,
                });
            }
        }
        Ok(result)
    }
    fn remove_cloud_note(&mut self, id: Uuid) {
        if let Some(note) = self.notes.iter().find(|note| note.id == id) {
            let linked = note.reminder_id;
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
        }
        self.notes.retain(|note| note.id != id);
    }
    fn use_cloud_content(&mut self, document: &CloudNote) -> Result<(), String> {
        if document.deleted {
            self.remove_cloud_note(document.id);
            return Ok(());
        }
        if let Some(note) = self.notes.iter_mut().find(|note| note.id == document.id) {
            if note.title != document.title || note.body != document.body {
                note.content_revision = note
                    .content_revision
                    .checked_add(1)
                    .ok_or("Note revision overflow")?;
            }
            note.title = document.title.clone();
            note.body = document.body.clone();
            note.color = document.color;
        } else {
            if self.notes.len() >= MAX_NOTES {
                return Err(
                    "Cloud note limit reached. Keep at most 100 notes on this device.".into(),
                );
            }
            self.notes.push(Note {
                id: document.id,
                title: document.title.clone(),
                body: document.body.clone(),
                color: document.color,
                visible: false,
                pinned: false,
                bounds: NoteBounds::default(),
                content_revision: 0,
                reminder_id: None,
            });
        }
        Ok(())
    }
    fn acknowledge_cloud(&mut self, document: &CloudNote) {
        self.note_sync
            .entries
            .retain(|entry| entry.id != document.id);
        if !document.deleted || self.notes.iter().any(|note| note.id == document.id) {
            self.note_sync.entries.push(CloudEntry {
                id: document.id,
                base: document.clone(),
                member_uids: document.member_uids.clone(),
                conflict: None,
                access_removed: false,
            });
        }
    }
    pub fn apply_note_sync(&mut self, action: &Action) -> Result<String, String> {
        match action {
            Action::SetNotesStorage { storage } => {
                self.settings.notes_storage = *storage;
                Ok("Notes storage updated.".into())
            }
            Action::BindNotesCloud { scope } => {
                scope.validate()?;
                if self.settings.notes_storage != NotesStorage::Firebase {
                    return Err("Enable Firebase notes first.".into());
                }
                if self
                    .note_sync
                    .scope
                    .as_ref()
                    .is_some_and(|current| current != scope)
                {
                    return Err("These cached notes belong to another Firebase account. Disconnect cloud links before copying them to a new account.".into());
                }
                self.note_sync.scope = Some(scope.clone());
                Ok("Cloud account connected.".into())
            }
            Action::DisconnectNotesCloud => {
                if self.notes.iter().any(|note| note.visible) {
                    return Err(
                        "Save and close note windows before disconnecting cloud links.".into(),
                    );
                }
                for note in &mut self.notes {
                    note.id = Uuid::new_v4();
                }
                self.note_sync = NoteSync::default();
                self.settings.notes_storage = NotesStorage::Local;
                Ok(
                    "Cloud links disconnected. Local copies kept; cloud originals unchanged."
                        .into(),
                )
            }
            Action::AcknowledgeCloudNote { scope, document } => {
                self.check_scope(scope)?;
                document.validate()?;
                if !document.member_uids.contains(&scope.uid) {
                    return Err("This account cannot access the note.".into());
                }
                if self
                    .note_sync
                    .entries
                    .iter()
                    .any(|entry| entry.id == document.id && entry.base.revision > document.revision)
                {
                    return Ok("Older cloud acknowledgement ignored.".into());
                }
                // Preserve local edits made while the network write was in flight.
                let pending_members = self
                    .note_sync
                    .entries
                    .iter()
                    .find(|entry| entry.id == document.id)
                    .filter(|entry| entry.member_uids != entry.base.member_uids)
                    .map(|entry| entry.member_uids.clone());
                self.acknowledge_cloud(document);
                if let Some(members) = pending_members
                    && members != document.member_uids
                    && let Some(entry) = self
                        .note_sync
                        .entries
                        .iter_mut()
                        .find(|entry| entry.id == document.id)
                {
                    entry.member_uids = members;
                }
                Ok("Cloud note acknowledged.".into())
            }
            Action::ApplyCloudNote { scope, document } => {
                self.check_scope(scope)?;
                document.validate()?;
                if !document.member_uids.contains(&scope.uid) {
                    return Err("This account cannot access the note.".into());
                }
                if self
                    .cloud_operations(scope)?
                    .iter()
                    .any(|operation| operation.document.same_content(document))
                {
                    self.acknowledge_cloud(document);
                    return Ok("Previously committed upload recovered.".into());
                }
                if let Some(entry) = self
                    .note_sync
                    .entries
                    .iter()
                    .find(|entry| entry.id == document.id)
                {
                    if document.revision <= entry.base.revision {
                        return Ok("Cloud note unchanged.".into());
                    }
                    let dirty = self
                        .cloud_operations(scope)?
                        .iter()
                        .any(|operation| operation.document.id == document.id);
                    if dirty || entry.conflict.is_some() {
                        self.note_sync
                            .entries
                            .iter_mut()
                            .find(|entry| entry.id == document.id)
                            .unwrap()
                            .conflict = Some(document.clone());
                        return Ok("Cloud conflict preserved for review.".into());
                    }
                } else if self.notes.iter().any(|note| note.id == document.id) {
                    // Recover an upload committed before a crash or preserve concurrent local edits.
                    let desired = self
                        .cloud_operations(scope)?
                        .into_iter()
                        .find(|op| op.document.id == document.id)
                        .map(|op| op.document);
                    if desired
                        .as_ref()
                        .is_some_and(|desired| !desired.same_content(document))
                    {
                        self.note_sync.entries.push(CloudEntry {
                            id: document.id,
                            base: document.clone(),
                            member_uids: document.member_uids.clone(),
                            conflict: Some(document.clone()),
                            access_removed: false,
                        });
                        return Ok("Cloud conflict preserved for review.".into());
                    }
                }
                self.use_cloud_content(document)?;
                self.acknowledge_cloud(document);
                Ok("Cloud note cached.".into())
            }
            Action::ResolveNoteConflict { id, keep_local } => {
                let document = self
                    .note_sync
                    .entries
                    .iter()
                    .find(|entry| entry.id == *id)
                    .and_then(|entry| entry.conflict.clone())
                    .ok_or("No conflict to resolve.")?;
                let members = self
                    .note_sync
                    .entries
                    .iter()
                    .find(|entry| entry.id == *id)
                    .unwrap()
                    .member_uids
                    .clone();
                if !keep_local {
                    self.use_cloud_content(&document)?;
                }
                self.acknowledge_cloud(&document);
                if *keep_local
                    && let Some(entry) = self
                        .note_sync
                        .entries
                        .iter_mut()
                        .find(|entry| entry.id == *id)
                {
                    entry.member_uids = members;
                }
                Ok("Conflict resolved.".into())
            }
            Action::SetNoteMembers { id, member_uids } => {
                let scope = self
                    .note_sync
                    .scope
                    .as_ref()
                    .ok_or("Sign in and sync the note first.")?;
                let entry = self
                    .note_sync
                    .entries
                    .iter_mut()
                    .find(|entry| entry.id == *id)
                    .ok_or("Wait for the first upload before sharing.")?;
                if entry.base.owner_uid != scope.uid || entry.access_removed {
                    return Err("Only the owner can change sharing permissions.".into());
                }
                let mut desired = entry.base.clone();
                desired.member_uids = member_uids.clone();
                desired.validate()?;
                entry.member_uids = member_uids.clone();
                Ok("Sharing change queued.".into())
            }
            Action::CloudNoteAccessRemoved { scope, id } => {
                self.check_scope(scope)?;
                if let Some(entry) = self
                    .note_sync
                    .entries
                    .iter_mut()
                    .find(|entry| entry.id == *id)
                {
                    entry.access_removed = true;
                    entry.conflict = None;
                }
                Ok("Cloud access removed. Cached local content kept.".into())
            }
            _ => Err("Not a note synchronization action.".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Model, CloudScope, Uuid) {
        let mut model = Model::default();
        model.apply(&Action::CreateNote, 0).unwrap();
        let id = model.notes[0].id;
        let scope = CloudScope {
            project_id: "demo-notes".into(),
            uid: "owner".into(),
        };
        model
            .apply(
                &Action::SetNotesStorage {
                    storage: NotesStorage::Firebase,
                },
                0,
            )
            .unwrap();
        model
            .apply(
                &Action::BindNotesCloud {
                    scope: scope.clone(),
                },
                0,
            )
            .unwrap();
        (model, scope, id)
    }
    fn ack(model: &mut Model, scope: &CloudScope) -> CloudNote {
        let document = model.cloud_operations(scope).unwrap().remove(0).document;
        model
            .apply(
                &Action::AcknowledgeCloudNote {
                    scope: scope.clone(),
                    document: document.clone(),
                },
                0,
            )
            .unwrap();
        document
    }
    fn edit(model: &mut Model, id: Uuid, body: &str) {
        let revision = model
            .notes
            .iter()
            .find(|note| note.id == id)
            .unwrap()
            .content_revision;
        model
            .apply(
                &Action::UpdateNote {
                    id,
                    expected_revision: revision,
                    title: "Note".into(),
                    body: body.into(),
                },
                0,
            )
            .unwrap();
    }
    #[test]
    fn offline_edits_survive_reload_and_ack_does_not_lose_newer_edits() {
        let (mut model, scope, id) = setup();
        let original = ack(&mut model, &scope);
        edit(&mut model, id, "Offline edit");
        let sent = model.cloud_operations(&scope).unwrap().remove(0).document;
        edit(&mut model, id, "More typing while upload runs");
        model
            .apply(
                &Action::AcknowledgeCloudNote {
                    scope: scope.clone(),
                    document: sent,
                },
                0,
            )
            .unwrap();
        let restored: Model =
            serde_json::from_str(&serde_json::to_string(&model).unwrap()).unwrap();
        let operation = restored.cloud_operations(&scope).unwrap().remove(0);
        assert_eq!(operation.expected_revision, Some(original.revision + 1));
        assert_eq!(operation.document.body, "More typing while upload runs");
        assert_eq!(operation.document.revision, 3);
    }
    #[test]
    fn a_committed_upload_recovers_after_restart_without_a_false_conflict() {
        let (mut model, scope, id) = setup();
        ack(&mut model, &scope);
        edit(&mut model, id, "Already uploaded");
        let document = model.cloud_operations(&scope).unwrap().remove(0).document;
        let mut restored: Model =
            serde_json::from_str(&serde_json::to_string(&model).unwrap()).unwrap();
        restored
            .apply(
                &Action::ApplyCloudNote {
                    scope: scope.clone(),
                    document,
                },
                0,
            )
            .unwrap();
        assert!(restored.note_sync.entries[0].conflict.is_none());
        assert!(restored.cloud_operations(&scope).unwrap().is_empty());
    }
    #[test]
    fn remote_changes_preserve_device_geometry_pinning_visibility_and_reminders() {
        let (mut model, scope, id) = setup();
        let mut remote = ack(&mut model, &scope);
        model
            .apply(
                &Action::SetNoteAppearance {
                    id,
                    color: NoteColor::Yellow,
                    pinned: true,
                },
                0,
            )
            .unwrap();
        model
            .apply(&Action::SetNoteReminder { id, seconds: 60 }, 0)
            .unwrap();
        let before = model.notes[0].clone();
        assert!(model.cloud_operations(&scope).unwrap().is_empty());
        remote.revision += 1;
        remote.body = "Remote text".into();
        remote.color = NoteColor::Blue;
        model
            .apply(
                &Action::ApplyCloudNote {
                    scope: scope.clone(),
                    document: remote,
                },
                0,
            )
            .unwrap();
        assert_eq!(model.notes[0].body, "Remote text");
        assert_eq!(model.notes[0].bounds, before.bounds);
        assert_eq!(model.notes[0].pinned, before.pinned);
        assert_eq!(model.notes[0].visible, before.visible);
        assert_eq!(model.notes[0].reminder_id, before.reminder_id);
        assert!(model.cloud_operations(&scope).unwrap().is_empty());
        assert!(
            model
                .apply(
                    &Action::UpdateNote {
                        id,
                        expected_revision: before.content_revision,
                        title: "Stale".into(),
                        body: "Would overwrite".into()
                    },
                    0
                )
                .is_err()
        );
    }
    #[test]
    fn concurrent_edits_preserve_both_versions_and_support_explicit_resolution() {
        for keep_local in [true, false] {
            let (mut model, scope, id) = setup();
            let mut remote = ack(&mut model, &scope);
            edit(&mut model, id, "My edit");
            remote.revision += 1;
            remote.body = "Their edit".into();
            model
                .apply(
                    &Action::ApplyCloudNote {
                        scope: scope.clone(),
                        document: remote,
                    },
                    0,
                )
                .unwrap();
            assert_eq!(model.notes[0].body, "My edit");
            assert_eq!(
                model.note_sync.entries[0].conflict.as_ref().unwrap().body,
                "Their edit"
            );
            assert!(model.cloud_operations(&scope).unwrap().is_empty());
            model
                .apply(&Action::ResolveNoteConflict { id, keep_local }, 0)
                .unwrap();
            assert_eq!(
                model.notes[0].body,
                if keep_local { "My edit" } else { "Their edit" }
            );
            let operations = model.cloud_operations(&scope).unwrap();
            assert_eq!(operations.len(), usize::from(keep_local));
            if keep_local {
                assert_eq!(operations[0].expected_revision, Some(2));
            }
        }
    }
    #[test]
    fn deletion_is_queued_and_remote_deletion_releases_local_reminder() {
        let (mut model, scope, id) = setup();
        let mut remote = ack(&mut model, &scope);
        model
            .apply(&Action::SetNoteReminder { id, seconds: 60 }, 0)
            .unwrap();
        remote.revision += 1;
        remote.deleted = true;
        model
            .apply(
                &Action::ApplyCloudNote {
                    scope: scope.clone(),
                    document: remote,
                },
                0,
            )
            .unwrap();
        assert!(model.notes.is_empty());
        assert_eq!(model.timers[0].status, TimerStatus::Cancelled);
        assert!(model.cloud_operations(&scope).unwrap().is_empty());
        model.apply(&Action::CreateNote, 0).unwrap();
        let id = model.notes[0].id;
        ack(&mut model, &scope);
        model.apply(&Action::DeleteNote { id }, 0).unwrap();
        let operation = model.cloud_operations(&scope).unwrap().remove(0);
        assert!(operation.document.deleted);
        assert_eq!(operation.expected_revision, Some(1));
        model
            .apply(
                &Action::AcknowledgeCloudNote {
                    scope: scope.clone(),
                    document: operation.document,
                },
                0,
            )
            .unwrap();
        assert!(model.cloud_operations(&scope).unwrap().is_empty());
        model.validate().unwrap();
    }
    #[test]
    fn sharing_is_owner_only_and_account_switch_never_uploads_foreign_cache() {
        let (mut model, scope, id) = setup();
        ack(&mut model, &scope);
        model
            .apply(
                &Action::SetNoteMembers {
                    id,
                    member_uids: vec![scope.uid.clone(), "editor".into()],
                },
                0,
            )
            .unwrap();
        let shared = ack(&mut model, &scope);
        assert_eq!(shared.member_uids.len(), 2);
        model
            .apply(
                &Action::CloudNoteAccessRemoved {
                    scope: scope.clone(),
                    id,
                },
                0,
            )
            .unwrap();
        edit(&mut model, id, "Kept locally");
        assert!(model.cloud_operations(&scope).unwrap().is_empty());
        let other = CloudScope {
            project_id: scope.project_id.clone(),
            uid: "different".into(),
        };
        assert!(
            model
                .apply(&Action::BindNotesCloud { scope: other }, 0)
                .is_err()
        );
        model
            .apply(
                &Action::SetNotesStorage {
                    storage: NotesStorage::Local,
                },
                0,
            )
            .unwrap();
        assert!(model.cloud_operations(&scope).is_err());
        model
            .apply(&Action::SetNoteVisible { id, visible: false }, 0)
            .unwrap();
        model.apply(&Action::DisconnectNotesCloud, 0).unwrap();
        assert_ne!(model.notes[0].id, id);
        assert_eq!(model.notes[0].body, "Kept locally");
        assert!(model.note_sync.scope.is_none());
    }
}
