//! The MCP server itself.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::tool_handler;

use crate::render::OutputLimits;

/// Which groups of tools to serve.
///
/// Reads are always served. Writes add the non-destructive product, discount, note,
/// print, and shop operations. Deletes add the destructive product deletion and require
/// writes to be on as well.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mode {
    /// Serve the tools that change a stand's products or shop state.
    pub writes: bool,
    /// Serve the tools that delete something, on top of writes.
    pub deletes: bool,
}

/// One Børneloppen session, served over MCP.
#[derive(Debug, Clone)]
pub struct BloppenServer {
    pub(crate) client: bloppen_api::Client,
    pub(crate) limits: OutputLimits,
    pub(crate) created_products: Arc<Mutex<HashSet<u64>>>,
    pub(crate) allow_existing_products: bool,
    mode: Mode,
    tool_router: ToolRouter<Self>,
}

impl BloppenServer {
    /// Builds a server around one client.
    pub fn new(client: bloppen_api::Client, limits: OutputLimits, mode: Mode) -> Self {
        Self {
            client,
            limits,
            created_products: Arc::new(Mutex::new(HashSet::new())),
            allow_existing_products: false,
            mode,
            tool_router: Self::router(mode),
        }
    }

    /// Allow product-id tools to reach listings this server did not create.
    ///
    /// The default is to permit only products recorded from a successful
    /// `bloppen_save_products` response in this process.
    pub fn allow_existing_products(mut self, allow: bool) -> Self {
        self.allow_existing_products = allow;
        self
    }

    /// Checks that every product id was created by this server, unless overridden.
    pub(crate) fn permits_products(
        &self,
        product_ids: &[u64],
    ) -> Result<(), rmcp::model::CallToolResult> {
        if self.allow_existing_products || product_ids.is_empty() {
            return Ok(());
        }
        let created = self
            .created_products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let unknown: Vec<u64> = product_ids
            .iter()
            .copied()
            .filter(|id| !created.contains(id))
            .collect();
        if unknown.is_empty() {
            Ok(())
        } else {
            Err(crate::render::rejected(format!(
                "refusing to modify product ids {unknown:?}: this server did not create them.                  Start it with --allow-existing-products if you really mean to touch                  pre-existing listings."
            )))
        }
    }

    /// Records product ids returned by a successful save.
    pub(crate) fn record_created_products(&self, ids: impl IntoIterator<Item = u64>) {
        let mut created = self
            .created_products
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        created.extend(ids);
    }

    /// The tools this mode serves.
    pub fn router(mode: Mode) -> ToolRouter<Self> {
        let mut router = Self::read_router();
        if mode.writes {
            router.merge(Self::write_router());
        }
        if mode.deletes {
            router.merge(Self::delete_router());
        }
        router
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BloppenServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(instructions(self.mode))
    }
}

fn instructions(mode: Mode) -> String {
    let mut text = String::from(
        "This server exposes Børneloppen.dk, the Danish children's flea-market site behind \
         the Børneloppen Android app. Read tools cover session status, public product \
         search, and HTML page fetches. Private /mit-boerneloppen/ pages use the vmcms \
         cookie, which lives for seven days and is not refreshed by logging in; the client \
         reauthenticates before it expires and once again if a page lands on the login \
         screen. Credentials come from BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD in the \
         server environment; the session is cached in the XDG data directory.",
    );

    if !mode.writes {
        text.push_str(
            " This server was started without --allow-writes, so it serves no tool that \
             changes a stand.",
        );
    } else if !mode.deletes {
        text.push_str(
            " Write tools are enabled for products, discounts, notes, price-tag printing, \
             and shop state. Deleting products is a separate tier behind --allow-deletes, \
             which is off.",
        );
    } else {
        text.push_str(
            " Write and delete tools are enabled. Deleting products is destructive and \
             cannot be undone.",
        );
    }

    text.push_str(
        " Buying, booking extensions, payment selection, and stand cancellation are not \
         exposed; they involve real money and are outside this server's scope. By default, \
         product-id tools only accept products created by this server in the current \
         process; --allow-existing-products overrides that guard.",
    );
    text
}
