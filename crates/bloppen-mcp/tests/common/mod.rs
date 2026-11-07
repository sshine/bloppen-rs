//! Shared harness: a mocked site underneath, a real MCP client and server on top.
#![allow(clippy::expect_used, dead_code)]

use std::time::Duration;

use bloppen_api::{Client, Credentials};
use bloppen_mcp::render::OutputLimits;
use bloppen_mcp::server::{BloppenServer, Mode};
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt as _};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const EMAIL: &str = "test@example.com";
pub const PASSWORD: &str = "hunter2";
pub const TOKEN: &str = "login-token-value";
pub const HOME: &str = "/mit-boerneloppen/forside";
pub const LOGIN: &str = "/users/login";
pub const TOKEN_PATH: &str = "/boerneloppen-theme/mobileApp/getLoginToken.json";
pub const SEARCH: &str = "/boerneloppen-theme/searches/search.json";

pub struct Harness {
    pub http: MockServer,
    pub mcp: RunningService<RoleClient, ()>,
}

impl Harness {
    pub async fn call(&self, name: &'static str, arguments: Value) -> CallToolResult {
        let mut request = CallToolRequestParams::new(name);
        if let Value::Object(map) = arguments {
            request = request.with_arguments(map);
        }
        self.mcp
            .call_tool(request)
            .await
            .expect("the call reaches the tool")
    }

    pub async fn read(&self, name: &'static str, arguments: Value) -> Value {
        let result = self.call(name, arguments).await;
        assert_ne!(result.is_error, Some(true), "{name} failed: {result:?}");
        result
            .structured_content
            .expect("a read tool answers with structured content")
    }

    pub async fn error(&self, name: &'static str, arguments: Value) -> String {
        let result = self.call(name, arguments).await;
        assert_eq!(result.is_error, Some(true), "{name} unexpectedly succeeded");
        text(&result)
    }

    pub async fn tools(&self) -> Vec<Tool> {
        self.mcp
            .list_all_tools()
            .await
            .expect("the server lists its tools")
    }
}

/// An authenticated server with the default output limits.
pub async fn serve() -> Harness {
    serve_authenticated(OutputLimits::default()).await
}

/// A server whose client has already signed in against the mock site.
pub async fn serve_authenticated(limits: OutputLimits) -> Harness {
    serve_with_mode(limits, Mode::default()).await
}

/// An authenticated server serving `mode`.
pub async fn serve_with_mode(limits: OutputLimits, mode: Mode) -> Harness {
    serve_with_policy(limits, mode, true).await
}

/// A server that only permits product ids it has created itself.
pub async fn serve_guarded(limits: OutputLimits, mode: Mode) -> Harness {
    serve_with_policy(limits, mode, false).await
}

pub async fn serve_with_policy(limits: OutputLimits, mode: Mode, allow_existing: bool) -> Harness {
    let server = MockServer::start().await;
    mount_site(&server).await;
    let client = client(&server, Some(Credentials::new(EMAIL, PASSWORD)));
    client.sign_in().await.expect("the mock login succeeds");
    start(server, client, limits, mode, allow_existing).await
}

/// A server with no credentials and an expired token endpoint.
pub async fn serve_unauthenticated(limits: OutputLimits) -> Harness {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(TOKEN_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "error",
            "message": "Log ind med brugernavn/adgangskode først.",
        })))
        .mount(&server)
        .await;
    let client = client(&server, None);
    start(server, client, limits, Mode::default(), false).await
}

async fn start(
    server: MockServer,
    client: Client,
    limits: OutputLimits,
    mode: Mode,
    allow_existing: bool,
) -> Harness {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        if let Ok(service) = BloppenServer::new(client, limits, mode)
            .allow_existing_products(allow_existing)
            .serve(server_io)
            .await
        {
            let _ = service.waiting().await;
        }
    });
    let mcp = ().serve(client_io).await.expect("the client handshakes");
    Harness { http: server, mcp }
}

pub fn client(server: &MockServer, credentials: Option<Credentials>) -> Client {
    let session = std::env::temp_dir()
        .join(format!(
            "bloppen-mcp-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ))
        .join("session.json");
    let mut builder = Client::builder()
        .base_url(server.uri())
        .session_file(session)
        .timeout(Duration::from_secs(10));
    if let Some(credentials) = credentials {
        builder = builder.credentials(credentials);
    }
    builder.build().expect("valid configuration")
}

pub async fn mount_site(server: &MockServer) {
    mount_login(server).await;
    mount_search(server).await;
    mount_page(server).await;
}

async fn mount_login(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(LOGIN))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Set-Cookie", "vmcms=test; Path=/; Max-Age=604800")
                .set_body_raw("<html><title>Login</title></html>", "text/html"),
        )
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path(LOGIN))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Set-Cookie", "hasLogin=1; Path=/")
                .set_body_raw("<html><title>Login</title></html>", "text/html"),
        )
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(HOME))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw("<html><title>Mit Børneloppen</title></html>", "text/html"),
        )
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(TOKEN_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "token": TOKEN,
        })))
        .mount(server)
        .await;
}

async fn mount_search(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(SEARCH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "discount": 0,
                "name": "Super Lego nøglering",
                "price": "39,00 kr.",
                "size": "",
                "sold": "",
                "stand": "18",
                "store": "Vanløse",
                "image_url": "/thumb.jpg",
                "full_image_url": "/full.jpg",
                "price_sold": "0,00 kr.",
            }],
            "pages": 1,
            "total": 1,
        })))
        .mount(server)
        .await;
}

async fn mount_page(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/mit-boerneloppen/klub"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            "<html><title>Mit Børneloppen | Loppeklub</title></html>",
            "text/html",
        ))
        .mount(server)
        .await;
}

pub fn text(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|block| block.as_text().map(|text| text.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}
