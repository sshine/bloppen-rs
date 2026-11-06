//! The signed-in session: the cookie jar, the optional login token, and persistence.
//!
//! The `vmcms` cookie is the primary credential. Its `Max-Age` is seven days, and
//! logging in does not refresh it, so the client also stores the expiry and reauths
//! before it lapses. The login token is optional and used for biometric-style re-entry.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, MutexGuard, RwLock};

use chrono::{DateTime, Duration, Utc};
use cookie::Expiration;
use cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;
use serde::{Deserialize, Serialize};

use crate::error::{Result, file_error};
use crate::types::Secret;

/// On-disk session shape. The version lets a future change be ignored rather than
/// misread; an unreadable cache is a login, not an error.
#[derive(Debug, Serialize, Deserialize)]
struct Stored {
    version: u8,
    token: Option<String>,
    cookies: String,
}

const STORED_VERSION: u8 = 1;

/// The shared session behind a [`Client`](crate::Client).
#[derive(Debug)]
pub(crate) struct Session {
    cookies: Arc<CookieStoreMutex>,
    token: RwLock<Option<Secret>>,
    file: Option<PathBuf>,
    trusted: RwLock<bool>,
}

impl Session {
    pub(crate) fn new(cookies: Arc<CookieStoreMutex>, file: Option<PathBuf>) -> Self {
        Self {
            cookies,
            token: RwLock::new(None),
            file,
            trusted: RwLock::new(true),
        }
    }

    #[cfg(test)]
    pub(crate) fn cookies(&self) -> &Arc<CookieStoreMutex> {
        &self.cookies
    }

    pub(crate) fn token(&self) -> Option<Secret> {
        read(&self.token).clone()
    }

    pub(crate) fn set_token(&self, token: Secret) {
        *write(&self.token) = Some(token);
    }

    pub(crate) fn clear_token(&self) {
        *write(&self.token) = None;
    }

    pub(crate) fn trust(&self) {
        *write(&self.trusted) = true;
    }

    pub(crate) fn untrust(&self) {
        *write(&self.trusted) = false;
    }

    pub(crate) fn is_trusted(&self) -> bool {
        *read(&self.trusted)
    }

    pub(crate) fn has_vmcms(&self) -> bool {
        lock(&self.cookies)
            .iter_any()
            .any(|cookie| cookie.name() == "vmcms")
    }

    /// The `vmcms` cookie's expiry, when it is a dated cookie.
    pub(crate) fn vmcms_expiry(&self) -> Option<DateTime<Utc>> {
        let store = lock(&self.cookies);
        let cookie = store.iter_any().find(|cookie| cookie.name() == "vmcms")?;
        match cookie.expires() {
            Some(Expiration::DateTime(expires)) => {
                DateTime::<Utc>::from_timestamp(expires.unix_timestamp(), 0)
            }
            Some(Expiration::Session) | None => cookie
                .max_age()
                .map(|age| Utc::now() + Duration::seconds(age.whole_seconds())),
        }
    }

    /// Whether the cookie is already expired or will be within `margin`.
    pub(crate) fn is_expiring(&self, margin: std::time::Duration) -> bool {
        let margin = Duration::from_std(margin).unwrap_or_else(|_| Duration::zero());
        match self.vmcms_expiry() {
            Some(expires) => expires <= Utc::now() + margin,
            None => false,
        }
    }

    /// Whether this session can be used without signing in again.
    pub(crate) fn is_usable(&self, margin: std::time::Duration) -> bool {
        self.is_trusted() && self.has_vmcms() && !self.is_expiring(margin)
    }

    pub(crate) fn cookie_names(&self) -> Vec<String> {
        let mut names: Vec<String> = lock(&self.cookies)
            .iter_any()
            .map(|cookie| cookie.name().to_string())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// Remove cookies and token, but leave any file untouched until a save or clear.
    pub(crate) fn reset(&self) {
        lock(&self.cookies).clear();
        self.clear_token();
        *write(&self.trusted) = false;
    }

    /// Restore a cached session. Anything unreadable or from another version is ignored.
    pub(crate) fn restore(&self) -> bool {
        let Some(path) = &self.file else {
            return false;
        };
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(stored) = serde_json::from_str::<Stored>(&raw) else {
            tracing::debug!(path = %path.display(), "ignoring an unreadable session cache");
            return false;
        };
        if stored.version != STORED_VERSION {
            return false;
        }
        let Ok(jar) = cookie_store::serde::json::load_all(Cursor::new(stored.cookies)) else {
            return false;
        };
        *lock(&self.cookies) = jar;
        *write(&self.token) = stored.token.map(Secret::new);
        *write(&self.trusted) = true;
        tracing::debug!(path = %path.display(), "restored a cached session");
        true
    }

    /// Persist the cookies and optional token.
    pub(crate) fn save(&self) -> Result<()> {
        let Some(path) = &self.file else {
            return Ok(());
        };

        let mut cookies = Vec::new();
        cookie_store::serde::json::save_incl_expired_and_nonpersistent(
            &lock(&self.cookies),
            &mut cookies,
        )
        .map_err(|error| file_error(path, std::io::Error::other(error)))?;

        let stored = Stored {
            version: STORED_VERSION,
            token: self.token().map(|token| token.expose().to_owned()),
            cookies: String::from_utf8_lossy(&cookies).into_owned(),
        };
        let json = serde_json::to_vec(&stored).map_err(|error| file_error(path, error.into()))?;
        write_private(path, &json).map_err(|source| file_error(path, source))
    }

    /// Drop the session and its cache file.
    pub(crate) fn clear(&self) -> Result<()> {
        self.reset();
        if let Some(path) = &self.file {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => return Err(file_error(path, source)),
            }
        }
        Ok(())
    }
}

/// Write a file only the owner can read, replacing whatever was there.
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;

    let tmp = path.with_extension("tmp");
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let mut file = options.open(&tmp)?;
        std::io::Write::write_all(&mut file, bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

fn lock(cookies: &CookieStoreMutex) -> MutexGuard<'_, CookieStore> {
    match cookies.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn read<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use url::Url;

    fn session(file: Option<PathBuf>) -> Session {
        Session::new(
            Arc::new(CookieStoreMutex::new(CookieStore::default())),
            file,
        )
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("bloppen-session-{}-{name}", std::process::id()))
            .join("session.json")
    }

    #[test]
    fn a_session_file_round_trips_cookies_and_token() {
        let path = temp_path("round-trip");
        let _ = std::fs::remove_dir_all(path.parent().expect("a parent"));
        let url = Url::parse("https://www.boerneloppen.dk/").expect("valid URL");
        let cookie = cookie::Cookie::parse("vmcms=abc; Path=/; Max-Age=60").expect("a cookie");

        let written = session(Some(path.clone()));
        lock(written.cookies())
            .insert_raw(&cookie, &url)
            .expect("the cookie is inserted");
        written.set_token(Secret::new("token"));
        written.save().expect("writes the cache");

        let read = session(Some(path));
        assert!(read.restore());
        assert_eq!(
            read.token().map(|token| token.expose().to_owned()),
            Some("token".to_owned())
        );
        assert!(read.has_vmcms());
    }

    #[test]
    fn a_cookie_inside_the_margin_is_not_usable() {
        let session = session(None);
        let url = Url::parse("https://www.boerneloppen.dk/").expect("valid URL");
        let cookie = cookie::Cookie::parse("vmcms=abc; Path=/; Max-Age=3600").expect("a cookie");
        lock(session.cookies())
            .insert_raw(&cookie, &url)
            .expect("the cookie is inserted");
        assert!(session.is_expiring(std::time::Duration::from_secs(7200)));
        assert!(!session.is_usable(std::time::Duration::from_secs(7200)));
    }

    #[test]
    fn clearing_removes_the_file() {
        let path = temp_path("clear");
        let _ = std::fs::remove_dir_all(path.parent().expect("a parent"));
        let session = session(Some(path.clone()));
        session.save().expect("writes the cache");
        assert!(path.exists());
        session.clear().expect("clears the cache");
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn the_session_file_is_private() {
        use std::os::unix::fs::PermissionsExt as _;

        let path = temp_path("private");
        let _ = std::fs::remove_dir_all(path.parent().expect("a parent"));
        let session = session(Some(path.clone()));
        session.save().expect("writes the cache");
        let mode = std::fs::metadata(&path)
            .expect("the file exists")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
