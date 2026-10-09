use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::error::Error;

const SERVICE_NAME: &str = "com.whitedog.middleauth.ssh-proxy";
const CREDENTIAL_KEY: &str = "session";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenSession {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at_unix: Option<i64>,
}

pub struct TokenStorage;

impl TokenStorage {
    /// Encrypts and writes the token session into the native OS vault
    pub fn save_session(session: &TokenSession) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, CREDENTIAL_KEY)?;
        let json_data = serde_json::to_string(session)?;
        entry.set_password(&json_data)?;
        Ok(())
    }

    /// Fetches and parses the unified session payload from the native OS vault
    pub fn get_session() -> Result<Option<TokenSession>, Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, CREDENTIAL_KEY)?;
        match entry.get_password() {
            Ok(json_data) => {
                let session: TokenSession = serde_json::from_str(&json_data)?;
                Ok(Some(session))
            }
            Err(keyring::Error::NoEntry) => Ok(None), // Return None gracefully if key isn't stored yet
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Completely wipes the session credentials from the native OS vault
    pub fn clear_session() -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, CREDENTIAL_KEY)?;
        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already cleared, safety check passed
            Err(e) => Err(Box::new(e)),
        }
    }
}
