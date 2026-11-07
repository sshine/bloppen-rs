//! The `bloppen` command-line client.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bloppen_api::{
    ActionResponse, Client, Credentials, ProductDraft, SaveProductsResponse, SearchQuery,
    SessionStatus,
};
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
    /// Manage products on one stand.
    Product {
        #[command(subcommand)]
        command: ProductCommand,
    },
    /// Activate or deactivate a stand's webshop.
    Shop {
        #[command(subcommand)]
        command: ShopCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ProductCommand {
    /// Print the stand's product-list markup.
    List {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Print the raw HTML.
        #[arg(long)]
        raw: bool,
    },
    /// Add one product to a stand.
    Add {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Short product name, at most 24 characters on the site.
        #[arg(long)]
        name: String,
        /// Price as the site accepts it, such as "39,00".
        #[arg(long)]
        price: String,
        /// Leaf category id.
        #[arg(long)]
        category_id: i64,
        /// Size/specification id.
        #[arg(long)]
        size_id: Option<i64>,
        /// Optional description.
        #[arg(long)]
        description: Option<String>,
        /// Add the product hidden from shoppers.
        #[arg(long)]
        inactive: bool,
    },
    /// Show products in the webshop.
    Active {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Product ids, repeated or comma-separated.
        #[arg(long = "product-id", required = true)]
        product_ids: Vec<u64>,
    },
    /// Hide products from the webshop.
    Inactive {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Product ids, repeated or comma-separated.
        #[arg(long = "product-id", required = true)]
        product_ids: Vec<u64>,
    },
    /// Delete products. Requires --yes.
    Delete {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Product ids, repeated or comma-separated.
        #[arg(long = "product-id", required = true)]
        product_ids: Vec<u64>,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Move products to another category on the same stand.
    Move {
        /// Destination category id.
        #[arg(long)]
        category_id: i64,
        /// Product ids, repeated or comma-separated.
        #[arg(long = "product-id", required = true)]
        product_ids: Vec<u64>,
    },
    /// Save a discount for a stand.
    Discount {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Discount percentage.
        #[arg(long)]
        percent: Option<f64>,
        /// Start date, passed through to the site.
        #[arg(long)]
        from: Option<String>,
        /// End date, passed through to the site.
        #[arg(long)]
        to: Option<String>,
    },
    /// Toggle the note marker on one product.
    Note {
        /// Product id.
        #[arg(long)]
        product_id: u64,
    },
    /// Fetch printable price-tag HTML for products.
    Print {
        /// Product ids, repeated or comma-separated.
        #[arg(long = "product-id", required = true)]
        product_ids: Vec<u64>,
        /// Print the HTML instead of a summary.
        #[arg(long)]
        raw: bool,
    },
    /// Mark products as not printed.
    Unprint {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
        /// Product ids, repeated or comma-separated.
        #[arg(long = "product-id", required = true)]
        product_ids: Vec<u64>,
    },
    /// Upload an image for a product.
    Upload {
        /// Existing product id.
        #[arg(long)]
        product_id: Option<u64>,
        /// Temporary id for a product that has not been saved yet.
        #[arg(long)]
        temp_id: Option<String>,
        /// Use 2 for the second image.
        #[arg(long)]
        image_index: Option<u8>,
        /// Image file.
        #[arg(long)]
        file: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum ShopCommand {
    /// Activate the stand's webshop.
    Activate {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
    },
    /// Deactivate the stand's webshop.
    Deactivate {
        /// Booking id of the stand.
        #[arg(long)]
        booking_id: u64,
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
        Command::Product { command } => run_product(&client, command, args.json).await?,
        Command::Shop { command } => run_shop(&client, command, args.json).await?,
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

async fn run_product(client: &Client, command: ProductCommand, json: bool) -> Result<()> {
    match command {
        ProductCommand::List { booking_id, raw } => {
            let markup = client
                .booking(booking_id)
                .products_markup()
                .await
                .context("fetching the product list")?;
            if raw {
                print!("{markup}");
            } else {
                println!(
                    "{} bytes of product markup (use --raw to print)",
                    markup.len()
                );
            }
        }
        ProductCommand::Add {
            booking_id,
            name,
            price,
            category_id,
            size_id,
            description,
            inactive,
        } => {
            let mut draft = ProductDraft::new(name, price)
                .with_category(category_id)
                .with_active(!inactive);
            if let Some(size_id) = size_id {
                draft = draft.with_size(size_id);
            }
            if let Some(description) = description {
                draft = draft.with_description(description);
            }
            let response = client
                .booking(booking_id)
                .save_products(Some(category_id), None, &[draft])
                .await
                .context("saving the product")?;
            report_save_products(&response, json)?;
        }
        ProductCommand::Active {
            booking_id,
            product_ids,
        } => {
            let response = client
                .booking(booking_id)
                .set_products_active(&product_ids, true)
                .await
                .context("activating products")?;
            report_action(&response, json)?;
        }
        ProductCommand::Inactive {
            booking_id,
            product_ids,
        } => {
            let response = client
                .booking(booking_id)
                .set_products_active(&product_ids, false)
                .await
                .context("deactivating products")?;
            report_action(&response, json)?;
        }
        ProductCommand::Delete {
            booking_id,
            product_ids,
            yes,
        } => {
            if !yes {
                bail!("pass --yes to delete products");
            }
            let response = client
                .booking(booking_id)
                .delete_products(&product_ids)
                .await
                .context("deleting products")?;
            report_action(&response, json)?;
        }
        ProductCommand::Move {
            category_id,
            product_ids,
        } => {
            let response = client
                .move_products_to_category(&product_ids, category_id)
                .await
                .context("moving products")?;
            report_action(&response, json)?;
        }
        ProductCommand::Discount {
            booking_id,
            percent,
            from,
            to,
        } => {
            let response = client
                .booking(booking_id)
                .save_discount(percent, from.as_deref(), to.as_deref())
                .await
                .context("saving the discount")?;
            report_action(&response, json)?;
        }
        ProductCommand::Note { product_id } => {
            let response = client
                .toggle_product_note(product_id)
                .await
                .context("toggling the note")?;
            report_action(&response, json)?;
        }
        ProductCommand::Print { product_ids, raw } => {
            let pages = client
                .print_price_tags(&product_ids)
                .await
                .context("printing price tags")?;
            if raw {
                for page in &pages {
                    print!("{page}");
                }
            } else {
                let bytes: usize = pages.iter().map(String::len).sum();
                println!("{} page(s), {} bytes of price-tag HTML", pages.len(), bytes);
            }
        }
        ProductCommand::Unprint {
            booking_id,
            product_ids,
        } => {
            let response = client
                .booking(booking_id)
                .unprint_products(&product_ids)
                .await
                .context("marking products unprinted")?;
            report_action(&response, json)?;
        }
        ProductCommand::Upload {
            product_id,
            temp_id,
            image_index,
            file,
        } => {
            let bytes = fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            let filename = file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("image");
            let response = client
                .upload_product_image(product_id, temp_id.as_deref(), image_index, bytes, filename)
                .await
                .context("uploading the image")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!(
                    "uploaded: {}",
                    response.image.as_deref().unwrap_or("(no path returned)")
                );
            }
        }
    }
    Ok(())
}

async fn run_shop(client: &Client, command: ShopCommand, json: bool) -> Result<()> {
    let (booking_id, active) = match command {
        ShopCommand::Activate { booking_id } => (booking_id, true),
        ShopCommand::Deactivate { booking_id } => (booking_id, false),
    };
    let response = client
        .booking(booking_id)
        .set_shop_active(active)
        .await
        .context("changing the shop state")?;
    report_action(&response, json)
}

fn report_action(response: &ActionResponse, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(response)?);
    } else {
        println!(
            "{}: {}",
            response.status.as_deref().unwrap_or("(no status)"),
            response.message.as_deref().unwrap_or("")
        );
    }
    if !response.is_success() {
        bail!("the site reported an error");
    }
    Ok(())
}

fn report_save_products(response: &SaveProductsResponse, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(response)?);
    } else {
        println!(
            "{}: {}",
            response.status.as_deref().unwrap_or("(no status)"),
            response.message.as_deref().unwrap_or("")
        );
        if !response.new_product_ids.is_empty() {
            println!("new product ids: {:?}", response.new_product_ids);
        }
    }
    if !response.is_success() {
        bail!("the site reported an error");
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
