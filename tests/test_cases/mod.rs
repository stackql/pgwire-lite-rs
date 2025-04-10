// tests/test_cases/mod.rs
// Shared test cases for both regular and mTLS integration tests

use pgwire_lite::PgwireLite;

/// Tests the library version function
pub fn test_libpq_version(conn: &PgwireLite) {
    let version = conn.libpq_version();
    assert!(!version.is_empty(), "libpq version should not be empty");
    assert!(version.contains("16.0"), "Expected libpq version 16.0.x"); // This might need adjustment based on your environment
}

/// Tests that verbosity setting works
pub fn test_verbosity(conn: &PgwireLite) {
    let verbosity = conn.verbosity();
    assert_eq!(verbosity, "Verbose", "Verbosity should be set to Verbose");
}

/// Tests a simple query with one result row
pub fn test_simple_query_one_row(conn: &PgwireLite) {
    let result = conn.query("SELECT 1 as col_name").expect("Query should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_TUPLES_OK");
    assert_eq!(result.row_count, 1);
    assert_eq!(result.col_count, 1);
    assert_eq!(result.column_names, vec!["col_name"]);
    assert_eq!(result.rows.len(), 1);
    
    let row = &result.rows[0];
    assert!(row.contains_key("col_name"));
    assert_eq!(row.get("col_name").unwrap().to_string(), "1");
}

/// Tests a simple query with no results
pub fn test_simple_query_no_rows(conn: &PgwireLite) {
    let result = conn.query("SELECT 1 as col_name WHERE 1=0").expect("Query should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_TUPLES_OK");
    assert_eq!(result.row_count, 0);
    assert_eq!(result.col_count, 1);
    assert_eq!(result.column_names, vec!["col_name"]);
    assert_eq!(result.rows.len(), 0);
}

/// Tests registry list functionality
pub fn test_registry_list(conn: &PgwireLite) {
    let result = conn.query("REGISTRY LIST aws").expect("Registry list should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_TUPLES_OK");
    assert_eq!(result.row_count, 1);
    assert_eq!(result.col_count, 2);
    assert_eq!(result.column_names, vec!["provider", "versions"]);
    assert_eq!(result.rows.len(), 1);
    
    let row = &result.rows[0];
    assert!(row.contains_key("provider"));
    assert!(row.contains_key("versions"));
    assert_eq!(row.get("provider").unwrap().to_string(), "aws");
    // Don't assert on specific versions as they might change
    assert!(!row.get("versions").unwrap().to_string().is_empty());
}

/// Tests registry pull functionality for homebrew
pub fn test_registry_pull_homebrew(conn: &PgwireLite) {
    let result = conn.query("REGISTRY PULL homebrew").expect("Registry pull should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_COMMAND_OK");
    assert_eq!(result.row_count, 0);
    assert_eq!(result.col_count, 0);
    assert_eq!(result.rows.len(), 0);
}

/// Tests registry pull functionality for github
pub fn test_registry_pull_github(conn: &PgwireLite) {
    let result = conn.query("REGISTRY PULL github").expect("Registry pull should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_COMMAND_OK");
    assert_eq!(result.row_count, 0);
    assert_eq!(result.col_count, 0);
    assert_eq!(result.rows.len(), 0);
}

/// Tests query with expected error
pub fn test_invalid_command(conn: &PgwireLite) {
    let result = conn.query("NOTACOMMAND");
    
    assert!(result.is_err(), "Invalid command should return an error");
    let err = format!("{}", result.err().unwrap());
    assert!(err.contains("ERROR"), "Error message should contain 'ERROR'");
    assert!(err.contains("syntax error"), "Error should mention syntax error");
}

/// Tests a StackQL query that returns multiple rows
pub fn test_stackql_multiple_rows(conn: &PgwireLite) {
    let result = conn.query(
        "SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name IN ('stackql','steampipe')"
    ).expect("StackQL query should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_TUPLES_OK");
    assert_eq!(result.row_count, 2);
    assert_eq!(result.col_count, 7);
    assert_eq!(result.rows.len(), 2);
    
    // Check that all expected columns are present
    let expected_columns = vec![
        "formula_name", "installs_30d", "installs_90d", "installs_365d", 
        "install_on_requests_30d", "install_on_requests_90d", "install_on_requests_365d"
    ];
    
    for col in expected_columns {
        assert!(result.column_names.contains(&col.to_string()), "Column {} should be present", col);
    }
    
    // Check that both stackql and steampipe are in the results
    let formula_names: Vec<String> = result.rows.iter()
        .map(|row| row.get("formula_name").unwrap().to_string())
        .collect();
    
    assert!(formula_names.contains(&"stackql".to_string()), "Results should include stackql");
    assert!(formula_names.contains(&"steampipe".to_string()), "Results should include steampipe");
}

/// Tests a StackQL query with a provider error (nonexistent org)
pub fn test_stackql_provider_error(conn: &PgwireLite) {
    let result = conn.query(
        "SELECT id, name, description, stargazers_count FROM github.repos.repos WHERE org = 'nonexistent-org'"
    ).expect("Query should execute without error, even with provider error");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_TUPLES_OK");
    assert_eq!(result.row_count, 0);
    assert_eq!(result.notice_count, 1);
    assert_eq!(result.rows.len(), 0);
    
    // Check that we got a notice with a 404 error
    assert!(!result.notices.is_empty(), "Should have at least one notice");
    let notice = &result.notices[0];
    let detail = notice.fields.get("detail").expect("Notice should have a detail field");
    assert!(detail.contains("404"), "Notice should mention 404 status code");
}

/// Tests a successful StackQL query for a specific repo
pub fn test_stackql_specific_repo(conn: &PgwireLite) {
    let result = conn.query(
        "SELECT id, name, description, stargazers_count FROM github.repos.repos WHERE org = 'stackql' AND name = 'stackql'"
    ).expect("StackQL query should succeed");
    
    assert_eq!(format!("{:?}", result.status), "PGRES_TUPLES_OK");
    assert_eq!(result.row_count, 1);
    assert_eq!(result.col_count, 4);
    assert_eq!(result.rows.len(), 1);
    
    let row = &result.rows[0];
    assert_eq!(row.get("name").unwrap().to_string(), "stackql");
    assert!(row.get("description").unwrap().to_string().contains("SQL"), "Description should mention SQL");
    
    // Don't assert on specific star count as it might change
    let stars = row.get("stargazers_count").unwrap().to_string();
    assert!(!stars.is_empty(), "Should have stargazers count");
    assert!(stars.parse::<i32>().is_ok(), "Stars should be a number");
}