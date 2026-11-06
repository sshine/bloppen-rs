//! Live validation probe for the Børneloppen website and mobile API.
//!
//! Run it through `just probe`, which resolves the credentials declared in
//! `secretspec.toml` with the configured provider (keyring by default) and
//! injects them as environment variables. This binary never reads a password
//! from a file or a command-line argument, and it never prints a password,
//! cookie value, or login token.

use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use cookie_store::CookieStore;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, Response};
use reqwest_cookie_store::CookieStoreMutex;
use serde::Deserialize;
use serde_json::Value;

const BASE_URL: &str = "https://www.boerneloppen.dk";
const LOGIN_PATH: &str = "/users/login";
const HOME_PATH: &str = "/mit-boerneloppen/forside";
const TOKEN_PATH: &str = "/boerneloppen-theme/mobileApp/getLoginToken.json";
const TOKEN_LOGIN_PATH: &str = "/boerneloppen-theme/mobileApp/loginWithToken.json";
const LOGOUT_PATH: &str = "/users/logout";
const USER_AGENT: &str = "BoerneloppenApp";

const PAGES: &[(&str, &str)] = &[
    ("home", "/mit-boerneloppen/forside"),
    ("stand", "/mit-boerneloppen/stand"),
    ("sale", "/mit-boerneloppen/salg"),
    ("club", "/mit-boerneloppen/klub"),
    ("profile", "/mit-boerneloppen/profil"),
    ("notifications", "/mit-boerneloppen/notifikationer"),
    ("faq", "/praktisk-info/oftest-stillede-spoergsmaal"),
    ("find", "/praktisk-info/soeg-efter-varer"),
    ("boost", "/mit-boerneloppen/boost-dit-salg"),
];

#[derive(Debug, Deserialize)]
struct LoginToken {
    status: Option<String>,
    token: Option<String>,
    message: Option<String>,
}

struct Config {
    email: String,
    password: String,
    test_writes: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    if args.iter().any(|arg| arg == "--check-env") {
        check_env();
        return Ok(());
    }

    let config = Config::from_env()?;
    println!("== environment");
    println!("BORNELOPPEN_EMAIL: present ({} bytes)", config.email.len());
    println!(
        "BORNELOPPEN_PASSWORD: present ({} bytes)",
        config.password.len()
    );
    println!(
        "BORNELOPPEN_TEST_WRITES: {}",
        if config.test_writes {
            "enabled (write probes are not implemented yet)"
        } else {
            "disabled"
        }
    );

    let (client, cookies) = build_client()?;

    login(&client, &cookies, &config).await?;
    let token = get_login_token(&client).await?;

    if args.iter().any(|arg| arg == "--token-login") {
        match token {
            Some(token) => token_login(&token).await?,
            None => println!("token login: skipped because the token is absent"),
        }
    }

    if !args.iter().any(|arg| arg == "--no-pages") {
        probe_pages(&client).await?;
    }

    if args.iter().any(|arg| arg == "--logout") {
        logout(&client).await?;
    }

    Ok(())
}

impl Config {
    fn from_env() -> Result<Self> {
        let email = require_env("BORNELOPPEN_EMAIL")?;
        let password = require_env("BORNELOPPEN_PASSWORD")?;
        let test_writes = env::var("BORNELOPPEN_TEST_WRITES")
            .ok()
            .filter(|value| !value.is_empty())
            .is_some();

        Ok(Self {
            email,
            password,
            test_writes,
        })
    }
}

fn require_env(name: &str) -> Result<String> {
    match env::var(name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => bail!(
            "{name} is missing. Run `just secrets-set` or \
             `secretspec set {name}`, then run `just probe` so secretspec injects it."
        ),
    }
}

fn check_env() {
    for name in [
        "BORNELOPPEN_EMAIL",
        "BORNELOPPEN_PASSWORD",
        "BORNELOPPEN_TEST_WRITES",
    ] {
        match env::var(name) {
            Ok(value) if !value.is_empty() => {
                println!("{name}: present ({} bytes)", value.len());
            }
            _ => println!("{name}: missing"),
        }
    }
    println!("check-env: no network access attempted");
}

fn print_help() {
    println!(
        "bloppen-probe [--check-env] [--token-login] [--no-pages] [--logout]\n\
         \n\
         Credentials are read from BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD.\n\
         Run through `just probe` so secretspec injects them from the keyring.\n\
         \n\
         --check-env   report whether the names are present, without network\n\
         --token-login exercise loginWithToken.json after getting the token\n\
         --no-pages    skip the authenticated page sweep\n\
         --logout      call /users/logout after the page sweep"
    );
}

fn build_client() -> Result<(Client, Arc<CookieStoreMutex>)> {
    let cookies = Arc::new(CookieStoreMutex::new(CookieStore::default()));
    let client = Client::builder()
        .cookie_provider(Arc::clone(&cookies))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .context("building the HTTP client")?;
    Ok((client, cookies))
}

async fn login(client: &Client, cookies: &CookieStoreMutex, config: &Config) -> Result<()> {
    println!("== login");

    let response = client
        .get(format!("{BASE_URL}{LOGIN_PATH}"))
        .send()
        .await
        .context("GET /users/login")?;
    println!(
        "GET {LOGIN_PATH}: status={} url={}",
        response.status(),
        response.url()
    );
    println!("cookies before login: {}", format_cookies(cookies));
    let before = vmcms_length(cookies);

    let response = client
        .post(format!("{BASE_URL}{LOGIN_PATH}"))
        .form(&[
            ("email", config.email.as_str()),
            ("password", config.password.as_str()),
        ])
        .send()
        .await
        .context("POST /users/login")?;
    println!(
        "POST {LOGIN_PATH}: status={} url={}",
        response.status(),
        response.url()
    );
    println!("cookies after login: {}", format_cookies(cookies));
    let after = vmcms_length(cookies);
    println!(
        "vmcms: before={:?} after={:?} rotated={}",
        before,
        after,
        before != after
    );

    if !is_authenticated(client).await? {
        bail!(
            "the login did not reach an authenticated page; check the credentials and any 2FA/CAPTCHA"
        );
    }

    println!("login: authenticated");
    Ok(())
}

async fn is_authenticated(client: &Client) -> Result<bool> {
    let response = client
        .get(format!("{BASE_URL}{HOME_PATH}"))
        .send()
        .await
        .context("GET the authenticated home page")?;
    let status = response.status();
    let url = response.url().clone();
    let body = response.text().await.context("reading the home page")?;
    let title = extract_title(&body).unwrap_or_default();
    let authenticated = status.is_success()
        && !url.path().contains("/users/login")
        && !title.to_lowercase().contains("login");

    println!(
        "GET {HOME_PATH}: status={status} url={url} title={title:?} authenticated={authenticated}"
    );
    Ok(authenticated)
}

async fn get_login_token(client: &Client) -> Result<Option<String>> {
    println!("== login token");

    let response = client
        .get(format!("{BASE_URL}{TOKEN_PATH}"))
        .header(ACCEPT, "application/json")
        .send()
        .await
        .context("GET getLoginToken.json")?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("reading getLoginToken.json")?;
    println!("GET {TOKEN_PATH}: status={status} bytes={}", body.len());

    match serde_json::from_str::<LoginToken>(&body) {
        Ok(model) => {
            let token = model.token.filter(|value| !value.is_empty());
            println!(
                "  status_field={:?} message={:?} token_present={} token_len={}",
                model.status,
                model
                    .message
                    .as_deref()
                    .map(|message| truncate(message, 200)),
                token.is_some(),
                token.as_ref().map(String::len).unwrap_or(0),
            );
            Ok(token)
        }
        Err(error) => {
            println!("  could not parse LoginToken: {error}");
            Ok(None)
        }
    }
}

async fn token_login(token: &str) -> Result<()> {
    println!("== token login");
    let (client, cookies) = build_client()?;

    let response = client
        .get(format!("{BASE_URL}{LOGIN_PATH}"))
        .send()
        .await
        .context("GET /users/login for the token flow")?;
    println!(
        "GET {LOGIN_PATH}: status={} url={}",
        response.status(),
        response.url()
    );
    println!("cookies before token login: {}", format_cookies(&cookies));

    let response = client
        .post(format!("{BASE_URL}{TOKEN_LOGIN_PATH}"))
        .form(&[("token", token)])
        .send()
        .await
        .context("POST loginWithToken.json")?;
    let status = response.status();
    let url = response.url().clone();
    let body = response
        .text()
        .await
        .context("reading loginWithToken.json")?;
    println!(
        "POST {TOKEN_LOGIN_PATH}: status={status} url={url} bytes={}",
        body.len()
    );
    if let Ok(value) = serde_json::from_str::<Value>(&body) {
        print_json_summary(&value);
    }
    println!("cookies after token login: {}", format_cookies(&cookies));

    let authenticated = is_authenticated(&client).await?;
    println!("token login: authenticated={authenticated}");
    Ok(())
}

async fn probe_pages(client: &Client) -> Result<()> {
    println!("== authenticated pages");
    for (name, path) in PAGES {
        let response = client
            .get(format!("{BASE_URL}{path}"))
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        describe_page(name, path, response).await?;
    }
    Ok(())
}

async fn describe_page(name: &str, path: &str, response: Response) -> Result<()> {
    let status = response.status();
    let url = response.url().clone();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = response
        .text()
        .await
        .with_context(|| format!("reading {path}"))?;
    let title = extract_title(&body).unwrap_or_default();

    println!(
        "  {name}: status={status} url={url} type={content_type:?} bytes={} title={title:?}",
        body.len()
    );

    if content_type.contains("json") {
        if let Ok(value) = serde_json::from_str::<Value>(&body) {
            print_json_summary(&value);
        }
    }
    Ok(())
}

async fn logout(client: &Client) -> Result<()> {
    println!("== logout");
    let response = client
        .get(format!("{BASE_URL}{LOGOUT_PATH}"))
        .send()
        .await
        .context("GET /users/logout")?;
    println!(
        "GET {LOGOUT_PATH}: status={} url={}",
        response.status(),
        response.url()
    );

    let authenticated = is_authenticated(client).await?;
    println!("authenticated after logout={authenticated}");
    Ok(())
}

fn vmcms_length(cookies: &CookieStoreMutex) -> Option<usize> {
    cookie_snapshot(cookies)
        .into_iter()
        .find(|(name, _)| name == "vmcms")
        .map(|(_, length)| length)
}

fn cookie_snapshot(cookies: &CookieStoreMutex) -> Vec<(String, usize)> {
    let store = cookies
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut snapshot: Vec<(String, usize)> = store
        .iter_any()
        .map(|cookie| (cookie.name().to_string(), cookie.value().len()))
        .collect();
    snapshot.sort();
    snapshot
}

fn format_cookies(cookies: &CookieStoreMutex) -> String {
    let snapshot = cookie_snapshot(cookies);
    if snapshot.is_empty() {
        return "(none)".to_string();
    }
    snapshot
        .into_iter()
        .map(|(name, length)| format!("{name}({length} bytes)"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_json_summary(value: &Value) {
    match value {
        Value::Object(map) => {
            let keys = map.keys().cloned().collect::<Vec<_>>().join(", ");
            println!("  json keys: {keys}");
            if let Some(status) = map.get("status").and_then(Value::as_str) {
                println!("  status_field={status:?}");
            }
            if let Some(message) = map.get("message").and_then(Value::as_str) {
                println!("  message={:?}", truncate(message, 200));
            }
            if let Some(token) = map.get("token").and_then(Value::as_str) {
                println!("  token_present=true token_len={}", token.len());
            } else {
                println!("  token_present=false");
            }
            if let Some(badge) = map.get("badge") {
                println!("  badge={badge}");
            }
        }
        other => println!("  json type: {}", json_type(other)),
    }
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn extract_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let end = html[start..].find("</title>")? + start;
    Some(truncate(html[start..end].trim(), 120))
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}
