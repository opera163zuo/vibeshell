use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairError {
    InvalidCode,
    InvalidName,
}

pub const PAIRING_TTL_SECONDS: i64 = 300;
pub const MAX_PAIRING_ATTEMPTS: u8 = 5;
const TOKEN_PREFIX: &str = "vsh_";
const PAIRING_AGENTS_SETTING: &str = "mcp_lan_agents_v1";

#[derive(Debug, Clone)]
struct PairingCode {
    value: String,
    expires_at: i64,
    attempts: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AgentRecord {
    id: String,
    name: String,
    token_hash: String,
    revoked: bool,
}

fn digest_token(token: &str) -> String {
    use base64::Engine as _;
    use sha2::Digest;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(token.as_bytes()))
}

#[derive(Debug, Clone)]
pub struct IssuedAgent {
    pub id: String,
    pub name: String,
    /// Shown only on pairing; never retained in the registry.
    pub token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub revoked: bool,
}

/// Pairing state. Token hashes can be serialized for protected storage; only the
/// plaintext token returned by a successful pair operation is sent to a client.
pub struct PairingRegistry {
    pending: Option<PairingCode>,
    agents: HashMap<String, AgentRecord>,
    database: Option<std::sync::Arc<crate::storage::Database>>,
}

impl PairingRegistry {
    pub fn new() -> Self {
        Self {
            pending: None,
            agents: HashMap::new(),
            database: None,
        }
    }

    pub fn with_database(database: std::sync::Arc<crate::storage::Database>) -> Self {
        let stored = database.get_setting(PAIRING_AGENTS_SETTING);
        let agents = match stored {
            Ok(Some(value)) => {
                let payload = value
                    .strip_prefix("enc:v1:")
                    .expect("agent registry is plaintext or has an unknown format");
                let crypto = crate::storage::crypto::Crypto::device()
                    .expect("device key unavailable; refusing to discard paired agents");
                let clear = crypto
                    .decrypt_base64(payload)
                    .expect("agent registry could not be decrypted");
                serde_json::from_slice::<HashMap<String, AgentRecord>>(&clear)
                    .expect("agent registry payload is corrupt")
            }
            Ok(None) => HashMap::new(),
            Err(_) => panic!("Could not read persisted paired agents"),
        };
        Self {
            pending: None,
            agents,
            database: Some(database),
        }
    }

    fn persist(&self) -> Result<(), String> {
        if let Some(database) = &self.database {
            let data = serde_json::to_string(&self.agents)
                .map_err(|_| "Could not encode agent registry".to_string())?;
            let encrypted = crate::storage::crypto::Crypto::device()
                .and_then(|crypto| crypto.encrypt_base64(data.as_bytes()))
                .map_err(|_| "Could not encrypt agent registry on this device".to_string())?;
            database
                .set_setting(PAIRING_AGENTS_SETTING, &format!("enc:v1:{encrypted}"))
                .map_err(|_| "Could not persist agent registry".to_string())?;
        }
        Ok(())
    }

    pub fn create_code(&mut self, now: i64) -> String {
        use rand::Rng;
        let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000u32));
        self.pending = Some(PairingCode {
            value: code.clone(),
            expires_at: now + PAIRING_TTL_SECONDS,
            attempts: 0,
        });
        code
    }

    pub fn pair(&mut self, name: &str, code: &str, now: i64) -> Result<IssuedAgent, PairError> {
        let name = name.trim();
        if name.is_empty() || name.len() > 80 {
            return Err(PairError::InvalidName);
        }
        let Some(pending) = self.pending.as_mut() else {
            return Err(PairError::InvalidCode);
        };
        if now >= pending.expires_at || pending.attempts >= MAX_PAIRING_ATTEMPTS {
            self.pending = None;
            return Err(PairError::InvalidCode);
        }
        pending.attempts += 1;
        use subtle::ConstantTimeEq;
        if !bool::from(pending.value.as_bytes().ct_eq(code.as_bytes())) {
            if pending.attempts >= MAX_PAIRING_ATTEMPTS {
                self.pending = None;
            }
            return Err(PairError::InvalidCode);
        }
        self.pending = None;
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut bytes);
        use base64::Engine as _;
        let token = format!(
            "{}{}",
            TOKEN_PREFIX,
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
        );
        let token_hash = digest_token(&token);
        let id = uuid::Uuid::new_v4().to_string();
        self.agents.insert(
            id.clone(),
            AgentRecord {
                id: id.clone(),
                name: name.into(),
                token_hash,
                revoked: false,
            },
        );
        self.persist().map_err(|_| PairError::InvalidCode)?;
        Ok(IssuedAgent {
            id,
            name: name.into(),
            token,
        })
    }

    pub fn authenticate(&self, token: &str) -> bool {
        use subtle::ConstantTimeEq;
        let candidate = digest_token(token);
        self.agents.values().any(|agent| {
            !agent.revoked && bool::from(agent.token_hash.as_bytes().ct_eq(candidate.as_bytes()))
        })
    }

    pub fn revoke(&mut self, id: &str) -> bool {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.revoked = true;
            return self.persist().is_ok();
        }
        false
    }

    pub fn list(&self) -> Vec<AgentSummary> {
        self.agents
            .values()
            .map(|agent| AgentSummary {
                id: agent.id.clone(),
                name: agent.name.clone(),
                revoked: agent.revoked,
            })
            .collect()
    }
}

impl Default for PairingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_code_is_single_use_and_token_is_returned_once() {
        let now = 1_800_000_000;
        let mut registry = PairingRegistry::new();
        let code = registry.create_code(now);
        let issued = registry.pair("hermes-office", &code, now).unwrap();
        assert_eq!(issued.name, "hermes-office");
        assert_eq!(
            registry.pair("another-agent", &code, now).unwrap_err(),
            PairError::InvalidCode
        );
        assert!(registry.authenticate(&issued.token));
    }

    #[test]
    fn pairing_code_expires_and_locks_after_five_failures() {
        let now = 1_800_000_000;
        let mut registry = PairingRegistry::new();
        let code = registry.create_code(now);
        assert_eq!(
            registry
                .pair("late-agent", &code, now + PAIRING_TTL_SECONDS)
                .unwrap_err(),
            PairError::InvalidCode
        );
        let code = registry.create_code(now);
        for _ in 0..MAX_PAIRING_ATTEMPTS {
            assert_eq!(
                registry.pair("bad-agent", "000000", now).unwrap_err(),
                PairError::InvalidCode
            );
        }
        assert_eq!(
            registry.pair("good-agent", &code, now).unwrap_err(),
            PairError::InvalidCode
        );
    }

    #[test]
    fn revoked_agent_token_stops_authenticating() {
        let now = 1_800_000_000;
        let mut registry = PairingRegistry::new();
        let code = registry.create_code(now);
        let issued = registry.pair("revoke-me", &code, now).unwrap();
        assert!(registry.authenticate(&issued.token));
        assert!(registry.revoke(&issued.id));
        assert!(!registry.authenticate(&issued.token));
    }
}
