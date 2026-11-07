//! The read-only tool surface.

use bloppen_api::{ProductDraft, SearchQuery};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::error;
use crate::render;
use crate::server::BloppenServer;

/// Parameters for a product search.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchProducts {
    /// Free-text query, such as "lego".
    pub query: String,

    /// Result page, when the site paginates.
    pub page: Option<u32>,

    /// Maximum products to return. Capped by the server's --max-items.
    pub limit: Option<usize>,
}

/// Parameters for fetching one page.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FetchPage {
    /// Absolute path on www.boerneloppen.dk, such as /mit-boerneloppen/forside.
    #[schemars(regex(pattern = r"^/[A-Za-z0-9._~!$&'()*+,;=:@%/-]*$"))]
    pub path: String,
}

#[tool_router(router = read_router, vis = "pub(crate)")]
impl BloppenServer {
    /// Report whether the session is authenticated and when the vmcms cookie expires.
    #[tool(
        name = "bloppen_session_status",
        description = "Reports whether the Børneloppen cookie session is authenticated, \
                       when the vmcms cookie expires, and whether a login token is stored. \
                       This is a cheap read; it does not sign in if the session has expired.",
        annotations(
            title = "Session status",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn session_status(&self) -> CallToolResult {
        match self.client.session_status().await {
            Ok(status) => render::json(&status),
            Err(error) => error::failed(&error),
        }
    }

    /// Search the public product catalogue.
    #[tool(
        name = "bloppen_search_products",
        description = "Searches public Børneloppen listings by free text. Returns a page of \
                       products with name, price, size, store, stand, sold marker, and image \
                       URLs, plus the server's total and page count. This endpoint is public \
                       and does not need a session.",
        annotations(
            title = "Search products",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn search_products(
        &self,
        Parameters(params): Parameters<SearchProducts>,
    ) -> CallToolResult {
        let mut query = SearchQuery::new(params.query);
        if let Some(page) = params.page {
            query = query.page(page);
        }

        let results = match self.client.search(&query).await {
            Ok(results) => results,
            Err(error) => return error::failed(&error),
        };

        let limit = params
            .limit
            .unwrap_or(self.limits.max_items)
            .min(self.limits.max_items);
        let shown = results.data.len().min(limit);
        let value = json!({
            "total": results.total,
            "pages": results.pages,
            "returned": shown,
            "truncated": shown < results.data.len(),
            "items": &results.data[..shown],
        });
        render::limited(value, self.limits.max_bytes)
    }

    /// Fetch an authenticated or public HTML page.
    #[tool(
        name = "bloppen_fetch_page",
        description = "Fetches an HTML page from www.boerneloppen.dk and returns its status, \
                       final URL, title, byte size, and body. Private /mit-boerneloppen/ \
                       paths use the stored session and reauthenticate once if it expired. \
                       The body is truncated to the server's --max-bytes budget.",
        annotations(
            title = "Fetch page",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn fetch_page(&self, Parameters(params): Parameters<FetchPage>) -> CallToolResult {
        let page = match self.client.page(&params.path).await {
            Ok(page) => page,
            Err(error) => return error::failed(&error),
        };

        let bytes = page.bytes();
        let (html, html_truncated) = if page.html.len() <= self.limits.max_bytes {
            (page.html, false)
        } else {
            (truncate_bytes(&page.html, self.limits.max_bytes), true)
        };

        render::limited(
            json!({
                "status": page.status,
                "final_url": page.final_url,
                "title": page.title,
                "bytes": bytes,
                "html": html,
                "html_truncated": html_truncated,
            }),
            self.limits.max_bytes + 8_192,
        )
    }
}

fn truncate_bytes(value: &str, max_bytes: usize) -> String {
    let end = value
        .char_indices()
        .take_while(|(index, character)| index + character.len_utf8() <= max_bytes)
        .last()
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0);
    value[..end].to_owned()
}

/// One product row for a save.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProductInput {
    /// Existing product id, omitted for a new product.
    pub product_id: Option<u64>,
    /// Temporary id for a new product, used to correlate an uploaded image.
    pub temp_id: Option<String>,
    /// Short product name, at most 24 characters on the site.
    pub name: String,
    /// Price as the site accepts it, such as "39,00".
    pub price: String,
    /// Optional description.
    pub description: Option<String>,
    /// Leaf category id; falls back to the request's category_id.
    pub category_id: Option<i64>,
    /// Size/specification id.
    pub size_id: Option<i64>,
    /// Whether the product is visible. Omitted means visible.
    pub active: Option<bool>,
    /// Sort order within the category. Omitted means zero.
    pub order: Option<i64>,
}

/// Add or update product rows on a stand.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SaveProducts {
    /// Booking id of the stand.
    pub booking_id: u64,
    /// Default leaf category id for products that omit their own.
    pub category_id: Option<i64>,
    /// Category to move products into while saving, when the site's editor does that.
    pub move_to_category_id: Option<i64>,
    /// Product rows to add or update.
    pub products: Vec<ProductInput>,
}

/// Change product visibility on a stand.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetProductsActive {
    /// Booking id of the stand.
    pub booking_id: u64,
    /// Product ids to change.
    pub product_ids: Vec<u64>,
    /// True to show the products, false to hide them.
    pub active: bool,
}

/// Move products to another category on the same stand.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MoveProducts {
    /// Destination leaf category id.
    pub category_id: i64,
    /// Product ids to move.
    pub product_ids: Vec<u64>,
}

/// Save a discount for a stand.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SaveDiscount {
    /// Booking id of the stand.
    pub booking_id: u64,
    /// Discount percentage.
    pub percent: Option<f64>,
    /// Start date, passed through to the site.
    pub from: Option<String>,
    /// End date, passed through to the site.
    pub to: Option<String>,
}

/// Toggle the note marker on one product.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ToggleNote {
    /// Product id.
    pub product_id: u64,
}

/// Fetch printable price-tag HTML.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PrintPriceTags {
    /// Product ids to print.
    pub product_ids: Vec<u64>,
}

/// Mark products as not printed.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UnprintProducts {
    /// Booking id of the stand.
    pub booking_id: u64,
    /// Product ids to mark unprinted.
    pub product_ids: Vec<u64>,
}

/// Activate or deactivate a stand's webshop.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetShopActive {
    /// Booking id of the stand.
    pub booking_id: u64,
    /// True to activate the webshop, false to deactivate it.
    pub active: bool,
}

/// Delete products from a stand.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteProducts {
    /// Booking id of the stand.
    pub booking_id: u64,
    /// Product ids to delete.
    pub product_ids: Vec<u64>,
    /// Must be true; deleting products cannot be undone.
    pub confirm: bool,
}

#[tool_router(router = write_router, vis = "pub(crate)")]
impl BloppenServer {
    /// Add or update products on a stand.
    #[tool(
        name = "bloppen_save_products",
        description = "Adds or updates product rows on one stand. New products can carry a \
                       temp_id that matches a later image upload. The site's editor sends \
                       the same JSON document in a form field named json.",
        annotations(
            title = "Save products",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn save_products(
        &self,
        Parameters(params): Parameters<SaveProducts>,
    ) -> CallToolResult {
        let existing: Vec<u64> = params
            .products
            .iter()
            .filter_map(|product| product.product_id)
            .collect();
        if let Err(result) = self.permits_products(&existing) {
            return result;
        }

        let drafts: Vec<ProductDraft> = params
            .products
            .into_iter()
            .map(|product| {
                let category = product.category_id.or(params.category_id);
                let mut draft = ProductDraft::new(product.name, product.price)
                    .with_active(product.active.unwrap_or(true));
                if let Some(id) = product.product_id {
                    draft = draft.with_product_id(id);
                }
                if let Some(temp_id) = product.temp_id {
                    draft = draft.with_temp_id(temp_id);
                }
                if let Some(category) = category {
                    draft = draft.with_category(category);
                }
                if let Some(size_id) = product.size_id {
                    draft = draft.with_size(size_id);
                }
                if let Some(description) = product.description {
                    draft = draft.with_description(description);
                }
                if let Some(order) = product.order {
                    draft = draft.with_order(order);
                }
                draft
            })
            .collect();

        match self
            .client
            .booking(params.booking_id)
            .save_products(params.category_id, params.move_to_category_id, &drafts)
            .await
        {
            Ok(response) => {
                if response.is_success() {
                    let new_ids = response.new_product_ids.values().filter_map(|value| {
                        value
                            .as_u64()
                            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
                    });
                    self.record_created_products(new_ids);
                }
                render::json(&response)
            }
            Err(error) => error::failed(&error),
        }
    }

    /// Show or hide products.
    #[tool(
        name = "bloppen_set_products_active",
        description = "Shows or hides a set of product ids on one stand. This is the same \
                       active/inactive toggle the stand editor uses.",
        annotations(
            title = "Set products active",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn set_products_active(
        &self,
        Parameters(params): Parameters<SetProductsActive>,
    ) -> CallToolResult {
        if let Err(result) = self.permits_products(&params.product_ids) {
            return result;
        }
        match self
            .client
            .booking(params.booking_id)
            .set_products_active(&params.product_ids, params.active)
            .await
        {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }

    /// Move products to another category.
    #[tool(
        name = "bloppen_move_products",
        description = "Moves product ids to another leaf category on the same stand.",
        annotations(
            title = "Move products",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn move_products(
        &self,
        Parameters(params): Parameters<MoveProducts>,
    ) -> CallToolResult {
        if let Err(result) = self.permits_products(&params.product_ids) {
            return result;
        }
        match self
            .client
            .move_products_to_category(&params.product_ids, params.category_id)
            .await
        {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }

    /// Save a stand discount.
    #[tool(
        name = "bloppen_save_discount",
        description = "Saves a discount percentage and optional date range for one stand.",
        annotations(
            title = "Save discount",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn save_discount(
        &self,
        Parameters(params): Parameters<SaveDiscount>,
    ) -> CallToolResult {
        match self
            .client
            .booking(params.booking_id)
            .save_discount(params.percent, params.from.as_deref(), params.to.as_deref())
            .await
        {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }

    /// Toggle a product's note marker.
    #[tool(
        name = "bloppen_toggle_product_note",
        description = "Toggles the note marker on one product and returns the site's \
                       status and message.",
        annotations(
            title = "Toggle product note",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn toggle_product_note(
        &self,
        Parameters(params): Parameters<ToggleNote>,
    ) -> CallToolResult {
        if let Err(result) = self.permits_products(&[params.product_id]) {
            return result;
        }
        match self.client.toggle_product_note(params.product_id).await {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }

    /// Fetch printable price-tag HTML.
    #[tool(
        name = "bloppen_print_price_tags",
        description = "Fetches the site's printable HTML for a set of product ids. The site \
                       prints 64 tags per page and marks them as printed.",
        annotations(
            title = "Print price tags",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn print_price_tags(
        &self,
        Parameters(params): Parameters<PrintPriceTags>,
    ) -> CallToolResult {
        if let Err(result) = self.permits_products(&params.product_ids) {
            return result;
        }
        match self.client.print_price_tags(&params.product_ids).await {
            Ok(pages) => render::limited(
                json!({
                    "pages": pages.len(),
                    "html": pages.join(""),
                }),
                self.limits.max_bytes + 8_192,
            ),
            Err(error) => error::failed(&error),
        }
    }

    /// Mark products unprinted.
    #[tool(
        name = "bloppen_unprint_products",
        description = "Marks a set of product ids as not printed on one stand.",
        annotations(
            title = "Unprint products",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn unprint_products(
        &self,
        Parameters(params): Parameters<UnprintProducts>,
    ) -> CallToolResult {
        if let Err(result) = self.permits_products(&params.product_ids) {
            return result;
        }
        match self
            .client
            .booking(params.booking_id)
            .unprint_products(&params.product_ids)
            .await
        {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }

    /// Activate or deactivate the stand's webshop.
    #[tool(
        name = "bloppen_set_shop_active",
        description = "Activates or deactivates the webshop for one booked stand.",
        annotations(
            title = "Set shop active",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn set_shop_active(
        &self,
        Parameters(params): Parameters<SetShopActive>,
    ) -> CallToolResult {
        match self
            .client
            .booking(params.booking_id)
            .set_shop_active(params.active)
            .await
        {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }
}

#[tool_router(router = delete_router, vis = "pub(crate)")]
impl BloppenServer {
    /// Delete products from a stand.
    #[tool(
        name = "bloppen_delete_products",
        description = "Deletes product ids from one stand. The confirm field must be true; \
                       deletion cannot be undone.",
        annotations(
            title = "Delete products",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn delete_products(
        &self,
        Parameters(params): Parameters<DeleteProducts>,
    ) -> CallToolResult {
        if !params.confirm {
            return render::rejected(
                "confirm was false. Deleting products cannot be undone, so the caller has \
                 to set confirm to true.",
            );
        }
        if let Err(result) = self.permits_products(&params.product_ids) {
            return result;
        }
        match self
            .client
            .booking(params.booking_id)
            .delete_products(&params.product_ids)
            .await
        {
            Ok(response) => render::json(&response),
            Err(error) => error::failed(&error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_keeps_utf8_boundaries() {
        let value = "abcødef";
        let truncated = truncate_bytes(value, 4);
        assert_eq!(truncated, "abc");
        assert!(value.is_char_boundary(truncated.len()));
    }
}
