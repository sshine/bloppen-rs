//! Write operations for stands, products, discounts, and shop state.
//!
//! These mirror the website's own AJAX calls. The website sends a JSON document in a
//! form field named `json` for product and discount changes, and repeated
//! `product_ids[]` fields for bulk actions. Buying is deliberately not implemented.

use std::collections::HashMap;

use reqwest::Method;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::client::{Client, accept_json};
use crate::error::{Error, Result};

const PRODUCTS: &str = "customer/boerneloppen-theme/products";

/// One product row as the website's stand editor sends it.
///
/// Fields that the website's hidden inputs send as `"0"`/`"1"` are encoded that way here
/// too. The site's editor sends the full row on add and update; [`ProductDraft::new`]
/// provides the defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDraft {
    /// Existing product id, absent for a new product.
    pub product_id: Option<u64>,
    /// Temporary id used to correlate a new row with uploaded images.
    pub temp_id: Option<String>,
    /// Short product name, at most 24 characters on the site.
    pub name: String,
    /// Formatted price, such as `39,00`.
    pub price: String,
    /// Optional description.
    pub description: Option<String>,
    /// Chosen leaf category id.
    pub category_id: Option<i64>,
    /// Chosen top-level category id.
    pub top_category_id: Option<i64>,
    /// Chosen subcategory id.
    pub sub_category_id: Option<i64>,
    /// Chosen size/specification id.
    pub specification_id: Option<i64>,
    /// Whether the product is visible to shoppers.
    pub active: bool,
    /// Whether this is a merchant "big item".
    pub big_item: bool,
    /// Existing first image path.
    pub image: Option<String>,
    /// Existing second image path.
    pub image_2: Option<String>,
    /// Whether the first image is new.
    pub new_image: bool,
    /// Whether the second image is new.
    pub new_image_2: bool,
    /// Whether the first image is to be deleted.
    pub delete_image: bool,
    /// Whether the second image is to be deleted.
    pub delete_image_2: bool,
    /// Whether the row is deleted.
    pub delete: bool,
    /// Sort order within the category.
    pub order: i64,
}

impl ProductDraft {
    /// A new product with safe defaults: hidden until explicitly activated.
    pub fn new(name: impl Into<String>, price: impl Into<String>) -> Self {
        Self {
            product_id: None,
            temp_id: None,
            name: name.into(),
            price: price.into(),
            description: None,
            category_id: None,
            top_category_id: None,
            sub_category_id: None,
            specification_id: None,
            active: false,
            big_item: false,
            image: None,
            image_2: None,
            new_image: false,
            new_image_2: false,
            delete_image: false,
            delete_image_2: false,
            delete: false,
            order: 0,
        }
    }

    /// Sets the product id for an update.
    pub fn with_product_id(mut self, product_id: u64) -> Self {
        self.product_id = Some(product_id);
        self
    }

    /// Sets the temporary id for a new product.
    pub fn with_temp_id(mut self, temp_id: impl Into<String>) -> Self {
        self.temp_id = Some(temp_id.into());
        self
    }

    /// Sets the leaf category.
    pub fn with_category(mut self, category_id: i64) -> Self {
        self.category_id = Some(category_id);
        self
    }

    /// Sets the size/specification.
    pub fn with_size(mut self, specification_id: i64) -> Self {
        self.specification_id = Some(specification_id);
        self
    }

    /// Sets visibility.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the sort order.
    pub fn with_order(mut self, order: i64) -> Self {
        self.order = order;
        self
    }

    /// The exact JSON object the site's editor would send.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            "product_id".to_owned(),
            json!(self.product_id.map(|id| id.to_string()).unwrap_or_default()),
        );
        map.insert(
            "temp_id".to_owned(),
            json!(self.temp_id.clone().unwrap_or_default()),
        );
        map.insert(
            "image".to_owned(),
            json!(self.image.clone().unwrap_or_default()),
        );
        map.insert("newImage".to_owned(), json!(flag(self.new_image)));
        map.insert("deleteImage".to_owned(), json!(flag(self.delete_image)));
        map.insert("_delete".to_owned(), json!(flag(self.delete)));
        map.insert("name".to_owned(), json!(self.name));
        map.insert("price".to_owned(), json!(self.price));
        map.insert(
            "description".to_owned(),
            json!(self.description.clone().unwrap_or_default()),
        );
        map.insert(
            "boerneloppen_theme_category_id".to_owned(),
            json!(id_string(self.category_id)),
        );
        map.insert(
            "top_category_id".to_owned(),
            json!(id_string(self.top_category_id)),
        );
        map.insert(
            "sub_category_id".to_owned(),
            json!(id_string(self.sub_category_id)),
        );
        map.insert(
            "boerneloppen_theme_specification_id".to_owned(),
            json!(id_string(self.specification_id)),
        );
        map.insert("active".to_owned(), json!(self.active));
        map.insert("big_item".to_owned(), json!(self.big_item));
        map.insert("order".to_owned(), json!(self.order));
        if self.image_2.is_some() || self.new_image_2 || self.delete_image_2 {
            map.insert(
                "image_2".to_owned(),
                json!(self.image_2.clone().unwrap_or_default()),
            );
            map.insert("newImage2".to_owned(), json!(flag(self.new_image_2)));
            map.insert("deleteImage2".to_owned(), json!(flag(self.delete_image_2)));
        }
        Value::Object(map)
    }
}

fn flag(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn id_string(value: Option<i64>) -> String {
    value.map(|id| id.to_string()).unwrap_or_default()
}

fn encoded_form(pairs: &[(String, String)]) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

/// The response from saving one or more product rows.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SaveProductsResponse {
    /// `success` or `error`.
    pub status: Option<String>,
    /// Human-readable message.
    pub message: Option<String>,
    /// Temporary ids mapped to the ids the server assigned.
    #[serde(default)]
    pub new_product_ids: HashMap<String, Value>,
    /// Product ids mapped to uploaded image paths.
    #[serde(default)]
    pub new_product_images: HashMap<String, String>,
    /// Row orders the server rejected as invalid prices.
    #[serde(default)]
    pub invalid_price_orders: Vec<i64>,
}

impl SaveProductsResponse {
    /// Whether the site reported success.
    pub fn is_success(&self) -> bool {
        self.status.as_deref() == Some("success")
    }
}

/// A simple status/message response from a write endpoint.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActionResponse {
    /// `success` or `error`.
    pub status: Option<String>,
    /// Human-readable message.
    pub message: Option<String>,
    /// Optional redirect the website follows after success.
    pub redirect: Option<String>,
}

impl ActionResponse {
    /// Whether the site reported success.
    pub fn is_success(&self) -> bool {
        self.status.as_deref() == Some("success")
    }
}

/// The response from an image upload.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct UploadImageResponse {
    /// The stored image path.
    pub image: Option<String>,
    /// Optional status for an error response.
    pub status: Option<String>,
    /// Optional message for an error response.
    pub message: Option<String>,
}

/// One stand's products, discounts, and shop state.
#[derive(Debug, Clone, Copy)]
pub struct BookingApi<'a> {
    client: &'a Client,
    booking_id: u64,
}

impl Client {
    /// Operations on one booked stand.
    pub fn booking(&self, booking_id: u64) -> BookingApi<'_> {
        BookingApi {
            client: self,
            booking_id,
        }
    }

    /// Moves products to another category within the same stand.
    ///
    /// The endpoint is not scoped to a booking; the category id is the destination.
    pub async fn move_products_to_category(
        &self,
        product_ids: &[u64],
        category_id: i64,
    ) -> Result<ActionResponse> {
        self.ensure_authenticated().await?;
        if product_ids.is_empty() {
            return Err(Error::Invalid("product id list"));
        }

        let mut form: Vec<(String, String)> =
            vec![("category_id".to_owned(), category_id.to_string())];
        for id in product_ids {
            form.push(("product_ids[]".to_owned(), id.to_string()));
        }

        let builder = accept_json(
            self.write_request(
                Method::POST,
                self.url(&format!("{PRODUCTS}/moveToCategory.json"))?,
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            )
            .body(encoded_form(&form)),
        );
        self.send_json(builder).await
    }

    /// Toggles the "note" marker on one product.
    ///
    /// The endpoint is not scoped to a booking; the website calls it as a plain GET and
    /// returns the new state in the message.
    pub async fn toggle_product_note(&self, product_id: u64) -> Result<ActionResponse> {
        self.ensure_authenticated().await?;
        let builder = accept_json(self.write_request(
            Method::GET,
            self.url(&format!("{PRODUCTS}/noteActiveToggle/{product_id}.json"))?,
        ));
        self.send_json(builder).await
    }

    /// Fetches printable price-tag HTML for a set of products.
    ///
    /// The site prints 64 tags per page and returns an HTML fragment per page. This is
    /// not scoped to a booking.
    pub async fn print_price_tags(&self, product_ids: &[u64]) -> Result<Vec<String>> {
        self.ensure_authenticated().await?;
        if product_ids.is_empty() {
            return Err(Error::Invalid("product id list"));
        }

        let pages = product_ids.len().div_ceil(64);
        let ids = product_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut fragments = Vec::with_capacity(pages);

        for page in 1..=pages {
            let mut url = self.url(&format!("{PRODUCTS}/printPriceTags"))?;
            url.query_pairs_mut()
                .append_pair("product_ids", &ids)
                .append_pair("page", &page.to_string())
                .append_pair("max", &pages.to_string());
            let (_status, _url, body) =
                self.send_text(self.write_request(Method::GET, url)).await?;
            fragments.push(body);
        }
        Ok(fragments)
    }

    /// Uploads an image for a new or existing product.
    ///
    /// `image_index` is `Some(2)` for the second image and `None` for the first. This is
    /// not scoped to a booking.
    pub async fn upload_product_image(
        &self,
        product_id: Option<u64>,
        temp_id: Option<&str>,
        image_index: Option<u8>,
        bytes: Vec<u8>,
        filename: &str,
    ) -> Result<UploadImageResponse> {
        self.ensure_authenticated().await?;
        if product_id.is_none() && temp_id.is_none() {
            return Err(Error::Invalid("product id or temp id"));
        }

        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename.to_owned())
            .mime_str("application/octet-stream")
            .map_err(Error::transport)?;
        let mut form = reqwest::multipart::Form::new().part("image", part);
        if let Some(product_id) = product_id {
            form = form.text("product_id", product_id.to_string());
        }
        if let Some(temp_id) = temp_id {
            form = form.text("temp_id", temp_id.to_owned());
        }
        if let Some(image_index) = image_index {
            form = form.text("image_index", image_index.to_string());
        }

        let builder = accept_json(
            self.write_request(Method::POST, self.url(&format!("{PRODUCTS}/uploadImage"))?)
                .multipart(form),
        );
        self.send_json(builder).await
    }
}

impl<'a> BookingApi<'a> {
    /// The booking id these operations address.
    pub fn booking_id(&self) -> u64 {
        self.booking_id
    }

    /// The product list HTML for this stand.
    pub async fn products_markup(&self) -> Result<String> {
        self.client.ensure_authenticated().await?;
        let mut url = self
            .client
            .url(&format!("{PRODUCTS}/getProductsForBooth"))?;
        url.query_pairs_mut()
            .append_pair("booking_id", &self.booking_id.to_string());
        let (_status, _url, body) = self
            .client
            .send_text(self.client.write_request(Method::GET, url))
            .await?;
        Ok(body)
    }

    /// Adds or updates product rows.
    ///
    /// The website sends a JSON object in a form field named `json`; this does the same.
    pub async fn save_products(
        &self,
        category_id: Option<i64>,
        move_to_category_id: Option<i64>,
        products: &[ProductDraft],
    ) -> Result<SaveProductsResponse> {
        self.client.ensure_authenticated().await?;
        if products.is_empty() {
            return Err(Error::Invalid("product list"));
        }

        let payload = json!({
            "booking_id": self.booking_id,
            "category_id": category_id,
            "move_to_category_id": move_to_category_id,
            "products": products.iter().map(ProductDraft::to_json).collect::<Vec<_>>(),
        });
        let json = serde_json::to_string(&payload).map_err(Error::Encode)?;
        let builder = accept_json(
            self.client
                .request(
                    Method::POST,
                    self.client.url(&format!("{PRODUCTS}/saveBooking.json"))?,
                )
                .form(&[("json", json.as_str())]),
        );
        self.client.send_json(builder).await
    }

    /// Shows or hides a set of products.
    pub async fn set_products_active(
        &self,
        product_ids: &[u64],
        active: bool,
    ) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        let action = if active { "active" } else { "inactive" };
        self.bulk_action(action, product_ids).await
    }

    /// Deletes a set of products. The site asks for confirmation; the caller owns that.
    pub async fn delete_products(&self, product_ids: &[u64]) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        self.bulk_action("delete", product_ids).await
    }

    /// Marks a set of products as not printed.
    pub async fn unprint_products(&self, product_ids: &[u64]) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        self.bulk_action("unPrint", product_ids).await
    }

    async fn bulk_action(&self, action: &str, product_ids: &[u64]) -> Result<ActionResponse> {
        if product_ids.is_empty() {
            return Err(Error::Invalid("product id list"));
        }

        let mut form: Vec<(String, String)> =
            vec![("booking_id".to_owned(), self.booking_id.to_string())];
        for id in product_ids {
            form.push(("product_ids[]".to_owned(), id.to_string()));
        }

        let builder = accept_json(
            self.client
                .request(
                    Method::POST,
                    self.client.url(&format!("{PRODUCTS}/{action}.json"))?,
                )
                .header(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                )
                .body(encoded_form(&form)),
        );
        self.client.send_json(builder).await
    }

    /// Moves products to another category within the same stand.
    pub async fn move_products_to_category(
        &self,
        product_ids: &[u64],
        category_id: i64,
    ) -> Result<ActionResponse> {
        self.client
            .move_products_to_category(product_ids, category_id)
            .await
    }

    /// Moves products to another booking for the same user.
    pub async fn move_products(
        &self,
        product_ids: &[u64],
        move_to_booking: u64,
    ) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        if product_ids.is_empty() {
            return Err(Error::Invalid("product id list"));
        }

        let mut form: Vec<(String, String)> = vec![
            ("move_from_booking".to_owned(), self.booking_id.to_string()),
            ("move_to_booking".to_owned(), move_to_booking.to_string()),
        ];
        for id in product_ids {
            form.push(("product_ids[]".to_owned(), id.to_string()));
        }

        let builder = accept_json(
            self.client
                .request(
                    Method::POST,
                    self.client.url(&format!("{PRODUCTS}/move.json"))?,
                )
                .header(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                )
                .body(encoded_form(&form)),
        );
        self.client.send_json(builder).await
    }

    /// Marks the stand's informative overlay as seen.
    pub async fn mark_info_seen(&self) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        let builder = accept_json(
            self.client
                .request(
                    Method::POST,
                    self.client.url(&format!("{PRODUCTS}/infoSeen.json"))?,
                )
                .form(&[("booking_id", self.booking_id.to_string())]),
        );
        self.client.send_json(builder).await
    }

    /// Toggles the "note" marker on one product.
    ///
    /// The website calls this as a plain GET and returns the new state in the message.
    pub async fn toggle_product_note(&self, product_id: u64) -> Result<ActionResponse> {
        self.client.toggle_product_note(product_id).await
    }

    /// Saves a discount for the stand.
    ///
    /// `from`/`to` are passed through as the site's date strings. The website sends a
    /// JSON document in the `json` form field.
    pub async fn save_discount(
        &self,
        percent: Option<f64>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        let payload = json!({
            "booking_id": self.booking_id,
            "percent": percent,
            "from": from,
            "to": to,
        });
        let json = serde_json::to_string(&payload).map_err(Error::Encode)?;
        let builder = accept_json(
            self.client
                .request(
                    Method::POST,
                    self.client.url(&format!("{PRODUCTS}/saveDiscount.json"))?,
                )
                .form(&[("json", json.as_str())]),
        );
        self.client.send_json(builder).await
    }

    /// Activates or deactivates the stand's webshop.
    pub async fn set_shop_active(&self, active: bool) -> Result<ActionResponse> {
        self.client.ensure_authenticated().await?;
        let action = if active {
            "aktiverShop"
        } else {
            "deaktiverShop"
        };
        let builder = accept_json(
            self.client
                .request(
                    Method::POST,
                    self.client.url(&format!("{PRODUCTS}/{action}"))?,
                )
                .form(&[("booking_id", self.booking_id.to_string())]),
        );
        self.client.send_json(builder).await
    }

    /// Fetches printable price-tag HTML for a set of products.
    ///
    /// The site prints 64 tags per page and returns an HTML fragment per page.
    pub async fn print_price_tags(&self, product_ids: &[u64]) -> Result<Vec<String>> {
        self.client.print_price_tags(product_ids).await
    }

    /// Uploads an image for a new or existing product.
    ///
    /// `image_index` is `Some(2)` for the second image and `None` for the first.
    pub async fn upload_product_image(
        &self,
        product_id: Option<u64>,
        temp_id: Option<&str>,
        image_index: Option<u8>,
        bytes: Vec<u8>,
        filename: &str,
    ) -> Result<UploadImageResponse> {
        self.client
            .upload_product_image(product_id, temp_id, image_index, bytes, filename)
            .await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn a_new_product_serializes_the_sites_hidden_flag_strings() {
        let value = ProductDraft::new("Lego", "39,00")
            .with_category(12)
            .with_size(34)
            .to_json();
        assert_eq!(value["product_id"], json!(""));
        assert_eq!(value["temp_id"], json!(""));
        assert_eq!(value["name"], json!("Lego"));
        assert_eq!(value["price"], json!("39,00"));
        assert_eq!(value["boerneloppen_theme_category_id"], json!("12"));
        assert_eq!(value["boerneloppen_theme_specification_id"], json!("34"));
        assert_eq!(value["newImage"], json!("0"));
        assert_eq!(value["_delete"], json!("0"));
        assert_eq!(value["active"], json!(false));
    }

    #[test]
    fn a_product_with_an_id_keeps_it_as_a_string() {
        let value = ProductDraft::new("Lego", "39,00")
            .with_product_id(99)
            .to_json();
        assert_eq!(value["product_id"], json!("99"));
    }
}
