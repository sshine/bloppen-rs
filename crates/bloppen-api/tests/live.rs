//! Tests that talk to the real Børneloppen site.
//!
//! Every test is `#[ignore]`d so `cargo test` never touches the network. Run them with
//! `just live-test`, which resolves `BORNELOPPEN_EMAIL` and `BORNELOPPEN_PASSWORD`
//! through secretspec and runs one test at a time.
#![allow(clippy::expect_used)]

use std::path::PathBuf;

use bloppen_api::{Client, Credentials, SearchQuery};

fn require(name: &str) -> String {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => value,
        _ => panic!(
            "{name} is missing. Run `just live-test`, which resolves the credentials \
             declared in secretspec.toml through the configured provider."
        ),
    }
}

fn session_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir()
        .join(format!("bloppen-live-{}-{name}", std::process::id()))
        .join("session.json");
    let _ = std::fs::remove_dir_all(path.parent().expect("a parent"));
    path
}

fn authenticated_client(name: &str) -> Client {
    Client::builder()
        .credentials(Credentials::new(
            require("BORNELOPPEN_EMAIL"),
            require("BORNELOPPEN_PASSWORD"),
        ))
        .session_file(session_path(name))
        .build()
        .expect("the client builds")
}

#[tokio::test]
#[ignore = "talks to the real site"]
async fn signing_in_yields_a_live_session_and_token() {
    let client = authenticated_client("sign-in");
    let status = client
        .sign_in()
        .await
        .expect("the credentials are accepted");
    assert!(status.authenticated);
    assert!(status.token_present, "sign-in should also fetch the token");
    let expires = status.expires_at.expect("vmcms has an expiry");
    println!(
        "authenticated; vmcms expires at {expires}; in {}s; token_present={}",
        status.expires_in_seconds.unwrap_or_default(),
        status.token_present
    );

    let status = client.session_status().await.expect("status is readable");
    assert!(status.authenticated);
}

#[tokio::test]
#[ignore = "talks to the real site"]
async fn product_search_is_public_and_typed() {
    let client = Client::builder().build().expect("the client builds");
    let results = client
        .search(&SearchQuery::new("lego"))
        .await
        .expect("search succeeds");
    assert!(results.total > 0, "the live search should find something");
    assert!(!results.is_empty());
    println!(
        "search lego: total={} pages={} returned={}",
        results.total,
        results.pages,
        results.len()
    );
}

#[tokio::test]
#[ignore = "talks to the real site"]
async fn the_home_page_is_authenticated_html() {
    let client = authenticated_client("home");
    client
        .sign_in()
        .await
        .expect("the credentials are accepted");
    let page = client
        .page("/mit-boerneloppen/forside")
        .await
        .expect("the home page is readable");
    assert_eq!(page.status, 200);
    assert_eq!(page.title.as_deref(), Some("Mit Børneloppen"));
    println!("home: {} bytes, title {:?}", page.bytes(), page.title);
}

#[tokio::test]
#[ignore = "talks to the real site"]
async fn the_stored_token_reestablishes_a_session() {
    let client = authenticated_client("token-source");
    client
        .sign_in()
        .await
        .expect("the credentials are accepted");
    let token = client.login_token().await.expect("the token is available");

    let fresh = Client::builder()
        .session_file(session_path("token-target"))
        .build()
        .expect("the client builds");
    let status = fresh
        .login_with_token(token.expose())
        .await
        .expect("the token is accepted");
    assert!(status.authenticated);
    println!("token re-entry authenticated a fresh cookie jar");
}

#[tokio::test]
#[ignore = "talks to the real site"]
async fn logout_clears_the_local_session() {
    let path = session_path("logout");
    let client = authenticated_client("logout");
    client
        .sign_in()
        .await
        .expect("the credentials are accepted");
    assert!(path.exists());

    client
        .sign_out()
        .await
        .expect("the logout request completes");
    assert!(!path.exists(), "logout should remove the session cache");
    let status = client.session_status().await.expect("status is readable");
    assert!(!status.authenticated);
}

fn first_booking_id(html: &str) -> Option<u64> {
    for needle in ["data-booking-id=\"", "data-bookingId=\""] {
        if let Some(start) = html.find(needle) {
            let rest = &html[start + needle.len()..];
            let end = rest.find('"')?;
            return rest[..end].parse().ok();
        }
    }
    None
}

#[tokio::test]
#[ignore = "talks to the real site and writes the info-seen flag"]
async fn a_safe_write_round_trips_or_reports_no_access() {
    let client = authenticated_client("write");
    client
        .sign_in()
        .await
        .expect("the credentials are accepted");

    let page = client
        .page("/mit-boerneloppen/stand")
        .await
        .expect("the stand page is readable");
    let booking_id = first_booking_id(&page.html).expect("the account has a stand");

    let response = client
        .booking(booking_id)
        .mark_info_seen()
        .await
        .expect("the write request completes");
    if response.is_success() {
        println!("mark_info_seen accepted for booking {booking_id}");
    } else {
        // The stand page also lists past bookings. The site refuses writes to those with
        // this message, which is useful evidence that ownership/state is enforced.
        println!(
            "mark_info_seen refused for booking {booking_id}: {:?}",
            response.message
        );
        assert!(
            response
                .message
                .as_deref()
                .is_some_and(|message| message.contains("adgang")),
            "{response:?}"
        );
    }
}
