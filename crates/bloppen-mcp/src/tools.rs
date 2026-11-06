//! The read-only tool surface.

use bloppen_api::SearchQuery;
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
