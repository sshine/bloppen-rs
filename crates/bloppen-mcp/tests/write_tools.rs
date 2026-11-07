//! Write tools through a real MCP client.
#![allow(clippy::expect_used)]

mod common;

use std::collections::HashMap;

use bloppen_mcp::render::OutputLimits;
use bloppen_mcp::server::Mode;
use common::{serve_with_mode, text};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PRODUCTS: &str = "/customer/boerneloppen-theme/products";

fn writes() -> Mode {
    Mode {
        writes: true,
        deletes: false,
    }
}

fn all() -> Mode {
    Mode {
        writes: true,
        deletes: true,
    }
}

async fn form(server: &MockServer, path_value: &str) -> HashMap<String, Vec<String>> {
    let requests = server
        .received_requests()
        .await
        .expect("requests are recorded");
    let request = requests
        .iter()
        .rev()
        .find(|request| request.url.path() == path_value)
        .expect("the endpoint was called");
    let body = String::from_utf8_lossy(&request.body);
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in url::form_urlencoded::parse(body.as_bytes()) {
        map.entry(key.into_owned())
            .or_default()
            .push(value.into_owned());
    }
    map
}

async fn mount_action(server: &MockServer, path_value: &str) {
    Mock::given(method("POST"))
        .and(path(path_value))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "status": "success", "message": "ok" })),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn set_products_active_is_served_in_write_mode() {
    let harness = serve_with_mode(OutputLimits::default(), writes()).await;
    mount_action(&harness.http, &format!("{PRODUCTS}/active.json")).await;

    let result = harness
        .read(
            "bloppen_set_products_active",
            json!({ "booking_id": 42, "product_ids": [1, 2], "active": true }),
        )
        .await;
    assert_eq!(result["status"], json!("success"));

    let form = form(&harness.http, &format!("{PRODUCTS}/active.json")).await;
    assert_eq!(form["booking_id"], vec!["42"]);
    assert_eq!(form["product_ids[]"], vec!["1", "2"]);
}

#[tokio::test]
async fn save_products_posts_the_sites_json_document() {
    let harness = serve_with_mode(OutputLimits::default(), writes()).await;
    Mock::given(method("POST"))
        .and(path(format!("{PRODUCTS}/saveBooking.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "newProductIds": { "tmp": "123" },
        })))
        .mount(&harness.http)
        .await;

    let result = harness
        .read(
            "bloppen_save_products",
            json!({
                "booking_id": 42,
                "category_id": 12,
                "products": [{
                    "temp_id": "tmp",
                    "name": "Lego",
                    "price": "39,00",
                    "category_id": 12,
                    "size_id": 34,
                }],
            }),
        )
        .await;
    assert_eq!(result["status"], json!("success"));

    let form = form(&harness.http, &format!("{PRODUCTS}/saveBooking.json")).await;
    let payload: Value = serde_json::from_str(
        form.get("json")
            .and_then(|values| values.first())
            .expect("the json field"),
    )
    .expect("valid json");
    assert_eq!(payload["booking_id"], json!(42));
    assert_eq!(payload["products"][0]["name"], json!("Lego"));
    assert_eq!(
        payload["products"][0]["boerneloppen_theme_specification_id"],
        json!("34")
    );
}

#[tokio::test]
async fn delete_needs_the_delete_tier_and_confirmation() {
    let harness = serve_with_mode(OutputLimits::default(), all()).await;
    mount_action(&harness.http, &format!("{PRODUCTS}/delete.json")).await;

    let refused = harness
        .error(
            "bloppen_delete_products",
            json!({ "booking_id": 42, "product_ids": [7], "confirm": false }),
        )
        .await;
    assert!(refused.contains("confirm"), "{refused}");

    let result = harness
        .read(
            "bloppen_delete_products",
            json!({ "booking_id": 42, "product_ids": [7], "confirm": true }),
        )
        .await;
    assert_eq!(result["status"], json!("success"));
}

#[tokio::test]
async fn write_mode_does_not_serve_delete() {
    let harness = serve_with_mode(OutputLimits::default(), writes()).await;
    let names: Vec<String> = harness
        .tools()
        .await
        .into_iter()
        .map(|tool| tool.name.into())
        .collect();
    assert!(!names.contains(&"bloppen_delete_products".to_owned()));
}

#[tokio::test]
async fn a_failed_write_is_a_tool_error() {
    let harness = serve_with_mode(OutputLimits::default(), writes()).await;
    Mock::given(method("POST"))
        .and(path(format!("{PRODUCTS}/active.json")))
        .respond_with(ResponseTemplate::new(500).set_body_string("no"))
        .mount(&harness.http)
        .await;

    let result = harness
        .call(
            "bloppen_set_products_active",
            json!({ "booking_id": 42, "product_ids": [1], "active": true }),
        )
        .await;
    assert_eq!(result.is_error, Some(true));
    assert!(text(&result).contains("500"), "{}", text(&result));
}
