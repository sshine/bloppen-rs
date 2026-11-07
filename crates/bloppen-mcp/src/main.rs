//! An MCP server over the Børneloppen site, speaking JSON-RPC on stdin and stdout.

use std::process::ExitCode;

use bloppen_mcp::cli::{self, Args, StartupError};
use bloppen_mcp::server::BloppenServer;
use clap::Parser as _;
use rmcp::ServiceExt as _;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    cli::init_tracing();

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            ExitCode::from(2)
        }
    }
}

async fn run(args: Args) -> Result<(), StartupError> {
    let mode = args.mode()?;

    if args.list_tools {
        let tools = BloppenServer::router(mode).list_all();
        println!("{}", serde_json::to_string_pretty(&tools)?);
        return Ok(());
    }

    let client = args.client()?;

    if args.check {
        let status = client
            .sign_in()
            .await
            .map_err(|error| StartupError::Rejected(Box::new(error)))?;
        println!(
            "authenticated; vmcms expires in {}s; token_present={}",
            status.expires_in_seconds.unwrap_or_default(),
            status.token_present
        );
        return Ok(());
    }

    let server = BloppenServer::new(client, args.limits(), mode);
    match server.serve(stdio()).await {
        Ok(service) => {
            if let Err(error) = service.waiting().await {
                tracing::error!(%error, "the connection ended badly");
            }
        }
        Err(error) => tracing::error!(%error, "could not start serving"),
    }

    Ok(())
}

fn report(error: &StartupError) {
    eprintln!("bloppen-mcp: {error}");
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}
