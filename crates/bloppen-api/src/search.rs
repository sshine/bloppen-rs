//! Product search: `searches/search.json`.

use reqwest::Method;

use crate::client::{Client, accept_json};
use crate::error::Result;
use crate::types::{SearchQuery, SearchResults};

impl Client {
    /// Searches the public product catalogue.
    ///
    /// This endpoint does not require a session. The response is paginated; the site
    /// reports `pages` and `total` alongside the current page of `data`.
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResults> {
        let mut url = self.url("boerneloppen-theme/searches/search.json")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("query", &query.query);
            pairs.append_pair("sku", query.sku.as_deref().unwrap_or(""));
            pairs.append_pair(
                "boerneloppen_theme_store_id",
                query.store_id.as_deref().unwrap_or(""),
            );
            if let Some(page) = query.page {
                pairs.append_pair("page", &page.to_string());
            }
        }

        let builder = accept_json(self.request(Method::GET, url));
        self.send_json(builder).await
    }
}
