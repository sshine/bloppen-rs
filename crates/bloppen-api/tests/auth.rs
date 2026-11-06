//! Sign-in, session status, reauthentication, persistence, and logout.
#![allow(clippy::expect_used)]

mod common;

use std::time::Duration;

use bloppen_api::{AuthError, Client, Error};
use common::{
    EMAIL, HOME, LOGIN, TOKEN_PATH, client, client_with_margin, count_requests, html,
    mount_home_with_redirect_at, mount_login_page, mount_password_login, mount_standard_login,
    mount_token,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn temp_session(name: &str) -> std::path::PathBuf {
    std::env::temp_dir()
        .join(format!("bloppen-api-{}-{name}", std::process::id()))
        .join("session.json")
}

#[tokio::test]
async fn signing_in_reaches_the_home_page_and_stores_the_token() {
    let server = MockServer::start().await;
    mount_standard_login(&server).await;
    let client = client(&server, None);

    let status = client.sign_in().await.expect("signs in");
    assert!(status.authenticated);
    assert_eq!(status.account.as_deref(), Some(EMAIL));
    assert!(status.token_present);
    assert!(status.expires_at.is_some());
    assert!(status.cookie_names.iter().any(|name| name == "vmcms"));
}

#[tokio::test]
async fn a_rejected_password_is_auth_rejected() {
    let server = MockServer::start().await;
    mount_login_page(&server).await;
    Mock::given(method("POST"))
        .and(path(LOGIN))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(HOME))
        .respond_with(html("Mit Børneloppen | Login"))
        .mount(&server)
        .await;

    let error = client(&server, None)
        .sign_in()
        .await
        .expect_err("fake credentials are rejected");
    assert!(
        matches!(error, Error::Auth(AuthError::Rejected)),
        "{error:?}"
    );
}

#[tokio::test]
async fn session_status_reports_an_expired_session_without_reauthing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(TOKEN_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "error",
            "message": "Log ind med brugernavn/adgangskode først.",
        })))
        .mount(&server)
        .await;

    let status = client(&server, None)
        .session_status()
        .await
        .expect("status is readable");
    assert!(!status.authenticated);
    assert!(!status.token_present);
    assert!(!status.cookie_names.iter().any(|name| name == "vmcms"));
}

#[tokio::test]
async fn a_login_redirect_on_a_private_page_triggers_one_reauth() {
    let server = MockServer::start().await;
    mount_login_page(&server).await;
    mount_password_login(&server).await;
    mount_token(&server).await;
    // Call 0 is the post-login check; call 1 is the page request that looks expired.
    mount_home_with_redirect_at(&server, 1).await;

    let client = client(&server, None);
    client.sign_in().await.expect("signs in");
    let page = client.page(HOME).await.expect("reauths and fetches");
    assert_eq!(page.title.as_deref(), Some("Mit Børneloppen"));
    assert_eq!(count_requests(&server, "POST", LOGIN).await, 2);
}

#[tokio::test]
async fn a_cookie_inside_the_reauth_margin_is_renewed_proactively() {
    let server = MockServer::start().await;
    mount_standard_login(&server).await;

    // The cookie is seven days old; an eight-day margin makes it stale by definition.
    let client = client_with_margin(&server, None, Duration::from_secs(8 * 24 * 3600));
    client.sign_in().await.expect("signs in");
    let page = client.page(HOME).await.expect("fetches after reauth");
    assert_eq!(page.title.as_deref(), Some("Mit Børneloppen"));
    assert_eq!(count_requests(&server, "POST", LOGIN).await, 2);
}

#[tokio::test]
async fn a_session_file_restores_without_another_login() {
    let server = MockServer::start().await;
    mount_standard_login(&server).await;
    let path = temp_session("restore");
    let _ = std::fs::remove_dir_all(path.parent().expect("a parent"));

    let first = client(&server, Some(path.clone()));
    first.sign_in().await.expect("signs in");
    assert!(path.exists());
    drop(first);

    let second: Client = client(&server, Some(path));
    let status = second.session_status().await.expect("restores status");
    assert!(status.authenticated);
    assert!(status.token_present);
    assert_eq!(count_requests(&server, "POST", LOGIN).await, 1);
}

#[tokio::test]
async fn signing_out_removes_the_local_session_file() {
    let server = MockServer::start().await;
    mount_standard_login(&server).await;
    Mock::given(method("GET"))
        .and(path("/users/logout"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let path = temp_session("logout");
    let _ = std::fs::remove_dir_all(path.parent().expect("a parent"));
    let client = client(&server, Some(path.clone()));
    client.sign_in().await.expect("signs in");
    assert!(path.exists());
    client.sign_out().await.expect("signs out");
    assert!(!path.exists());
}

#[tokio::test]
async fn a_foreign_origin_is_rejected_before_a_request() {
    let server = MockServer::start().await;
    let error = client(&server, None)
        .page("https://evil.example/")
        .await
        .expect_err("a foreign URL is refused");
    assert!(matches!(error, Error::Invalid(_)), "{error:?}");
}
