//! # OCI API
//!
//! Oracle Cloud Infrastructure (OCI) API client for Rust.
//!
//! ## Features
//!
//! - Environment variable-based authentication
//! - Email Delivery service support
//! - Async I/O (tokio)
//!
//! ## Quick Start
//!
//! ```no_run
//! use oci_api::auth::OciConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment variables
//!     let config = OciConfig::from_env()?;
//!     
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod auth;
pub mod client;
pub mod error;
pub mod services;
pub mod utils;

// Re-exports for convenient imports
pub use auth::OciConfig;
pub use client::OciClient;
pub use error::{OciError, Result};

// Re-export email module to allow `oci_api::email::*` (without `services`)
pub use services::email;
