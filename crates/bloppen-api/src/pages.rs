//! Page access, with login-redirect detection and reauthentication.

use reqwest::Method;
use url::Url;

use crate::client::Client;
use crate::error::{AuthError, Result};
use crate::types::Page;

impl Client {
    /// Fetches an HTML page from the Børneloppen origin.
    ///
    /// Private paths under `/mit-boerneloppen/` are treated as authenticated. If the
    /// response lands on the login page, the client reauthenticates once and retries.
    pub async fn page(&self, path: &str) -> Result<Page> {
        let url = self.url(path)?;
        let private = path.starts_with("/mit-boerneloppen/");

        if private {
            self.ensure_authenticated().await?;
        }

        let mut attempted_reauth = false;
        loop {
            let builder = self.request(Method::GET, url.clone());
            let (status, final_url, html) = self.send_text(builder).await?;
            let title = extract_title(&html);
            let login = is_login_page(&final_url, title.as_deref());

            if private && login {
                if attempted_reauth {
                    return Err(AuthError::NotAuthenticated.into());
                }
                self.session().untrust();
                self.reauth().await?;
                attempted_reauth = true;
                continue;
            }

            return Ok(Page {
                path: path.to_owned(),
                final_url: final_url.to_string(),
                status: status.as_u16(),
                title,
                html,
            });
        }
    }
}

/// Whether a response is the login page rather than the requested page.
pub(crate) fn is_login_page(final_url: &Url, title: Option<&str>) -> bool {
    final_url.path().contains("/users/login")
        || title.is_some_and(|title| title.to_lowercase().contains("login"))
}

/// Extracts the first `<title>` text, if present.
pub(crate) fn extract_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let end = html[start..].find("</title>")? + start;
    Some(html[start..end].trim().chars().take(200).collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn a_title_is_extracted_and_trimmed() {
        assert_eq!(
            extract_title("<html><title> Mit Børneloppen </title></html>"),
            Some("Mit Børneloppen".to_owned())
        );
        assert_eq!(extract_title("<html>no title</html>"), None);
    }

    #[test]
    fn a_login_redirect_or_title_is_detected() {
        let url =
            Url::parse("https://www.boerneloppen.dk/users/login?redirect=%2Fx").expect("valid URL");
        assert!(is_login_page(&url, Some("Mit Børneloppen | Login")));
        let home =
            Url::parse("https://www.boerneloppen.dk/mit-boerneloppen/forside").expect("valid URL");
        assert!(!is_login_page(&home, Some("Mit Børneloppen")));
    }
}
