//! The MCP server itself.

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::tool_handler;

use crate::render::OutputLimits;

/// One Børneloppen session, served over MCP.
#[derive(Debug, Clone)]
pub struct BloppenServer {
    pub(crate) client: bloppen_api::Client,
    pub(crate) limits: OutputLimits,
    tool_router: ToolRouter<Self>,
}

impl BloppenServer {
    /// Builds a server around one client.
    pub fn new(client: bloppen_api::Client, limits: OutputLimits) -> Self {
        Self {
            client,
            limits,
            tool_router: Self::router(),
        }
    }

    /// The read-only tools this server serves.
    pub fn router() -> ToolRouter<Self> {
        Self::read_router()
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
            .with_instructions(instructions())
    }
}

fn instructions() -> String {
    "This server exposes read-only access to www.boerneloppen.dk, the Danish children's \
     flea-market site behind the Børneloppen Android app. Product search uses the public \
     search.json endpoint. Page fetches use the website session: private /mit-boerneloppen/ \
     pages require the vmcms cookie, which lives for seven days and is not refreshed by \
     logging in. The client reauthenticates before the cookie expires and once again if a \
     page lands on the login screen. Credentials come from BORNELOPPEN_EMAIL and \
     BORNELOPPEN_PASSWORD in the server environment; the session is cached in the XDG data \
     directory. This server has no write tools."
        .to_owned()
}
