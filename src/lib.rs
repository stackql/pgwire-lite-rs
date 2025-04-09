// src/lib.rs

//! # PgWire Lite
//!
//! A lightweight PostgreSQL wire protocol client library built on top of libpq.
//!
//! This crate provides a simple, efficient interface for executing queries against
//! PostgreSQL-compatible servers, including StackQL and similar services.
//!
//! ## Features
//!
//! - Built on the robust libpq C library
//! - Simple API for query execution
//! - Comprehensive error handling with configurable verbosity
//! - Support for SSL/TLS connections
//! - Detailed query result information including notices
//!
//! ## Example
//!
//! ```rust
//! use pgwire_lite::PgwireLite;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a connection to PGWire protocol server (like a local StackQL server)
//!     let conn = PgwireLite::new("localhost", 5444, false, "default")?;
//!     
//!     // Execute a multi-line query using a raw string
//!     let result = conn.query(r#"
//!         SELECT region, instance_type, COUNT(*) as num_instances
//!         FROM aws.ec2.instances
//!         WHERE region = 'us-east-1'
//!         GROUP BY instance_type
//!     "#)?;
//!     
//!     // Process the result
//!     println!("Instance types in us-east-1:");
//!     for row in &result.rows {
//!         println!(
//!             "  {}: {} instances",
//!             row.get("instance_type").unwrap(),
//!             row.get("num_instances").unwrap()
//!         );
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod connection;
pub mod notices;
pub mod value;

// Re-export types from the connection module
pub use connection::{PgwireLite, QueryResult};

// Re-export types from the notices module
pub use notices::{Notice, Verbosity};

// Re-export the Value type
pub use value::Value;
