//! Shared wiremock harness for the API tests.
#![allow(clippy::expect_used, dead_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bloppen_api::{Client, Credentials};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

pub const EMAIL: &str = "test@example.com";
pub const PASSWORD: &str = "hunter2";
pub const TOKEN: &str = "login-token-value";
pub const HOME: &str = "/mit-boerneloppen/forside";
pub const LOGIN: &str = "/users/login";
pub const TOKEN_PATH: &str = "/boerneloppen-theme/mobileApp/getLoginToken.json";
pub const SEARCH: &str = "/boerneloppen-theme/searches/search.json";

pub fn client(server: &MockServer, session_file: Option<PathBuf>) -> Client {
    client_with_margin(server, session_file, Duration::from_secs(300))
}

pub fn client_with_margin(
    server: &MockServer,
    session_file: Option<PathBuf>,
    reauth_margin: Duration,
) -> Client {
    let mut builder = Client::builder()
        .credentials(Credentials::new(EMAIL, PASSWORD))
        .base_url(server.uri())
        .reauth_margin(reauth_margin);
    if let Some(path) = session_file {
        builder = builder.session_file(path);
    }
    builder.build().expect("valid configuration")
}

pub fn html(title: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(
        format!("<html><head><title>{title}</title></head><body></body></html>"),
        "text/html",
    )
}

/// GET /users/login sets the anonymous vmcms cookie.
pub async fn mount_login_page(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(LOGIN))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Set-Cookie",
                    "vmcms=test-session; Path=/; Max-Age=604800; HttpOnly; Secure",
                )
                .set_body_raw("<html><title>Login</title></html>", "text/html"),
        )
        .mount(server)
        .await;
}

/// POST /users/login sets the logged-in marker; the client then checks the home page.
pub async fn mount_password_login(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path(LOGIN))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Set-Cookie", "hasLogin=1; Path=/")
                .set_body_raw("<html><title>Login</title></html>", "text/html"),
        )
        .mount(server)
        .await;
}

pub async fn mount_home(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(HOME))
        .respond_with(html("Mit Børneloppen"))
        .mount(server)
        .await;
}

pub async fn mount_token(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(TOKEN_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "success",
            "token": TOKEN,
        })))
        .mount(server)
        .await;
}

pub async fn mount_standard_login(server: &MockServer) {
    mount_login_page(server).await;
    mount_password_login(server).await;
    mount_home(server).await;
    mount_token(server).await;
}

/// A home page that redirects to the login page on the given call, then succeeds.
///
/// `redirect_at` is zero-based: 0 redirects the first request, 1 the second, and so on.
/// The sign-in check consumes the first call, so 1 makes the next page fetch look expired.
pub async fn mount_home_with_redirect_at(server: &MockServer, redirect_at: usize) {
    let calls = Arc::new(AtomicUsize::new(0));
    let location = format!("{}/users/login", server.uri());
    Mock::given(method("GET"))
        .and(path(HOME))
        .respond_with(move |_request: &Request| {
            if calls.fetch_add(1, Ordering::SeqCst) == redirect_at {
                ResponseTemplate::new(302).insert_header("Location", location.as_str())
            } else {
                html("Mit Børneloppen")
            }
        })
        .mount(server)
        .await;
}

pub async fn count_requests(server: &MockServer, method_name: &str, path_name: &str) -> usize {
    server
        .received_requests()
        .await
        .expect("wiremock records requests")
        .iter()
        .filter(|request| request.method.as_str() == method_name && request.url.path() == path_name)
        .count()
}

pub fn anonymous_client(server: &MockServer) -> Client {
    Client::builder()
        .base_url(server.uri())
        .build()
        .expect("valid configuration")
}
