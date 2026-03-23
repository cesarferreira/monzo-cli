use anyhow::{Context, Result};
use url::Url;

use crate::config::Config;
use crate::models::TokenResponse;

const AUTH_URL: &str = "https://auth.monzo.com";
const TOKEN_URL: &str = "https://api.monzo.com/oauth2/token";
const REDIRECT_PORT: u16 = 6789;

/// Run the full OAuth2 authorization code flow:
/// 1. Start local HTTP server on localhost:6789
/// 2. Open browser to Monzo authorization page
/// 3. Receive callback with auth code
/// 4. Exchange code for tokens
/// 5. Save tokens to config
pub async fn login_flow(client_id: &str, client_secret: &str) -> Result<()> {
    let redirect_uri = format!("http://localhost:{REDIRECT_PORT}/callback");
    let state = uuid::Uuid::new_v4().to_string();

    let auth_url = format!(
        "{AUTH_URL}/?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&state={state}"
    );

    println!("Opening browser for Monzo authorization...");
    println!("If the browser doesn't open, visit:\n{auth_url}\n");
    let _ = open::that(&auth_url);

    println!("Waiting for authorization callback on localhost:{REDIRECT_PORT}...");
    println!("(You'll need to approve access in the Monzo app after logging in)\n");

    let code = wait_for_callback(&state)?;
    println!("Authorization code received. Exchanging for tokens...");

    let token = exchange_code(client_id, client_secret, &code, &redirect_uri).await?;

    let mut config = Config::load()?;
    config.access_token = Some(token.access_token);
    config.refresh_token = token.refresh_token;
    config.user_id = Some(token.user_id);
    config.client_id = Some(client_id.to_string());
    config.client_secret = Some(client_secret.to_string());
    if let Some(expires_in) = token.expires_in {
        config.token_expires_at = Some(chrono::Utc::now().timestamp() + expires_in);
    }
    config.save()?;

    println!("Authentication successful! Tokens saved.");
    println!("\nIMPORTANT: Open the Monzo app and approve the login request.");
    println!("Your access token expires in ~6 hours. Use `monzo auth refresh` to renew.");

    // Try to auto-detect account
    println!("\nDetecting accounts...");
    let client = crate::client::MonzoClient::new(&config)?;
    match client.accounts(Some("uk_retail")).await {
        Ok(accounts) if !accounts.is_empty() => {
            let acc = &accounts[0];
            config.account_id = Some(acc.id.clone());
            config.save()?;
            println!("Auto-selected account: {} ({})", acc.description, acc.id);
        }
        _ => {
            println!("Could not auto-detect account. You may need to approve in the Monzo app first.");
            println!("Then run: monzo accounts");
        }
    }

    Ok(())
}

fn wait_for_callback(expected_state: &str) -> Result<String> {
    let server = tiny_http::Server::http(format!("127.0.0.1:{REDIRECT_PORT}"))
        .map_err(|e| anyhow::anyhow!("Failed to start local server: {e}"))?;

    let request = server
        .recv()
        .context("Failed to receive callback request")?;

    let url_str = format!("http://localhost{}", request.url());
    let url = Url::parse(&url_str)?;

    let params: std::collections::HashMap<_, _> = url.query_pairs().collect();

    let state = params
        .get("state")
        .context("Missing state parameter in callback")?;
    if state != expected_state {
        anyhow::bail!("State mismatch - possible CSRF attack");
    }

    let code = params
        .get("code")
        .context("Missing code parameter in callback")?
        .to_string();

    let html = r#"<html><body style="font-family:system-ui;text-align:center;padding:60px">
        <h1>Authenticated!</h1>
        <p>You can close this tab and return to your terminal.</p>
        </body></html>"#;
    let response = tiny_http::Response::from_string(html)
        .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
    let _ = request.respond(response);

    Ok(code)
}

async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    let http = reqwest::Client::new();
    let resp = http
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("code", code),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({status}): {body}");
    }

    Ok(resp.json().await?)
}

/// Refresh an expired access token
pub async fn refresh_flow() -> Result<()> {
    let mut config = Config::load()?;

    let client_id = config
        .client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .context("No client_id in config. Run `monzo auth login` first.")?
        .to_string();
    let client_secret = config
        .client_secret
        .as_deref()
        .filter(|s| !s.is_empty())
        .context("No client_secret in config. Run `monzo auth login` first.")?
        .to_string();
    let refresh_token = config
        .refresh_token
        .as_deref()
        .filter(|s| !s.is_empty())
        .context(
            "No refresh token available. Only confidential clients get refresh tokens.\n\
             Run `monzo auth login` to re-authenticate.",
        )?
        .to_string();

    println!("Refreshing access token...");
    let token =
        crate::client::MonzoClient::refresh_token(&client_id, &client_secret, &refresh_token)
            .await?;

    config.access_token = Some(token.access_token);
    config.refresh_token = token.refresh_token;
    if let Some(expires_in) = token.expires_in {
        config.token_expires_at = Some(chrono::Utc::now().timestamp() + expires_in);
    }
    config.save()?;

    println!("Token refreshed successfully!");
    Ok(())
}

/// Quick token setup for users who have a token from the Monzo developer playground
pub fn set_token(token: &str, account_id: Option<&str>) -> Result<()> {
    let mut config = Config::load()?;
    config.access_token = Some(token.to_string());
    if let Some(aid) = account_id {
        config.account_id = Some(aid.to_string());
    }
    config.save()?;
    println!("Token saved to config.");
    if account_id.is_some() {
        println!("Account ID saved.");
    } else {
        println!("Tip: run `monzo accounts` to see your accounts and set one.");
    }
    Ok(())
}
