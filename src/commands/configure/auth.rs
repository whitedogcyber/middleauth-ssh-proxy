use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use axum::extract::Query;
use axum::response::Html;
use axum::Router;
use axum::routing::get;
use clap::ArgMatches;
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::{AuthenticationFlow, AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse};
use tokio::sync::oneshot;
use openidconnect::reqwest;
use crate::config::get_config;
use crate::constants::CLIENT_ID;
use crate::token_storage::{TokenSession, TokenStorage};

#[derive(serde::Deserialize)]
struct AuthCallback {
    code: String,
    state: String
}

struct AppState {
    expected_state: String,
    tx: std::sync::Mutex<Option<oneshot::Sender<AuthCallback>>>
}

pub fn auth(args: &ArgMatches) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(auth_async(args)).expect("Failed to run auth");
}

async fn auth_async(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config();

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let provider_metadata = CoreProviderMetadata::discover_async(IssuerUrl::new(format!("{}/", config.api_url.as_str()))?, &http_client).await?;

    let client_id = ClientId::new(config.client_id.unwrap_or(CLIENT_ID.into()));
    let redirect_url = RedirectUrl::new("http://localhost:59080/callback".to_string())?;

    let client = CoreClient::from_provider_metadata(provider_metadata, client_id, None)
        .set_redirect_uri(redirect_url);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let (tx, rx) = oneshot::channel::<AuthCallback>();
    let shared_state = Arc::new(AppState {
        expected_state: csrf_token.secret().to_string(),
        tx: std::sync::Mutex::new(Some(tx))
    });

    let app = Router::new()
        .route("/callback", get(handle_callback))
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 59080));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let server_task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    println!("Opening your browser to authenticate...");
    println!("Or Visit: {}", auth_url.as_str());

    open::that(auth_url.as_str())?;

    tokio::select! {
        auth_callback = rx => {
            match auth_callback {
                Ok(callback) => {
                    let token_response = client
                        .exchange_code(AuthorizationCode::new(callback.code))?
                        .set_pkce_verifier(pkce_verifier)
                        .request_async(&http_client)
                        .await?;

                    let expire_time = SystemTime::now() + token_response.expires_in().expect("Missing expires_in");
                    let expire_timestamp = expire_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs();

                    let session = TokenSession {
                        access_token: token_response.access_token().secret().to_string(),
                        id_token: token_response.id_token().expect("Missing id_token").to_string(),
                        refresh_token: token_response.refresh_token().expect("Missing refresh token").secret().to_string(),
                        expires_at_unix: expire_timestamp
                    };

                    TokenStorage::save_session(&session)?;

                    println!("Authenticated successfully!");
                }
                Err(_) => println!("Authentication failed.")
            }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
            println!("Authentication timed out after 5 minutes.");
        }
    }

    server_task.abort();
    Ok(())
}

async fn handle_callback(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(params): Query<AuthCallback>,
) -> Html<&'static str> {
    // Validate CSRF state parity
    if params.state != state.expected_state {
        return Html("<h3>Authentication Failed: State Mismatch Security Alert.</h3>");
    }

    // Extract the sender and signal completion back to main thread
    if let Some(tx) = state.tx.lock().unwrap().take() {
        let _ = tx.send(params);
    }
    Html("<h3>Authentication complete! You can close this tab and return to the terminal.</h3>")

}