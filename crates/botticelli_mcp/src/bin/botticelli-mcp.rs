//! Botticelli MCP server binary.

use anyhow::Result;
use botticelli_mcp::{BotticelliRouter, ByteTransport, Router, RouterService, Server};
use tokio::io::{stdin, stdout};
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting Botticelli MCP server");

    // Create router with default tools
    let router = BotticelliRouter::builder()
        .name("botticelli")
        .version(env!("CARGO_PKG_VERSION"))
        .build();

    tracing::info!(
        tools = router.list_tools().len(),
        "Router initialized"
    );

    // Create and run server with stdio transport
    let server = Server::new(RouterService(router));
    let transport = ByteTransport::new(stdin(), stdout());

    tracing::info!("Server ready, listening on stdio");
    server.run(transport).await?;

    Ok(())
}
