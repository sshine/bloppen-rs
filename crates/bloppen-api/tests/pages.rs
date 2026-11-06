//! HTML page access, public paths, and authentication requirements.
#![allow(clippy::expect_used)]

mod common;

use bloppen_api::{AuthError, Error};
use common::{anonymous_client, html};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer};

#[tokio::test]
async fn a_public_page_needs_no_session() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/praktisk-info/oftest-stillede-spoergsmaal"))
        .respond_with(html("Oftest stillede spørgsmål"))
        .mount(&server)
        .await;

    let page = anonymous_client(&server)
        .page("/praktisk-info/oftest-stillede-spoergsmaal")
        .await
        .expect("public page");
    assert_eq!(page.title.as_deref(), Some("Oftest stillede spørgsmål"));
    assert_eq!(page.status, 200);
}

#[tokio::test]
async fn a_private_page_without_credentials_reports_missing_credentials() {
    let server = MockServer::start().await;
    let error = anonymous_client(&server)
        .page("/mit-boerneloppen/forside")
        .await
        .expect_err("no session and no credentials");
    assert!(
        matches!(error, Error::Auth(AuthError::MissingCredentials)),
        "{error:?}"
    );
}
