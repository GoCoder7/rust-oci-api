//! # OCI API
//!
//! Oracle Cloud Infrastructure (OCI) API client for Rust.
//!
//! ## Features
//!
//! - Environment variable-based authentication
//! - Instance principal authentication support
//! - Email Delivery service support
//! - Async I/O (tokio)
//!
//! ## Quick Start
//!
//! ```no_run
//! use oci_api::Oci;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment variables
//!     let oci = Oci::from_env()?;
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
pub use client::{AuthMode, Oci, OciBuilder};
pub use error::{Error, Result};

// Re-export email module to allow `oci_api::email::*` (without `services`)
pub use services::email;
pub use services::keys;
pub use services::object_storage;
pub use services::vault;

// Re-export async_trait for downstream mock implementations
pub use async_trait::async_trait;
