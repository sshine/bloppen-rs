//! Product search parsing and error handling.
#![allow(clippy::expect_used)]

mod common;

use bloppen_api::{Error, SearchQuery};
use common::{SEARCH, anonymous_client};
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_returns_typed_products_without_a_session() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(SEARCH))
        .and(query_param("query", "lego"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "discount": 0,
                "full_image_url": "/image/full.jpg",
                "image_url": "/image/thumb.jpg",
                "name": "Super Lego nøglering",
                "price": "39,00 kr.",
                "price_sold": "0,00 kr.",
                "size": "",
                "sold": "",
                "stand": "18",
                "store": "Vanløse",
            }],
            "pages": 1,
            "total": 1,
        })))
        .mount(&server)
        .await;

    let results = anonymous_client(&server)
        .search(&SearchQuery::new("lego"))
        .await
        .expect("search succeeds");
    assert_eq!(results.total, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results.data[0].name, "Super Lego nøglering");
    assert_eq!(results.data[0].price, "39,00 kr.");
    assert_eq!(results.data[0].stand.as_deref(), Some("18"));
}

#[tokio::test]
async fn a_non_json_body_is_a_decode_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(SEARCH))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let error = anonymous_client(&server)
        .search(&SearchQuery::new("lego"))
        .await
        .expect_err("the body is not JSON");
    assert!(matches!(error, Error::Decode { .. }), "{error:?}");
}
