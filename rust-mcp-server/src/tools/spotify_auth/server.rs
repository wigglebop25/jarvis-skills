use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use base64::Engine;

#[derive(Deserialize)]
struct AuthCode {
    code: Option<String>,
    error: Option<String>,
}

pub async fn start_callback_server(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
) -> Result<(), String> {
    let auth_state = Arc::new(Mutex::new(None::<String>));

    let auth_state_clone = auth_state.clone();
    let callback_handler = move |Query(auth_code): Query<AuthCode>| {
        let client_id = client_id.clone();
        let client_secret = client_secret.clone();
        let redirect_uri = redirect_uri.clone();
        let auth_state = auth_state_clone.clone();

        async move {
            if let Some(error) = auth_code.error {
                return Html(format!(
                    "<html><body><h1>Authorization Failed</h1><p>{}</p></body></html>",
                    error
                ))
                .into_response();
            }

            if let Some(code) = auth_code.code {
                let client = reqwest::Client::new();
                let auth = base64::engine::general_purpose::STANDARD
                    .encode(format!("{}:{}", client_id, client_secret));

                match client
                    .post("https://accounts.spotify.com/api/token")
                    .header("Authorization", format!("Basic {}", auth))
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .form(&[
                        ("grant_type", "authorization_code"),
                        ("code", &code),
                        ("redirect_uri", &redirect_uri),
                    ])
                    .send()
                    .await
                {
                    Ok(response) => match response.json::<Value>().await {
                        Ok(data) => {
                            if let Some(access_token) = data["access_token"].as_str() {
                                let refresh_token = data["refresh_token"].as_str();
                                let expires_in = data["expires_in"].as_i64().unwrap_or(3600);

                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64;

                                let mut cache_json = json!({
                                    "access_token": access_token,
                                    "expires_at": now + expires_in,
                                });

                                if let Some(refresh_token) = refresh_token {
                                    cache_json["refresh_token"] = json!(refresh_token);
                                }

                                let _ = std::fs::write(
                                    ".cache",
                                    serde_json::to_string_pretty(&cache_json).unwrap_or_default(),
                                );

                                let mut state = auth_state.lock().await;
                                *state = Some(access_token.to_string());

                                return Html(
                                    "<html><body><h1>✅ Authorization Successful!</h1><p>You can now close this window and use Spotify commands.</p></body></html>"
                                )
                                .into_response();
                            }
                        }
                        Err(e) => {
                            return Html(format!(
                                "<html><body><h1>Error</h1><p>Failed to parse token response: {}</p></body></html>",
                                e
                            ))
                            .into_response();
                        }
                    },
                    Err(e) => {
                        return Html(format!(
                            "<html><body><h1>Error</h1><p>Token exchange failed: {}</p></body></html>",
                            e
                        ))
                        .into_response();
                    }
                }
            }

            Html("<html><body><h1>Error</h1><p>No authorization code received</p></body></html>")
                .into_response()
        }
    };

    let app = Router::new().route("/callback", get(callback_handler));

    let addr: SocketAddr = "127.0.0.1:8888"
        .parse()
        .map_err(|e| format!("Failed to parse address: {}", e))?;

    tokio::spawn(async move {
        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            let _ = axum::serve(listener, app).await;
        }
    });

    Ok(())
}
