use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::SystemTime;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{ClientId, IssuerUrl, RedirectUrl, Scope, RefreshToken, OAuth2TokenResponse, TokenResponse};
use crate::config::get_config;
use crate::constants::CLIENT_ID;

const SERVICE_NAME: &str = "com.whitedog.middleauth.ssh-proxy";
const CREDENTIAL_KEY: &str = "session";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenSession {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub expires_at_unix: u64,
}

impl TokenSession {
    pub fn is_expired(&self, buffer: Option<u64>) -> bool {
        let buffer = buffer.unwrap_or(30);

        let current_unix_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        current_unix_timestamp <= (self.expires_at_unix - buffer)
    }

    pub fn refresh(&self) -> Result<TokenSession, Box<dyn Error>> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(TokenSession::refresh_async(&self))
    }

    pub async fn refresh_async(&self) -> Result<TokenSession, Box<dyn Error>> {
        let config = get_config();

        let http_client = openidconnect::reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");

        let provider_metadata = CoreProviderMetadata::discover_async(IssuerUrl::new(format!("{}/", config.api_url.as_str()))?, &http_client).await?;

        let client_id = ClientId::new(config.client_id.unwrap_or(CLIENT_ID.into()));
        let redirect_url = RedirectUrl::new("http://localhost:59080/callback".to_string())?;

        let client = CoreClient::from_provider_metadata(provider_metadata, client_id, None)
            .set_redirect_uri(redirect_url);

        let response = client.exchange_refresh_token(&RefreshToken::new(self.refresh_token.clone()))?
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .request_async(&http_client).await?;

        let expire_time = SystemTime::now() + response.expires_in().expect("Missing expires_in");
        let expire_timestamp = expire_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let session = TokenSession {
            access_token: response.access_token().secret().to_string(),
            id_token: response.id_token().expect("Missing id_token").to_string(),
            refresh_token: response.refresh_token().expect("Missing refresh token").secret().to_string(),
            expires_at_unix: expire_timestamp
        };

        TokenStorage::save_session(&session)?;

        Ok(session)
    }
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
