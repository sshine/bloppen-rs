//! Public data types shared by the client, CLI, and MCP server.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A value that must not appear in logs or `Debug` output.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    /// Wraps a secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// The underlying value, for the one place that needs to send it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl<T: Into<String>> From<T> for Secret {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Email/password credentials for the Børneloppen website.
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    /// Account email address.
    pub email: String,
    /// Account password.
    pub password: Secret,
}

impl Credentials {
    /// Builds credentials from the values the login form accepts.
    pub fn new(email: impl Into<String>, password: impl Into<Secret>) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("email", &self.email)
            .field("password", &"Secret(<redacted>)")
            .finish()
    }
}

/// The current state of the cookie-backed session.
#[derive(Debug, Clone, Serialize)]
pub struct SessionStatus {
    /// Whether `getLoginToken.json` reports a live session.
    pub authenticated: bool,
    /// The account email, when the client was given credentials.
    pub account: Option<String>,
    /// The `vmcms` cookie expiry, when it has one.
    pub expires_at: Option<DateTime<Utc>>,
    /// Seconds until that expiry, when known.
    pub expires_in_seconds: Option<i64>,
    /// Whether an encrypted login token is available for re-entry.
    pub token_present: bool,
    /// Cookie names in the jar. Values are never exposed.
    pub cookie_names: Vec<String>,
}

/// One HTML page fetched from the site.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    /// The path that was requested.
    pub path: String,
    /// The final URL after redirects.
    pub final_url: String,
    /// HTTP status.
    pub status: u16,
    /// The `<title>` text, when present.
    pub title: Option<String>,
    /// The full response body.
    pub html: String,
}

impl Page {
    /// Body size in bytes.
    pub fn bytes(&self) -> usize {
        self.html.len()
    }
}

/// A product search request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    /// Free-text query, such as `lego`.
    pub query: String,
    /// One-based result page, when the site paginates.
    pub page: Option<u32>,
    /// Optional SKU filter.
    pub sku: Option<String>,
    /// Optional store filter.
    pub store_id: Option<String>,
}

impl SearchQuery {
    /// A query with no filters.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            page: None,
            sku: None,
            store_id: None,
        }
    }

    /// Selects a result page.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Filters by SKU.
    pub fn sku(mut self, sku: impl Into<String>) -> Self {
        self.sku = Some(sku.into());
        self
    }

    /// Filters by store.
    pub fn store_id(mut self, store_id: impl Into<String>) -> Self {
        self.store_id = Some(store_id.into());
        self
    }
}

/// One product from `searches/search.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Product {
    /// Discount in the site's own units, usually `0`.
    #[serde(default)]
    pub discount: i64,
    /// Full-size image URL.
    #[serde(default)]
    pub full_image_url: Option<String>,
    /// Thumbnail image URL.
    #[serde(default)]
    pub image_url: Option<String>,
    /// Product name.
    #[serde(default)]
    pub name: String,
    /// Formatted price, such as `39,00 kr.`.
    #[serde(default)]
    pub price: String,
    /// Formatted sold price, when sold.
    #[serde(default)]
    pub price_sold: Option<String>,
    /// Size label.
    #[serde(default)]
    pub size: Option<String>,
    /// Sold marker or timestamp from the site.
    #[serde(default)]
    pub sold: Option<String>,
    /// Stand number.
    #[serde(default)]
    pub stand: Option<String>,
    /// Store name.
    #[serde(default)]
    pub store: Option<String>,
}

/// A page of product search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    /// Matching products.
    #[serde(default)]
    pub data: Vec<Product>,
    /// Number of result pages.
    #[serde(default)]
    pub pages: i64,
    /// Total result count.
    #[serde(default)]
    pub total: i64,
}

impl SearchResults {
    /// Number of products in this page.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether this page has no products.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
