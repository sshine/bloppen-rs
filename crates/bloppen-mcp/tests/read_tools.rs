//! Calling the tools through a real MCP client.
#![allow(clippy::expect_used)]

mod common;

use common::{serve, serve_unauthenticated};
use serde_json::json;

#[tokio::test]
async fn session_status_reports_the_authenticated_session() {
    let harness = serve().await;
    let status = harness.read("bloppen_session_status", json!({})).await;
    assert_eq!(status["authenticated"], json!(true));
    assert_eq!(status["token_present"], json!(true));
    assert!(status["expires_at"].is_string());
    assert!(status["cookie_names"].is_array());
}

#[tokio::test]
async fn search_returns_products_and_respects_the_limit() {
    let harness = serve().await;
    let results = harness
        .read("bloppen_search_products", json!({ "query": "lego" }))
        .await;
    assert_eq!(results["total"], json!(1));
    assert_eq!(results["returned"], json!(1));
    assert_eq!(results["items"][0]["name"], json!("Super Lego nøglering"));
    assert_eq!(results["items"][0]["stand"], json!("18"));

    let limited = harness
        .read(
            "bloppen_search_products",
            json!({ "query": "lego", "limit": 0 }),
        )
        .await;
    assert_eq!(limited["returned"], json!(0));
    assert_eq!(limited["truncated"], json!(true));
}

#[tokio::test]
async fn fetch_page_returns_the_html_and_title() {
    let harness = serve().await;
    let page = harness
        .read(
            "bloppen_fetch_page",
            json!({ "path": "/mit-boerneloppen/klub" }),
        )
        .await;
    assert_eq!(page["status"], json!(200));
    assert_eq!(page["title"], json!("Mit Børneloppen | Loppeklub"));
    assert!(
        page["html"]
            .as_str()
            .is_some_and(|html| html.contains("Loppeklub"))
    );
}

#[tokio::test]
async fn an_unauthenticated_private_page_explains_the_missing_credentials() {
    let harness = serve_unauthenticated(bloppen_mcp::render::OutputLimits::default()).await;
    let message = harness
        .error(
            "bloppen_fetch_page",
            json!({ "path": "/mit-boerneloppen/forside" }),
        )
        .await;
    assert!(
        message.contains("BORNELOPPEN_EMAIL") || message.contains("session"),
        "{message}"
    );
}

#[tokio::test]
async fn an_unauthenticated_status_is_false_rather_than_an_error() {
    let harness = serve_unauthenticated(bloppen_mcp::render::OutputLimits::default()).await;
    let status = harness.read("bloppen_session_status", json!({})).await;
    assert_eq!(status["authenticated"], json!(false));
}
