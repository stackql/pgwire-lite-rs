// tests/integration.rs

use colorize::AnsiColor;
use libpq_sys::ExecStatusType;
use pgwire_lite::{PgwireLite, Value, QueryResult};

fn print_heading(title: &str) {
    let title_owned = title.to_string(); // Convert &str to String
    println!("{}", title_owned.blue().bold());
}

// Pretty print a row with formatting
fn print_row(row: &std::collections::HashMap<String, Value>, index: usize) {
    if index == 0 {
        // Print header
        println!("Row {}: {{", index);
    } else {
        println!("\nRow {}: {{", index);
    }

    for (key, value) in row {
        println!(
            "  {}: {}",
            key.clone().green(),
            format!("{}", value).yellow()
        );
    }
    println!("}}");
}

fn execute_query_with_assertions(conn: &PgwireLite, query: &str, expected_assertions: QueryAssertions) -> bool {
    println!("\nExecuting query: {}", query);
    
    match conn.query(query) {
        Ok(result) => {
            println!();
            println!("Elapsed time: {} ms", result.elapsed_time_ms);
            println!("Result status: {:?}", result.status);
            println!(
                "{} columns, {} rows, {} notices",
                result.col_count, result.row_count, result.notice_count
            );

            if !result.column_names.is_empty() {
                println!("Column names: {:?}", result.column_names);
            }

            if !result.rows.is_empty() {
                println!("Data:");
                for (i, row) in result.rows.iter().enumerate() {
                    print_row(row, i);
                }
            }

            if !result.notices.is_empty() {
                println!("Notices (detail):");
                for notice in result.notices.iter() {
                    if let Some(detail) = notice.fields.get("detail") {
                        println!("{}", detail);
                    }
                }
            }
            println!();
            
            // Run assertions on the result
            let passed = expected_assertions.assert_result(&result);
            if !passed {
                println!("❌ Assertions failed for query: {}", query);
            } else {
                println!("✅ Assertions passed for query: {}", query);
            }
            passed
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Check if we expected an error
            if expected_assertions.expect_error {
                println!("✅ Expected error received: {}", e);
                if let Some(error_contains) = &expected_assertions.error_contains {
                    let error_str = e.to_string();
                    if !error_str.contains(error_contains) {
                        println!("❌ Error doesn't contain expected text '{}': {}", error_contains, error_str);
                        return false;
                    }
                }
                true
            } else {
                println!("❌ Unexpected error: {}", e);
                false
            }
        }
    }
}

// Define a struct to hold assertions for each query
struct QueryAssertions {
    // For normal query results
    min_elapsed_time_ms: Option<u64>,
    expected_status: Option<ExecStatusType>,
    expected_col_count: Option<i32>,
    min_row_count: Option<i32>,
    expected_column_names: Option<Vec<String>>,
    expected_values: Option<Vec<(String, String)>>, // (column, expected value)
    expected_notice_count: Option<usize>,
    
    // For error cases
    expect_error: bool,
    error_contains: Option<String>,
}

impl QueryAssertions {
    fn assert_result(&self, result: &QueryResult) -> bool {
        let mut passed = true;
        
        // Check elapsed time
        if let Some(min_time) = self.min_elapsed_time_ms {
            if result.elapsed_time_ms < min_time {
                println!("❌ Elapsed time should be at least {} ms, got {} ms", 
                    min_time, result.elapsed_time_ms);
                passed = false;
            }
        }
        
        // Check status
        if let Some(expected_status) = self.expected_status {
            if result.status != expected_status {
                println!("❌ Expected status {:?}, got {:?}", 
                    expected_status, result.status);
                passed = false;
            }
        }
        
        // Check column count
        if let Some(expected_col_count) = self.expected_col_count {
            if result.col_count != expected_col_count {
                println!("❌ Expected {} columns, got {}", 
                    expected_col_count, result.col_count);
                passed = false;
            }
        }
        
        // Check row count
        if let Some(min_row_count) = self.min_row_count {
            if result.row_count < min_row_count {
                println!("❌ Expected at least {} rows, got {}", 
                    min_row_count, result.row_count);
                passed = false;
            }
        }
        
        // Check column names
        if let Some(expected_names) = &self.expected_column_names {
            for name in expected_names {
                if !result.column_names.contains(name) {
                    println!("❌ Missing expected column: {}", name);
                    passed = false;
                }
            }
        }
        
        // Check expected values
        if let Some(expected_values) = &self.expected_values {
            if result.rows.is_empty() {
                println!("❌ Expected values to check but no rows returned");
                passed = false;
            } else {
                let row = &result.rows[0]; // Check first row
                for (col, expected_value) in expected_values {
                    match row.get(col) {
                        Some(val) => {
                            let actual = val.to_string();
                            if actual != *expected_value {
                                println!("❌ For column '{}', expected '{}', got '{}'", 
                                    col, expected_value, actual);
                                passed = false;
                            }
                        },
                        None => {
                            println!("❌ Column '{}' not found in result", col);
                            passed = false;
                        }
                    }
                }
            }
        }
        
        // Check notice count
        if let Some(expected_notice_count) = self.expected_notice_count {
            if result.notice_count != expected_notice_count {
                println!("❌ Expected {} notices, got {}", 
                    expected_notice_count, result.notice_count);
                passed = false;
            }
        }
        
        passed
    }
}

impl Default for QueryAssertions {
    fn default() -> Self {
        QueryAssertions {
            min_elapsed_time_ms: Some(0),
            expected_status: None,
            expected_col_count: None,
            min_row_count: None,
            expected_column_names: None,
            expected_values: None,
            expected_notice_count: None,
            expect_error: false,
            error_contains: None,
        }
    }
}

#[test]
fn test_queries() {
    // Create a single connection to be used for all queries
    let conn = match PgwireLite::new("localhost", 5444, false, "verbose") {
        Ok(conn) => conn,
        Err(e) => {
            panic!("Failed to create connection: {}", e);
        }
    };

    println!();
    println!("libpq version: {}", conn.libpq_version());
    println!("Verbosity set to: {}", conn.verbosity());
    println!();

    // Track overall test status
    let mut all_queries_succeeded = true;

    //
    // registry list example
    //
    print_heading("REGISTRY LIST example");
    all_queries_succeeded &= execute_query_with_assertions(&conn, "REGISTRY LIST aws", QueryAssertions {
        expected_status: Some(ExecStatusType::PGRES_TUPLES_OK),
        expected_col_count: Some(2),
        min_row_count: Some(1),
        expected_column_names: Some(vec!["provider".to_string(), "versions".to_string()]),
        expected_values: Some(vec![("provider".to_string(), "aws".to_string())]),
        ..Default::default()
    });

    //
    // registry pull examples
    //
    print_heading("REGISTRY PULL examples");
    all_queries_succeeded &= execute_query_with_assertions(&conn, "REGISTRY PULL homebrew", QueryAssertions {
        expected_status: Some(ExecStatusType::PGRES_COMMAND_OK),
        ..Default::default()
    });
    
    all_queries_succeeded &= execute_query_with_assertions(&conn, "REGISTRY PULL github", QueryAssertions {
        expected_status: Some(ExecStatusType::PGRES_COMMAND_OK),
        ..Default::default()
    });

    //
    // simple select with one row
    //
    print_heading("Literal SELECT example (one row)");
    all_queries_succeeded &= execute_query_with_assertions(&conn, "SELECT 1 as col_name", QueryAssertions {
        expected_status: Some(ExecStatusType::PGRES_TUPLES_OK),
        expected_col_count: Some(1),
        min_row_count: Some(1),
        expected_column_names: Some(vec!["col_name".to_string()]),
        expected_values: Some(vec![("col_name".to_string(), "1".to_string())]),
        ..Default::default()
    });

    //
    // simple select with no rows
    //
    print_heading("Literal SELECT example (no rows)");
    all_queries_succeeded &= execute_query_with_assertions(&conn, "SELECT 1 as col_name WHERE 1=0", QueryAssertions {
        expected_status: Some(ExecStatusType::PGRES_TUPLES_OK),
        expected_col_count: Some(1),
        min_row_count: Some(0),
        expected_column_names: Some(vec!["col_name".to_string()]),
        ..Default::default()
    });

    //
    // failed command - handle expected error
    //
    print_heading("Failed command example");
    all_queries_succeeded &= execute_query_with_assertions(&conn, "NOTACOMMAND", QueryAssertions {
        expect_error: true,
        error_contains: Some("syntax error".to_string()),
        ..Default::default()
    });

    //
    // stackql provider select, multiple rows
    //
    print_heading("StackQL SELECT example (multiple rows)");
    all_queries_succeeded &= execute_query_with_assertions(
        &conn,
        "SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name IN ('stackql','steampipe')",
        QueryAssertions {
            expected_status: Some(ExecStatusType::PGRES_TUPLES_OK),
            min_row_count: Some(1),
            expected_column_names: Some(vec![
                "formula_name".to_string(),
                "installs_30d".to_string(),
                "installs_90d".to_string(),
                "installs_365d".to_string(),
            ]),
            ..Default::default()
        }
    );

    //
    // stackql provider select, provider error, no rows
    //
    print_heading("StackQL SELECT example with provider error and no rows");
    all_queries_succeeded &= execute_query_with_assertions(
        &conn, 
        "SELECT id, name, description, stargazers_count FROM github.repos.repos WHERE org = 'nonexistent-org'",
        QueryAssertions {
            expected_status: Some(ExecStatusType::PGRES_TUPLES_OK),
            expected_col_count: Some(4),
            min_row_count: Some(0),
            expected_column_names: Some(vec![
                "id".to_string(),
                "name".to_string(),
                "description".to_string(),
                "stargazers_count".to_string(),
            ]),
            expected_notice_count: Some(1),
            ..Default::default()
        }
    );

    //
    // another stackql provider select, should succeed
    //
    print_heading("StackQL SELECT example");
    all_queries_succeeded &= execute_query_with_assertions(
        &conn,
        "SELECT * FROM homebrew.formula.vw_info WHERE formula_name = 'stackql'",
        QueryAssertions {
            expected_status: Some(ExecStatusType::PGRES_TUPLES_OK),
            expected_col_count: Some(8),
            min_row_count: Some(1),
            expected_column_names: Some(vec![
                "latest_version".to_string(),
                "license".to_string(),
                "homepage".to_string(),
                "disabled".to_string(),
                "full_name".to_string(),
                "deprecated".to_string(),
                "generated_date".to_string(),
                "formula_name".to_string(),
            ]),
            expected_values: Some(vec![("full_name".to_string(), "stackql".to_string())]),
            ..Default::default()
        }
    );

    // Assert that all queries succeeded as expected
    assert!(all_queries_succeeded, "One or more queries failed unexpectedly");
    
    println!("All tests completed successfully!");
}