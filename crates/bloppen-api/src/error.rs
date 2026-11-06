//! Error types.

use reqwest::StatusCode;

/// Result alias used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Anything that can go wrong talking to Børneloppen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The request never produced a response, or the body could not be read.
    ///
    /// The URL carried by the underlying error has had its query string stripped,
    /// since `reqwest::Error` renders it in both `Debug` and `Display`.
    #[error("HTTP transport error")]
    Transport(#[source] reqwest::Error),

    /// A URL could not be constructed.
    #[error("could not build request URL")]
    Url(#[from] url::ParseError),

    /// The server answered with a non-success status.
    #[error("{method} {path} failed with HTTP {status}: {body}")]
    Http {
        /// Status code of the response.
        status: StatusCode,
        /// Request method, for context in logs.
        method: reqwest::Method,
        /// Request path without a query string.
        path: String,
        /// A truncated copy of the response body.
        body: String,
    },

    /// The response body did not match the expected shape.
    #[error("could not decode response body as {expected}")]
    Decode {
        /// Name of the Rust type the body was being decoded into.
        expected: &'static str,
        /// The raw body, truncated for diagnostics.
        body: String,
        /// The underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// Authentication or session failure.
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// The session file could not be read or written.
    #[error("could not access the session file {path}")]
    SessionFile {
        /// Path to the session file.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A caller-supplied value was rejected before a request was made.
    #[error("invalid {0}")]
    Invalid(&'static str),
}

impl Error {
    /// The HTTP status, when the error came from a response.
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::Http { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// True when the session is missing, expired, or was rejected as logged out.
    pub fn is_auth_expired(&self) -> bool {
        matches!(
            self,
            Self::Auth(
                AuthError::Expired | AuthError::CookieExpired { .. } | AuthError::NotAuthenticated
            )
        ) || self.status() == Some(StatusCode::UNAUTHORIZED)
    }

    /// True when the server refused access while the caller is authenticated.
    pub fn is_forbidden(&self) -> bool {
        self.status() == Some(StatusCode::FORBIDDEN)
    }

    /// True when the server reported the resource as absent.
    pub fn is_not_found(&self) -> bool {
        self.status() == Some(StatusCode::NOT_FOUND)
    }

    /// True when the server asked the caller to slow down.
    pub fn is_rate_limited(&self) -> bool {
        self.status() == Some(StatusCode::TOO_MANY_REQUESTS)
    }

    /// Wraps a transport failure, dropping the URL's query string on the way in.
    pub(crate) fn transport(mut err: reqwest::Error) -> Self {
        if let Some(url) = err.url_mut() {
            url.set_query(None);
            url.set_fragment(None);
        }
        Self::Transport(err)
    }
}

/// Why a login or session operation failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AuthError {
    /// No email/password and no token were available to establish a session.
    #[error(
        "BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD are needed to sign in; \
         set them in the configured secretspec provider"
    )]
    MissingCredentials,

    /// The login page did not accept the credentials.
    #[error("Børneloppen rejected the email/password")]
    Rejected,

    /// The session had no usable `vmcms` cookie.
    #[error("the vmcms session is missing or expired")]
    Expired,

    /// The `vmcms` cookie's own expiry has passed.
    #[error("the vmcms cookie expired at {expires_at}")]
    CookieExpired {
        /// RFC 3339 timestamp of the cookie expiry.
        expires_at: String,
    },

    /// A page or token call landed on the login page despite a stored session.
    #[error("the session was not authenticated")]
    NotAuthenticated,
}

/// Truncates a response body for an error message.
pub(crate) fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

/// Wraps a session-file I/O error with the path.
pub(crate) fn file_error(path: &std::path::Path, source: std::io::Error) -> Error {
    Error::SessionFile {
        path: path.display().to_string(),
        source,
    }
}
