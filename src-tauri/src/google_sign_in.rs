use serde::{Deserialize, Serialize};
const TIMEOUT_MS: u64 = 5 * 60 * 1000;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GoogleCredential {
    pub id_token: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoogleCompletion {
    pub challenge: String,
    pub id_token: String,
}
#[derive(Default)]
pub struct GoogleSignIn {
    pending: Option<Pending>,
}
struct Pending {
    challenge: String,
    expires: u64,
    credential: Option<GoogleCredential>,
}
impl GoogleSignIn {
    pub fn begin(&mut self, now: u64) -> String {
        let challenge = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        self.pending = Some(Pending {
            challenge: challenge.clone(),
            expires: now.saturating_add(TIMEOUT_MS),
            credential: None,
        });
        challenge
    }
    fn pending(&mut self, challenge: &str, now: u64) -> Result<&mut Pending, String> {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| now >= pending.expires)
        {
            self.pending = None;
        }
        self.pending
            .as_mut()
            .filter(|pending| pending.challenge == challenge)
            .ok_or(
                "This sign-in request expired or was cancelled. Start again from the app.".into(),
            )
    }
    pub fn complete(&mut self, result: GoogleCompletion, now: u64) -> Result<(), String> {
        let pending = self.pending(&result.challenge, now)?;
        if pending.credential.is_some()
            || result.id_token.is_empty()
            || result.id_token.len() > 16 * 1024
            || result.id_token.chars().any(char::is_control)
        {
            return Err("Invalid or already completed sign-in request.".into());
        }
        pending.credential = Some(GoogleCredential {
            id_token: result.id_token,
        });
        Ok(())
    }
    pub fn take(&mut self, challenge: &str, now: u64) -> Result<Option<GoogleCredential>, String> {
        let credential = self.pending(challenge, now)?.credential.take();
        if credential.is_some() {
            self.pending = None;
        }
        Ok(credential)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handoff_requires_the_challenge_and_is_consumed_once() {
        let mut flow = GoogleSignIn::default();
        let challenge = flow.begin(0);
        assert!(flow.take(&challenge, 1).unwrap().is_none());
        assert!(
            flow.complete(
                GoogleCompletion {
                    challenge: "wrong".into(),
                    id_token: "test-token".into()
                },
                1
            )
            .is_err()
        );
        flow.complete(
            GoogleCompletion {
                challenge: challenge.clone(),
                id_token: "test-token".into(),
            },
            1,
        )
        .unwrap();
        assert_eq!(
            flow.take(&challenge, 2).unwrap().unwrap().id_token,
            "test-token"
        );
        assert!(flow.take(&challenge, 3).is_err());
    }
    #[test]
    fn expiry_and_new_attempts_invalidate_old_requests() {
        let mut flow = GoogleSignIn::default();
        let old = flow.begin(0);
        let current = flow.begin(1);
        assert!(flow.take(&old, 2).is_err());
        assert!(
            flow.complete(
                GoogleCompletion {
                    challenge: current,
                    id_token: "test-token".into()
                },
                TIMEOUT_MS + 1
            )
            .is_err()
        );
    }
}
