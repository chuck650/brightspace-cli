use anyhow::Result;
use keyring::Entry;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};


use crate::config;

const SERVICE_NAME: &str = "brightspace-cli";

pub fn store_tokens(
    username: &str,
    access_token: &str,
    refresh_token: &str,
) -> Result<()> {
    let access_token_entry = Entry::new(SERVICE_NAME, &format!("{}_access_token", username))?;
    access_token_entry.set_password(access_token)?;

    let refresh_token_entry = Entry::new(SERVICE_NAME, &format!("{}_refresh_token", username))?;
    refresh_token_entry.set_password(refresh_token)?;
    Ok(())
}

pub fn get_access_token(username: &str) -> Result<String> {
    let access_token_entry = Entry::new(SERVICE_NAME, &format!("{}_access_token", username))?;
    let access_token = access_token_entry.get_password()?;
    Ok(access_token)
}

pub fn delete_tokens(username: &str) -> Result<()> {
    let access_token_entry = Entry::new(SERVICE_NAME, &format!("{}_access_token", username))?;
    access_token_entry.delete_password()?;

    let refresh_token_entry = Entry::new(SERVICE_NAME, &format!("{}_refresh_token", username))?;
    refresh_token_entry.delete_password()?;
    Ok(())
}

pub fn login() -> Result<()> {
    let config = config::Config::load()?;

    let client = BasicClient::new(
        ClientId::new(config.client_id),
        Some(ClientSecret::new(config.client_secret)),
        AuthUrl::new(config.auth_url)?,
        Some(TokenUrl::new(config.token_url)?),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_uri)?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("core:*:*".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Open this URL in your browser:\n{}\n", auth_url);

    println!("Please enter the authorization code:");
    let mut code = String::new();
    std::io::stdin().read_line(&mut code)?;

    let token_result = std::thread::spawn(move || {
        client
            .exchange_code(AuthorizationCode::new(code.trim().to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request(http_client)
    })
    .join()
    .unwrap()?;

    store_tokens(
        &config.username,
        token_result.access_token().secret(),
        token_result.refresh_token().unwrap().secret(),
    )?;

    Ok(())
}
