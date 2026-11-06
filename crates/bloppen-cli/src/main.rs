//! The `bloppen` command-line client.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result};
use bloppen_api::{Client, Credentials, SearchQuery, SessionStatus};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Where to store the cookie jar and login token.
    #[arg(long, global = true, env = "BORNELOPPEN_SESSION_FILE")]
    session_file: Option<PathBuf>,

    /// Base URL, for a staging host or a test server.
    #[arg(long, global = true, hide = true, env = "BORNELOPPEN_BASE_URL")]
    base_url: Option<String>,

    /// Print machine-readable JSON where the command supports it.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Sign in, inspect the session, or sign out.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Search the public product catalogue.
    Search {
        /// Free-text query, such as "lego".
        query: String,
        /// Result page, when the site paginates.
        #[arg(long)]
        page: Option<u32>,
        /// Maximum products to print.
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Fetch an HTML page from the site.
    Page {
        /// Absolute path, such as /mit-boerneloppen/forside.
        path: String,
        /// Print the raw HTML instead of a summary.
        #[arg(long)]
        raw: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    /// Sign in with the credentials in BORNELOPPEN_EMAIL/PASSWORD.
    Login,
    /// Report the current session.
    Status,
    /// Sign out and remove the local session.
    Logout,
    /// Print the native login token, redacted unless --show is given.
    Token {
        /// Print the token value.
        #[arg(long)]
        show: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bloppen: {error:#}");
            ExitCode::from(2)
        }
    }
}

async fn run(args: Args) -> Result<()> {
    let client = build_client(&args)?;

    match args.command {
        Command::Auth { command } => match command {
            AuthCommand::Login => {
                let status = client.sign_in().await.context("signing in")?;
                print_status(&status, args.json)?;
            }
            AuthCommand::Status => {
                let status = client.session_status().await.context("reading status")?;
                print_status(&status, args.json)?;
            }
            AuthCommand::Logout => {
                client.sign_out().await.context("signing out")?;
                if args.json {
                    println!("{}", json!({ "signed_out": true }));
                } else {
                    println!("signed out; local session removed");
                }
            }
            AuthCommand::Token { show } => {
                let token = client.login_token().await.context("fetching login token")?;
                if show {
                    println!("{}", token.expose());
                } else {
                    println!("<redacted> ({} bytes)", token.expose().len());
                }
            }
        },
        Command::Search { query, page, limit } => {
            let mut request = SearchQuery::new(query);
            if let Some(page) = page {
                request = request.page(page);
            }
            let results = client.search(&request).await.context("searching")?;
            let shown = results.data.len().min(limit);

            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "total": results.total,
                        "pages": results.pages,
                        "returned": shown,
                        "items": &results.data[..shown],
                    }))?
                );
            } else {
                println!(
                    "{} results in {} pages; showing {}",
                    results.total, results.pages, shown
                );
                for product in &results.data[..shown] {
                    let stand = product.stand.as_deref().unwrap_or("-");
                    let store = product.store.as_deref().unwrap_or("-");
                    let size = product.size.as_deref().unwrap_or("");
                    println!(
                        "  {:<40} {:>10}  {:<12} stand {:<4} {}",
                        truncate(&product.name, 40),
                        product.price,
                        truncate(store, 12),
                        stand,
                        size
                    );
                }
            }
        }
        Command::Page { path, raw } => {
            let page = client.page(&path).await.context("fetching page")?;
            if raw {
                print!("{}", page.html);
            } else if args.json {
                println!("{}", serde_json::to_string_pretty(&page)?);
            } else {
                println!(
                    "{} {} — {} ({} bytes)",
                    page.status,
                    page.final_url,
                    page.title.as_deref().unwrap_or("(no title)"),
                    page.bytes()
                );
            }
        }
    }

    Ok(())
}

fn build_client(args: &Args) -> Result<Client> {
    let session_file = args
        .session_file
        .clone()
        .unwrap_or_else(default_session_file);
    let mut builder = Client::builder()
        .session_file(session_file)
        .timeout(Duration::from_secs(30));

    if let (Ok(email), Ok(password)) = (
        std::env::var("BORNELOPPEN_EMAIL"),
        std::env::var("BORNELOPPEN_PASSWORD"),
    ) {
        if !email.is_empty() && !password.is_empty() {
            builder = builder.credentials(Credentials::new(email, password));
        }
    }

    if let Some(base_url) = &args.base_url {
        builder = builder.base_url(base_url);
    }

    builder.build().map_err(anyhow::Error::from)
}

fn default_session_file() -> PathBuf {
    directories::ProjectDirs::from("dk", "sshine", "bloppen")
        .map(|dirs| dirs.data_dir().join("session.json"))
        .unwrap_or_else(|| PathBuf::from("bloppen-session.json"))
}

fn print_status(status: &SessionStatus, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(status)?);
        return Ok(());
    }

    println!("authenticated: {}", status.authenticated);
    if !status.authenticated {
        if !status.cookie_names.is_empty() {
            println!("cookies: {}", status.cookie_names.join(", "));
        }
        return Ok(());
    }

    if let Some(account) = &status.account {
        println!("account: {account}");
    }
    match status.expires_at {
        Some(expires) => println!(
            "vmcms cookie expires: {expires} (in {}s)",
            status.expires_in_seconds.unwrap_or_default()
        ),
        None => println!("vmcms cookie expires: unknown"),
    }
    println!(
        "login token: {}",
        if status.token_present {
            "present"
        } else {
            "absent"
        }
    );
    if !status.cookie_names.is_empty() {
        println!("cookies: {}", status.cookie_names.join(", "));
    }
    Ok(())
}

fn truncate(value: &str, max: usize) -> String {
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}
