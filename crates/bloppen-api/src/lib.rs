//! An async client for the Børneloppen website and its mobile JSON endpoints.
//!
//! Børneloppen is a Danish children's flea-market chain. The Android app is a WebView
//! shell around `www.boerneloppen.dk` with two native JSON calls. This crate keeps the
//! website session in a cookie jar, persists it, and exposes the product search plus raw
//! page access.
//!
//! # Getting started
//!
//! ```no_run
//! use bloppen_api::{Client, Credentials, SearchQuery};
//!
//! # async fn run() -> Result<(), bloppen_api::Error> {
//! let client = Client::new(Credentials::new("you@example.com", "password"))?;
//! client.sign_in().await?;
//!
//! let results = client.search(&SearchQuery::new("lego")).await?;
//! for product in &results.data {
//!     println!("{} — {}", product.name, product.price);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Sessions and expiry
//!
//! The `vmcms` cookie is the primary credential. It lives for seven days, and logging in
//! does not refresh it. The client reads its expiry, reauthenticates before it lapses, and
//! also reacts to the server's expiry signature: a private page that lands on
//! `/users/login`, or `getLoginToken.json` returning `status:"error"`. Either triggers one
//! fresh login and a retry.
//!
//! With [`ClientBuilder::session_file`], cookies and the optional login token are stored
//! in a file readable only by the owner. The token is a re-entry credential for the
//! website's `loginWithToken` endpoint; it is not a bearer token for the native JSON API.
//!
//! # Authentication
//!
//! The login itself is the website form at `POST /users/login` with `email` and
//! `password`. There is no CSRF field in the served form. Credentials are only needed to
//! establish a session; every later request uses the cookie jar.
//!
//! # Product search
//!
//! `GET /boerneloppen-theme/searches/search.json` is public and returns typed product
//! results. The account pages are server-rendered HTML; [`Client::page`] returns the body
//! for those, with the same reauthentication behaviour.

#![warn(missing_docs)]

mod auth;
mod client;
mod error;
mod pages;
mod search;
mod session;
mod types;

pub use client::{
    Client, ClientBuilder, DEFAULT_BASE_URL, DEFAULT_REAUTH_MARGIN, DEFAULT_USER_AGENT,
};
pub use error::{AuthError, Error, Result};
pub use types::{Credentials, Page, Product, SearchQuery, SearchResults, Secret, SessionStatus};
