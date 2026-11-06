//! Command line, environment, and client construction.

use std::io::{self, IsTerminal as _};
use std::path::PathBuf;
use std::time::Duration;

use bloppen_api::{Client, Credentials};
use clap::Parser;
use directories::ProjectDirs;
use tracing_subscriber::EnvFilter;

use crate::render::OutputLimits;

/// Serve the Børneloppen site and mobile API to an MCP client over stdio.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Where the cookie jar and login token are stored.
    #[arg(long, env = "BORNELOPPEN_SESSION_FILE")]
    session_file: Option<PathBuf>,

    /// Base URL, for a staging host or a test server.
    #[arg(long, env = "BORNELOPPEN_BASE_URL", hide = true)]
    base_url: Option<String>,

    /// Seconds to wait for one request.
    #[arg(long, default_value_t = 30)]
    timeout: u64,

    /// Rows a listing returns before it truncates.
    #[arg(long, default_value_t = 100)]
    max_items: usize,

    /// Bytes a page tool may return before truncating the HTML.
    #[arg(long, default_value_t = 100_000)]
    max_bytes: usize,

    /// Print the tool schemas as JSON and exit.
    #[arg(long)]
    pub list_tools: bool,

    /// Sign in and exit without serving.
    #[arg(long)]
    pub check: bool,
}

impl Args {
    /// How much one tool may return.
    pub fn limits(&self) -> OutputLimits {
        OutputLimits {
            max_items: self.max_items,
            max_bytes: self.max_bytes,
        }
    }

    /// Build the one client the whole process shares.
    pub fn client(&self) -> Result<Client, StartupError> {
        let mut builder = Client::builder()
            .session_file(
                self.session_file
                    .clone()
                    .unwrap_or_else(default_session_file),
            )
            .timeout(Duration::from_secs(self.timeout));

        if let Some(credentials) = credentials() {
            builder = builder.credentials(credentials);
        }
        if let Some(base_url) = self
            .base_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            builder = builder.base_url(base_url);
        }

        builder
            .build()
            .map_err(|error| StartupError::Client(Box::new(error)))
    }
}

/// Credentials from the environment, when both parts are present.
pub fn credentials() -> Option<Credentials> {
    let email = std::env::var("BORNELOPPEN_EMAIL").ok()?;
    let password = std::env::var("BORNELOPPEN_PASSWORD").ok()?;
    if email.trim().is_empty() || password.is_empty() {
        return None;
    }
    Some(Credentials::new(email, password))
}

fn default_session_file() -> PathBuf {
    ProjectDirs::from("dk", "sshine", "bloppen")
        .map(|dirs| dirs.data_dir().join("session.json"))
        .unwrap_or_else(|| PathBuf::from("bloppen-session.json"))
}

/// What went wrong before the first request.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// The client could not be built.
    #[error("could not build the API client")]
    Client(#[source] Box<bloppen_api::Error>),

    /// The credentials were not accepted.
    #[error("Børneloppen did not accept the credentials")]
    Rejected(#[source] Box<bloppen_api::Error>),

    /// The tool schemas could not be serialized.
    #[error("could not write the tool schemas")]
    Schemas(#[from] serde_json::Error),
}

/// Send the log to stderr, because stdout carries the protocol.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_env("BORNELOPPEN_MCP_LOG")
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("bloppen_mcp=info,bloppen_api=warn"));

    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_ansi(io::stderr().is_terminal())
        .with_env_filter(filter)
        .init();
}
