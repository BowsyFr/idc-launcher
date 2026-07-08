use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, TokenResponse,
};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
}

pub struct DiscordAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    csrf_token: Arc<Mutex<Option<CsrfToken>>>,
}

impl DiscordAuth {
    pub fn new() -> Self {
        dotenv::dotenv().ok();

        Self {
            client_id: env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID must be set"),
            client_secret: env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET must be set"),
            redirect_uri: env::var("DISCORD_REDIRECT_URI").expect("DISCORD_REDIRECT_URI must be set"),
            pkce_verifier: Arc::new(Mutex::new(None)),
            csrf_token: Arc::new(Mutex::new(None)),
        }
    }

    fn build_client(&self) -> oauth2::basic::BasicClient {
        oauth2::basic::BasicClient::new(
            oauth2::ClientId::new(self.client_id.clone()),
            Some(oauth2::ClientSecret::new(self.client_secret.clone())),
            oauth2::AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(),
            Some(oauth2::TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_uri.clone()).unwrap())
    }

    pub async fn get_auth_url(&self) -> (String, String) {
        let client = self.build_client();

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let csrf_token = CsrfToken::new_random();

        *self.pkce_verifier.lock().await = Some(pkce_verifier);
        *self.csrf_token.lock().await = Some(csrf_token.clone());

        let (auth_url, _csrf_token) = client
            .authorize_url(|| csrf_token.clone())
            .add_scope(oauth2::Scope::new("identify".to_string()))
            .add_scope(oauth2::Scope::new("email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let csrf_secret = csrf_token.secret().as_str().to_string();
        (auth_url.to_string(), csrf_secret)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<String, String> {
        let client = self.build_client();

        let pkce_verifier = self.pkce_verifier.lock().await;
        let verifier = pkce_verifier.as_ref().ok_or_else(|| "PKCE verifier not set".to_string())?;
        let verifier_secret = verifier.secret().clone();

        let token = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier_secret))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| format!("Token exchange failed: {}", e))?;

        Ok(token.access_token().secret().clone())
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<DiscordUser, String> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://discord.com/api/users/@me")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get user info: {}", response.status()));
        }

        let user_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(DiscordUser {
            id: user_data["id"].as_str().unwrap_or("").to_string(),
            username: user_data["username"].as_str().unwrap_or("").to_string(),
            discriminator: user_data["discriminator"].as_str().unwrap_or("").to_string(),
            avatar: user_data["avatar"].as_str().map(|s| s.to_string()),
        })
    }
}
