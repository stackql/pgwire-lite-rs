use pgwire_lite::PgwireLite;
use std::env;
use std::path::PathBuf;

fn main() {
    // Get the home directory and properly construct paths
    let home_dir = env::var("HOME").expect("Could not find HOME environment variable");
    let ssl_dir = PathBuf::from(&home_dir).join("ssl-test");
    
    // Set environment variables with absolute paths
    env::set_var("PGSSLMODE", "verify-full");
    env::set_var("PGSSLCERT", ssl_dir.join("client_cert.pem").to_string_lossy().to_string());
    env::set_var("PGSSLKEY", ssl_dir.join("client_key.pem").to_string_lossy().to_string());
    env::set_var("PGSSLROOTCERT", ssl_dir.join("server_cert.pem").to_string_lossy().to_string());
    
    // Create the connection with SSL enabled
    let conn = PgwireLite::new("localhost", 5444, true, "verbose").expect("Failed to create connection");
    
    // Try to execute a simple query
    match conn.query("SELECT 1 as col_name") {
        Ok(result) => println!("Query result (TLS): {:?}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}