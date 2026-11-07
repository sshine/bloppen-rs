//! Write endpoints: paths, form encoding, JSON payloads, and responses.
#![allow(clippy::expect_used)]

mod common;

use std::collections::HashMap;

use bloppen_api::{Client, ProductDraft};
use common::{client, mount_standard_login};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PRODUCTS: &str = "/customer/boerneloppen-theme/products";

async fn signed_in() -> (MockServer, Client) {
    let server = MockServer::start().await;
    mount_standard_login(&server).await;
    let client = client(&server, None);
    client.sign_in().await.expect("the mock login succeeds");
    (server, client)
}

async fn ok(server: &MockServer, path_value: &str) {
    Mock::given(method("POST"))
        .and(path(path_value))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "status": "success", "message": "ok" })),
        )
        .mount(server)
        .await;
}

async fn last_form(server: &MockServer, path_value: &str) -> HashMap<String, Vec<String>> {
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

fn json_field(form: &HashMap<String, Vec<String>>) -> Value {
    let raw = form
        .get("json")
        .and_then(|values| values.first())
        .expect("the json form field");
    serde_json::from_str(raw).expect("the json field is JSON")
}

#[tokio::test]
async fn saving_products_posts_the_sites_json_document() {
    let (server, client) = signed_in().await;
    Mock::given(method("POST"))
        .and(path(format!("{PRODUCTS}/saveBooking.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "newProductIds": { "tmp": "123" },
        })))
        .mount(&server)
        .await;

    let draft = ProductDraft::new("Lego", "39,00")
        .with_temp_id("tmp")
        .with_category(12)
        .with_size(34);
    let response = client
        .booking(42)
        .save_products(Some(12), None, &[draft])
        .await
        .expect("save succeeds");
    assert!(response.is_success());

    let form = last_form(&server, &format!("{PRODUCTS}/saveBooking.json")).await;
    let payload = json_field(&form);
    assert_eq!(payload["booking_id"], json!(42));
    assert_eq!(payload["category_id"], json!(12));
    assert_eq!(payload["products"][0]["name"], json!("Lego"));
    assert_eq!(payload["products"][0]["temp_id"], json!("tmp"));
    assert_eq!(
        payload["products"][0]["boerneloppen_theme_category_id"],
        json!("12")
    );
}

#[tokio::test]
async fn active_and_delete_use_repeated_product_ids() {
    let (server, client) = signed_in().await;
    ok(&server, &format!("{PRODUCTS}/active.json")).await;
    ok(&server, &format!("{PRODUCTS}/delete.json")).await;

    client
        .booking(42)
        .set_products_active(&[1, 2], true)
        .await
        .expect("active succeeds");
    let form = last_form(&server, &format!("{PRODUCTS}/active.json")).await;
    assert_eq!(form["booking_id"], vec!["42"]);
    assert_eq!(form["product_ids[]"], vec!["1", "2"]);

    client
        .booking(42)
        .delete_products(&[7])
        .await
        .expect("delete succeeds");
    let form = last_form(&server, &format!("{PRODUCTS}/delete.json")).await;
    assert_eq!(form["product_ids[]"], vec!["7"]);
}

#[tokio::test]
async fn moving_to_a_category_posts_the_category_and_ids() {
    let (server, client) = signed_in().await;
    ok(&server, &format!("{PRODUCTS}/moveToCategory.json")).await;

    client
        .booking(42)
        .move_products_to_category(&[3, 4], 99)
        .await
        .expect("move succeeds");
    let form = last_form(&server, &format!("{PRODUCTS}/moveToCategory.json")).await;
    assert_eq!(form["category_id"], vec!["99"]);
    assert_eq!(form["product_ids[]"], vec!["3", "4"]);
}

#[tokio::test]
async fn a_discount_posts_the_sites_json_document() {
    let (server, client) = signed_in().await;
    ok(&server, &format!("{PRODUCTS}/saveDiscount.json")).await;

    client
        .booking(42)
        .save_discount(Some(25.0), Some("2026-11-08"), Some("2026-11-10"))
        .await
        .expect("discount succeeds");
    let form = last_form(&server, &format!("{PRODUCTS}/saveDiscount.json")).await;
    let payload = json_field(&form);
    assert_eq!(payload["booking_id"], json!(42));
    assert_eq!(payload["percent"], json!(25.0));
    assert_eq!(payload["from"], json!("2026-11-08"));
}

#[tokio::test]
async fn toggling_a_note_is_a_get_on_the_product() {
    let (server, client) = signed_in().await;
    Mock::given(method("GET"))
        .and(path(format!("{PRODUCTS}/noteActiveToggle/7.json")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "status": "success", "message": "Noten er slået til" })),
        )
        .mount(&server)
        .await;

    let response = client
        .booking(42)
        .toggle_product_note(7)
        .await
        .expect("toggle succeeds");
    assert!(response.is_success());
}

#[tokio::test]
async fn shop_activation_posts_the_booking_id() {
    let (server, client) = signed_in().await;
    ok(&server, &format!("{PRODUCTS}/aktiverShop")).await;

    client
        .booking(42)
        .set_shop_active(true)
        .await
        .expect("shop activation succeeds");
    let form = last_form(&server, &format!("{PRODUCTS}/aktiverShop")).await;
    assert_eq!(form["booking_id"], vec!["42"]);
}

#[tokio::test]
async fn printing_price_tags_returns_the_html_pages() {
    let (server, client) = signed_in().await;
    Mock::given(method("GET"))
        .and(path(format!("{PRODUCTS}/printPriceTags")))
        .respond_with(ResponseTemplate::new(200).set_body_raw("<html>tags</html>", "text/html"))
        .mount(&server)
        .await;

    let pages = client
        .booking(42)
        .print_price_tags(&[1, 2])
        .await
        .expect("printing succeeds");
    assert_eq!(pages, vec!["<html>tags</html>"]);
}

#[tokio::test]
async fn uploading_an_image_posts_multipart_form_data() {
    let (server, client) = signed_in().await;
    Mock::given(method("POST"))
        .and(path(format!("{PRODUCTS}/uploadImage")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "image": "/uploads/image.jpg" })),
        )
        .mount(&server)
        .await;

    let response = client
        .booking(42)
        .upload_product_image(Some(7), None, None, vec![0, 1, 2], "photo.jpg")
        .await
        .expect("upload succeeds");
    assert_eq!(response.image.as_deref(), Some("/uploads/image.jpg"));

    let requests = server
        .received_requests()
        .await
        .expect("requests are recorded");
    let request = requests
        .iter()
        .find(|request| request.url.path() == format!("{PRODUCTS}/uploadImage"))
        .expect("the upload was called");
    let content_type = request
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(
        content_type.starts_with("multipart/form-data"),
        "{content_type}"
    );
    let body = String::from_utf8_lossy(&request.body);
    assert!(body.contains("product_id"), "{body}");
    assert!(body.contains("photo.jpg"), "{body}");
}
