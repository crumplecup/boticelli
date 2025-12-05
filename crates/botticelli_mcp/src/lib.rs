//! Model Context Protocol (MCP) server for Botticelli.
//!
//! This crate provides an MCP server that exposes Botticelli's capabilities
//! as standardized tools and resources that LLMs can use.
//!
//! # Features
//!
//! - **Tools**: Functions LLMs can call (DB queries, narrative execution, etc.)
//! - **Resources**: Data sources LLMs can read (content, narratives, etc.)
//! - **Prompts**: Reusable prompt templates
//!
//! # Usage
//!
//! ```no_run
//! use botticelli_mcp::{BotticelliRouter, ByteTransport, Server, RouterService};
//! use tokio::io::{stdin, stdout};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = BotticelliRouter::builder()
//!         .name("botticelli")
//!         .version(env!("CARGO_PKG_VERSION"))
//!         .build();
//!     
//!     let server = Server::new(RouterService(router));
//!     let transport = ByteTransport::new(stdin(), stdout());
//!     server.run(transport).await?;
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod resources;
mod server;
mod tools;

pub use error::{McpError, McpResult};
pub use resources::{ResourceRegistry, McpResource, ResourceInfo, NarrativeResource};
pub use server::{BotticelliRouter, BotticelliRouterBuilder};
pub use tools::{ToolRegistry, McpTool, QueryContentTool, EchoTool, ServerInfoTool};

#[cfg(feature = "database")]
pub use resources::ContentResource;

// Re-export key mcp-server types for convenience
pub use mcp_server::{ByteTransport, Router, Server};
pub use mcp_server::router::RouterService;
