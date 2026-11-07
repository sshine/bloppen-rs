//! HTTP client construction and shared request plumbing.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{ACCEPT, HeaderValue, REFERER};
use reqwest::{Method, RequestBuilder};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use url::Url;

use crate::error::{Error, Result, truncate};
use crate::session::Session;
use crate::types::{Credentials, Secret};

/// The website the app talks to.
pub const DEFAULT_BASE_URL: &str = "https://www.boerneloppen.dk";

/// `User-Agent` sent unless the builder overrides it.
pub const DEFAULT_USER_AGENT: &str = concat!("bloppen-rs/", env!("CARGO_PKG_VERSION"));

/// How long before the `vmcms` expiry the client signs in again.
pub const DEFAULT_REAUTH_MARGIN: Duration = Duration::from_secs(5 * 60);

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) http: reqwest::Client,
    pub(crate) base: Url,
    pub(crate) credentials: Option<Credentials>,
    pub(crate) session: Session,
    pub(crate) auth_lock: Mutex<()>,
    pub(crate) reauth_margin: Duration,
}

/// A client for the Børneloppen website and its mobile JSON endpoints.
///
/// Cheap to clone; clones share one connection pool, one cookie jar, and one auth lock.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) inner: Arc<Inner>,
}

impl Client {
    /// Starts building a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// A client with credentials and default settings.
    pub fn new(credentials: Credentials) -> Result<Self> {
        Self::builder().credentials(credentials).build()
    }

    /// The base URL requests are built against.
    pub fn base_url(&self) -> &Url {
        &self.inner.base
    }

    /// The account email, when credentials were supplied.
    pub fn account(&self) -> Option<&str> {
        self.inner
            .credentials
            .as_ref()
            .map(|credentials| credentials.email.as_str())
    }

    pub(crate) fn session(&self) -> &Session {
        &self.inner.session
    }

    pub(crate) fn credentials(&self) -> Option<&Credentials> {
        self.inner.credentials.as_ref()
    }

    pub(crate) fn reauth_margin(&self) -> Duration {
        self.inner.reauth_margin
    }

    pub(crate) fn auth_lock(&self) -> &Mutex<()> {
        &self.inner.auth_lock
    }

    /// Builds a same-origin URL from a path.
    pub(crate) fn url(&self, path: &str) -> Result<Url> {
        let url = self.inner.base.join(path.trim_start_matches('/'))?;
        self.ensure_same_origin(&url)?;
        Ok(url)
    }

    pub(crate) fn ensure_same_origin(&self, url: &Url) -> Result<()> {
        if url.scheme() != self.inner.base.scheme()
            || url.host_str() != self.inner.base.host_str()
            || url.port_or_known_default() != self.inner.base.port_or_known_default()
        {
            return Err(Error::Invalid("URL outside the Børneloppen origin"));
        }
        Ok(())
    }

    pub(crate) fn request(&self, method: Method, url: Url) -> RequestBuilder {
        self.inner.http.request(method, url)
    }

    /// A request with the headers the website's jQuery AJAX calls send.
    pub(crate) fn write_request(&self, method: Method, url: Url) -> RequestBuilder {
        self.request(method, url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header(
                REFERER,
                format!("{}mit-boerneloppen/stand", self.base_url()),
            )
    }

    /// Sends a builder and decodes a successful JSON body.
    pub(crate) async fn send_json<T: DeserializeOwned>(
        &self,
        builder: RequestBuilder,
    ) -> Result<T> {
        let request = builder.build().map_err(Error::transport)?;
        let method = request.method().clone();
        let path = request.url().path().to_owned();
        let response = self
            .inner
            .http
            .execute(request)
            .await
            .map_err(Error::transport)?;
        let status = response.status();
        let body = response.text().await.map_err(Error::transport)?;
        if !status.is_success() {
            return Err(Error::Http {
                status,
                method,
                path,
                body: truncate(&body, 512),
            });
        }
        serde_json::from_str(&body).map_err(|source| Error::Decode {
            expected: std::any::type_name::<T>(),
            body: truncate(&body, 512),
            source,
        })
    }

    /// Sends a builder and returns the status, final URL, and text body.
    pub(crate) async fn send_text(
        &self,
        builder: RequestBuilder,
    ) -> Result<(reqwest::StatusCode, Url, String)> {
        let response = builder.send().await.map_err(Error::transport)?;
        let status = response.status();
        let final_url = response.url().clone();
        let body = response.text().await.map_err(Error::transport)?;
        if !status.is_success() {
            return Err(Error::Http {
                status,
                method: Method::GET,
                path: final_url.path().to_owned(),
                body: truncate(&body, 512),
            });
        }
        Ok((status, final_url, body))
    }

    /// The current `vmcms` expiry, if the cookie has one.
    pub(crate) fn vmcms_expiry(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.session().vmcms_expiry()
    }

    pub(crate) fn token(&self) -> Option<Secret> {
        self.session().token()
    }
}

/// Builds a [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    credentials: Option<Credentials>,
    base_url: Option<String>,
    session_file: Option<PathBuf>,
    user_agent: String,
    timeout: Duration,
    reauth_margin: Duration,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            credentials: None,
            base_url: None,
            session_file: None,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            timeout: Duration::from_secs(30),
            reauth_margin: DEFAULT_REAUTH_MARGIN,
        }
    }
}

impl ClientBuilder {
    /// Sets the account credentials.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Overrides the base URL, mainly for tests or a staging host.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Stores the cookie jar and optional token in this file.
    pub fn session_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.session_file = Some(path.into());
        self
    }

    /// Sets the `User-Agent`.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Sets the per-request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets how early the client reauthenticates before the `vmcms` expiry.
    pub fn reauth_margin(mut self, margin: Duration) -> Self {
        self.reauth_margin = margin;
        self
    }

    /// Builds the shared client and restores any cached session.
    pub fn build(self) -> Result<Client> {
        let mut base = Url::parse(self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL))?;
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }

        let cookies = Arc::new(reqwest_cookie_store::CookieStoreMutex::new(
            cookie_store::CookieStore::default(),
        ));
        let session = Session::new(Arc::clone(&cookies), self.session_file);
        session.restore();

        let http = reqwest::Client::builder()
            .cookie_provider(cookies)
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent(self.user_agent)
            .timeout(self.timeout)
            .build()
            .map_err(Error::transport)?;

        Ok(Client {
            inner: Arc::new(Inner {
                http,
                base,
                credentials: self.credentials,
                session,
                auth_lock: Mutex::new(()),
                reauth_margin: self.reauth_margin,
            }),
        })
    }
}

/// Adds a JSON `Accept` header to a request.
pub(crate) fn accept_json(builder: RequestBuilder) -> RequestBuilder {
    builder.header(ACCEPT, HeaderValue::from_static("application/json"))
}
