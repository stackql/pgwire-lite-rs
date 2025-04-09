# pgwire-lite-rs

[![Crates.io](https://img.shields.io/crates/v/pgwire-lite.svg)](https://crates.io/crates/pgwire-lite)
[![Documentation](https://docs.rs/pgwire-lite/badge.svg)](https://docs.rs/pgwire-lite)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight PostgreSQL wire protocol client library for Rust.

## Overview

**pgwire-lite** provides a simple, efficient interface for executing queries against PostgreSQL-compatible servers, including StackQL and other wire-protocol compatible services.

This crate was created for applications that need a robust, well-tested connection to PostgreSQL-compatible servers without the overhead of a full-featured ORM.

## Features

- **Simple API** - Straightforward query execution with minimal boilerplate
- **Robust Error Handling** - Comprehensive error information with configurable verbosity
- **Flexible Value Types** - Easy type conversion between PostgreSQL and Rust types
- **SSL/TLS Support** - Secure connections with TLS and certificate validation
- **Detailed Results** - Full access to all aspects of query results including notices
- **libpq Foundation** - Built on the stable, production-tested libpq C library

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pgwire-lite = "0.1.0"
```

## Quick Start

The following example connects to a local [**stackql**](https://github.com/stackql/stackql) server used to query cloud providers.

```rust
use pgwire_lite::PgwireLite;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a StackQL server
    let client = PgwireLite::new("localhost", 5444, false, "verbose")?;
    
    // Pull a provider registry
    client.query("REGISTRY PULL aws")?;
    
    // Query AWS resources using SQL
    let result = client.query(
        "SELECT region, name, instance_type 
         FROM aws.ec2.instances 
         WHERE region = 'us-east-1'"
    )?;
    
    // Process the results
    println!("Found {} EC2 instances:", result.row_count);
    for row in &result.rows {
        println!(
            "Instance: {} ({}), Type: {}", 
            row.get("name").unwrap(),
            row.get("region").unwrap(),
            row.get("instance_type").unwrap()
        );
    }
    
    Ok(())
}
```
## Error Handling

**pgwire-lite** provides detailed error information and configurable verbosity:

```rust
use pgwire_lite::PgwireLite;

fn main() {
    // Set verbosity to "verbose" for maximum error detail
    let client = PgwireLite::new("localhost", 5432, false, "verbose")
        .expect("Failed to create client");
    
    match client.query("SELECT * FROM nonexistent_table") {
        Ok(result) => {
            println!("Query succeeded with {} rows", result.row_count);
        },
        Err(e) => {
            eprintln!("Query failed: {}", e);
            // Detailed error with context, hint, line numbers, etc.
        }
    }
}
```

## TLS/SSL Support

Secure your connections with TLS:

```rust
use pgwire_lite::PgwireLite;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set environment variables for TLS certificates
    env::set_var("PGSSLMODE", "verify-full");
    env::set_var("PGSSLCERT", "/path/to/client-cert.pem");
    env::set_var("PGSSLKEY", "/path/to/client-key.pem");
    env::set_var("PGSSLROOTCERT", "/path/to/server-ca.pem");
    
    // Create a client with TLS enabled
    let client = PgwireLite::new("db.example.com", 5432, true, "default")?;
    
    // Execute queries over a secure connection
    let result = client.query("SELECT 1 as secure_conn_example")?;
    
    Ok(())
}
```

## Documentation

For more detailed usage examples and API documentation, please visit [docs.rs/pgwire-lite](https://docs.rs/pgwire-lite).

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [libpq](https://www.postgresql.org/docs/current/libpq.html) C library
- [stackql](https://github.com/stackql/stackql)