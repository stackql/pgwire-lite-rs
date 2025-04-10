// tests/integration_mtls.rs
// Integration tests for the PgwireLite library against a server with TLS

use std::env;
use std::path::PathBuf;
mod test_cases;
use pgwire_lite::PgwireLite;

const SERVER_HOST: &str = "localhost";
const SERVER_PORT: u16 = 5444;

// Setup function to create a connection with TLS
fn setup() -> PgwireLite {
    // Set up environment variables for TLS
    let home_dir = env::var("HOME").expect("Could not find HOME environment variable");
    let ssl_dir = PathBuf::from(&home_dir).join("ssl-test");

    env::set_var("PGSSLMODE", "verify-full");
    env::set_var(
        "PGSSLCERT",
        ssl_dir
            .join("client_cert.pem")
            .to_string_lossy()
            .to_string(),
    );
    env::set_var(
        "PGSSLKEY",
        ssl_dir.join("client_key.pem").to_string_lossy().to_string(),
    );
    env::set_var(
        "PGSSLROOTCERT",
        ssl_dir
            .join("server_cert.pem")
            .to_string_lossy()
            .to_string(),
    );

    PgwireLite::new(SERVER_HOST, SERVER_PORT, true, "verbose")
        .expect("Failed to create TLS connection")
}

// Test for setting up a connection without TLS but against a TLS server (should fail)
#[test]
fn test_non_tls_connection_to_tls_server_fails() {
    let conn_result = PgwireLite::new(SERVER_HOST, SERVER_PORT, false, "verbose");
    assert!(conn_result.is_err(), "Non-TLS connection to TLS server should fail");
    
    let err = format!("{}", conn_result.err().unwrap());
    assert!(
        err.contains("connection to server") && 
        (err.contains("closed") || err.contains("terminated abnormally")),
        "Expected connection error, got: {}", err
    );
}

// Run all the shared test cases against a TLS server
#[test]
fn test_libpq_version_tls() {
    let conn = setup();
    test_cases::test_libpq_version(&conn);
}

#[test]
fn test_verbosity_tls() {
    let conn = setup();
    test_cases::test_verbosity(&conn);
}

#[test]
fn test_simple_query_one_row_tls() {
    let conn = setup();
    test_cases::test_simple_query_one_row(&conn);
}

#[test]
fn test_simple_query_no_rows_tls() {
    let conn = setup();
    test_cases::test_simple_query_no_rows(&conn);
}

#[test]
fn test_registry_list_tls() {
    let conn = setup();
    test_cases::test_registry_list(&conn);
}

#[test]
fn test_registry_pull_homebrew_tls() {
    let conn = setup();
    test_cases::test_registry_pull_homebrew(&conn);
}

#[test]
fn test_registry_pull_github_tls() {
    let conn = setup();
    test_cases::test_registry_pull_github(&conn);
}

#[test]
fn test_invalid_command_tls() {
    let conn = setup();
    test_cases::test_invalid_command(&conn);
}

#[test]
fn test_stackql_multiple_rows_tls() {
    let conn = setup();
    test_cases::test_stackql_multiple_rows(&conn);
}

#[test]
fn test_stackql_provider_error_tls() {
    let conn = setup();
    test_cases::test_stackql_provider_error(&conn);
}

#[test]
fn test_stackql_specific_repo_tls() {
    let conn = setup();
    test_cases::test_stackql_specific_repo(&conn);
}