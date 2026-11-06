//! Signing in, session status, login tokens, and reauthentication.

use chrono::Utc;
use reqwest::Method;
use serde::Deserialize;

use crate::client::{Client, accept_json};
use crate::error::{AuthError, Error, Result};
use crate::pages::{extract_title, is_login_page};
use crate::types::{Secret, SessionStatus};

const LOGIN_PATH: &str = "users/login";
const HOME_PATH: &str = "mit-boerneloppen/forside";
const TOKEN_PATH: &str = "boerneloppen-theme/mobileApp/getLoginToken.json";
const TOKEN_LOGIN_PATH: &str = "boerneloppen-theme/mobileApp/loginWithToken.json";
const LOGOUT_PATH: &str = "users/logout";

#[derive(Debug, Deserialize)]
struct LoginToken {
    status: Option<String>,
    token: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginWithToken {
    status: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

impl Client {
    /// Establishes a session from the stored credentials or login token.
    ///
    /// Password login is preferred; the token is the fallback when no password is
    /// available.
    pub async fn sign_in(&self) -> Result<SessionStatus> {
        let _guard = self.auth_lock().lock().await;
        self.sign_in_locked().await
    }

    /// Signs out on the server and removes the local session.
    ///
    /// The local cookie jar and token are cleared even if the logout request fails, so a
    /// caller never remains authenticated locally after asking to sign out.
    pub async fn sign_out(&self) -> Result<()> {
        let _guard = self.auth_lock().lock().await;
        if let Ok(url) = self.url(LOGOUT_PATH) {
            let _ = self.request(Method::GET, url).send().await;
        }
        self.session().clear()?;
        Ok(())
    }

    /// Reports the current session without signing in again.
    ///
    /// An expired session is reported as `authenticated = false`; transport and server
    /// failures are returned as errors.
    pub async fn session_status(&self) -> Result<SessionStatus> {
        let authenticated = match self.fetch_login_token_locked().await {
            Ok(_) => true,
            Err(error) if error.is_auth_expired() => false,
            Err(error) => return Err(error),
        };
        Ok(self.status(authenticated))
    }

    /// Returns the native login token, signing in or reauthenticating if needed.
    pub async fn login_token(&self) -> Result<Secret> {
        self.ensure_authenticated().await?;
        match self.fetch_login_token_locked().await {
            Ok(token) => Ok(token),
            Err(error) if error.is_auth_expired() => {
                self.session().untrust();
                self.reauth().await?;
                self.fetch_login_token_locked().await
            }
            Err(error) => Err(error),
        }
    }

    /// Establishes a session from a native login token.
    pub async fn login_with_token(&self, token: &str) -> Result<SessionStatus> {
        let _guard = self.auth_lock().lock().await;
        self.login_with_token_locked(token).await
    }

    /// Whether the current cookie jar reaches an authenticated page.
    pub async fn is_authenticated(&self) -> Result<bool> {
        self.check_authenticated().await
    }

    pub(crate) async fn ensure_authenticated(&self) -> Result<()> {
        if self.session().is_usable(self.reauth_margin()) {
            return Ok(());
        }
        self.reauth().await
    }

    pub(crate) async fn reauth(&self) -> Result<()> {
        let _guard = self.auth_lock().lock().await;

        // Another task may have refreshed while this one waited for the lock.
        if self.session().is_usable(self.reauth_margin()) {
            return Ok(());
        }

        if self.credentials().is_some() {
            self.sign_in_locked().await?;
        } else if let Some(token) = self.token() {
            self.login_with_token_locked(token.expose()).await?;
        } else {
            return Err(AuthError::MissingCredentials.into());
        }
        Ok(())
    }

    async fn sign_in_locked(&self) -> Result<SessionStatus> {
        if self.credentials().is_some() {
            self.sign_in_with_password().await
        } else if let Some(token) = self.token() {
            self.login_with_token_locked(token.expose()).await
        } else {
            Err(AuthError::MissingCredentials.into())
        }
    }

    async fn sign_in_with_password(&self) -> Result<SessionStatus> {
        let credentials = self
            .credentials()
            .cloned()
            .ok_or(AuthError::MissingCredentials)?;

        self.session().reset();

        let login_url = self.url(LOGIN_PATH)?;
        let _ = self
            .send_text(self.request(Method::GET, login_url.clone()))
            .await?;

        let response = self
            .request(Method::POST, login_url)
            .form(&[
                ("email", credentials.email.as_str()),
                ("password", credentials.password.expose()),
            ])
            .send()
            .await
            .map_err(Error::transport)?;
        let _ = response.text().await.map_err(Error::transport)?;

        if !self.check_authenticated().await? {
            return Err(AuthError::Rejected.into());
        }

        self.session().trust();
        // The token is optional; a login is still valid if fetching it fails.
        let _ = self.fetch_login_token_locked().await;
        self.session().save()?;
        tracing::info!(account = %credentials.email, "signed in");
        Ok(self.status(true))
    }

    async fn login_with_token_locked(&self, token: &str) -> Result<SessionStatus> {
        self.session().reset();

        let login_url = self.url(LOGIN_PATH)?;
        let _ = self.send_text(self.request(Method::GET, login_url)).await?;

        let response = self
            .request(Method::POST, self.url(TOKEN_LOGIN_PATH)?)
            .form(&[("token", token)])
            .send()
            .await
            .map_err(Error::transport)?;
        let status = response.status();
        let body = response.text().await.map_err(Error::transport)?;
        if !status.is_success() {
            return Err(Error::Http {
                status,
                method: Method::POST,
                path: TOKEN_LOGIN_PATH.to_owned(),
                body: crate::error::truncate(&body, 512),
            });
        }

        let parsed: LoginWithToken =
            serde_json::from_str(&body).map_err(|source| Error::Decode {
                expected: "loginWithToken response",
                body: crate::error::truncate(&body, 512),
                source,
            })?;
        if !is_success(parsed.status.as_deref()) {
            return Err(AuthError::Rejected.into());
        }
        if !self.check_authenticated().await? {
            return Err(AuthError::NotAuthenticated.into());
        }

        self.session().set_token(Secret::new(token));
        self.session().trust();
        self.session().save()?;
        tracing::info!("signed in with the stored login token");
        Ok(self.status(true))
    }

    async fn fetch_login_token_locked(&self) -> Result<Secret> {
        let builder = accept_json(self.request(Method::GET, self.url(TOKEN_PATH)?));
        let parsed: LoginToken = self.send_json(builder).await?;
        let token = parsed
            .token
            .filter(|token| !token.is_empty())
            .filter(|_| is_success(parsed.status.as_deref()));

        match token {
            Some(token) => {
                self.session().set_token(Secret::new(&token));
                self.session().trust();
                self.session().save()?;
                Ok(Secret::new(token))
            }
            None => Err(AuthError::Expired.into()),
        }
    }

    async fn check_authenticated(&self) -> Result<bool> {
        let url = self.url(HOME_PATH)?;
        match self.send_text(self.request(Method::GET, url)).await {
            Ok((status, final_url, html)) => {
                let title = extract_title(&html);
                Ok(status.is_success() && !is_login_page(&final_url, title.as_deref()))
            }
            Err(Error::Http { .. }) => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn status(&self, authenticated: bool) -> SessionStatus {
        let expires_at = self.vmcms_expiry();
        let expires_in_seconds = expires_at.map(|expires| (expires - Utc::now()).num_seconds());
        SessionStatus {
            authenticated,
            account: if authenticated {
                self.account().map(ToOwned::to_owned)
            } else {
                None
            },
            expires_at,
            expires_in_seconds,
            token_present: self.token().is_some(),
            cookie_names: self.session().cookie_names(),
        }
    }
}

fn is_success(status: Option<&str>) -> bool {
    status == Some("success")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_literal_success_status_counts() {
        assert!(is_success(Some("success")));
        assert!(!is_success(Some("error")));
        assert!(!is_success(None));
    }
}
