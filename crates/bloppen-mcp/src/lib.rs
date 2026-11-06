//! The MCP server over the Børneloppen site and mobile API.
//!
//! The binary is a thin wrapper: startup lives in [`cli`], the served surface in
//! [`server`] and [`tools`]. It is a library so the integration tests can drive a real
//! client against a mock site over a pipe.

pub mod cli;
pub mod error;
pub mod render;
pub mod server;
pub mod tools;
