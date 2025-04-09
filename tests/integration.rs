#[cfg(test)]
mod integration_tests {
    use colorize::AnsiColor;
    use pgwire_lite::{PgwireLite, Value, QueryResult};
    use std::collections::HashMap;
    use std::sync::{Once, Mutex, Arc};
    use libpq_sys::ExecStatusType;
    use lazy_static::lazy_static;

    // Setup static connection that will be shared across tests
    lazy_static! {
        static ref CONNECTION: Arc<Mutex<Option<PgwireLite>>> = Arc::new(Mutex::new(None));
        static ref INIT: Once = Once::new();
    }

    // Initialize connection once for all tests
    fn setup_connection() -> Arc<Mutex<Option<PgwireLite>>> {
        INIT.call_once(|| {
            let conn = PgwireLite::new("localhost", 5444, false, "verbose").expect("Failed to create connection");
            println!("\nConnection created successfully");
            println!("libpq version: {}", conn.libpq_version());
            println!("Verbosity set to: {}", conn.verbosity());
            
            *CONNECTION.lock().unwrap() = Some(conn);
        });
        
        CONNECTION.clone()
    }

    // Helper function to validate query results
    fn validate_result(result: &QueryResult, expected_col_count: i32, min_row_count: i32) {
        assert!(result.elapsed_time_ms > 0, "Elapsed time should be greater than 0");
        
        if expected_col_count > 0 {
            assert_eq!(result.status, ExecStatusType::PGRES_TUPLES_OK);
            assert_eq!(result.col_count, expected_col_count, "Column count mismatch");
            assert_eq!(result.column_names.len() as i32, expected_col_count, "Column names length mismatch");
        } else {
            assert_eq!(result.status, ExecStatusType::PGRES_COMMAND_OK);
        }
        
        assert!(result.row_count >= min_row_count, "Row count should be at least {}", min_row_count);
    }

    // Helper to check if a row contains expected column names
    fn validate_row_has_columns(row: &HashMap<String, Value>, expected_columns: &[&str]) {
        for col in expected_columns {
            assert!(row.contains_key(&col.to_string()), "Row should contain column '{}'", col);
        }
    }

    // Test 1: Registry List
    #[test]
    fn test_registry_list() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "REGISTRY LIST example".blue().bold());
        let result = conn.query("REGISTRY LIST aws").expect("REGISTRY LIST should succeed");
        validate_result(&result, 2, 1);
        assert!(result.column_names.contains(&"provider".to_string()));
        assert!(result.column_names.contains(&"versions".to_string()));
        
        // Validate at least the first row has proper content
        if !result.rows.is_empty() {
            let row = &result.rows[0];
            validate_row_has_columns(row, &["provider", "versions"]);
            assert_eq!(row.get("provider").unwrap().to_string(), "aws", "Provider should be aws");
            // Just check that versions is non-empty, as it may change over time
            assert!(!row.get("versions").unwrap().to_string().is_empty(), "Versions should not be empty");
        }
    }

    // Test 2: Registry Pull
    #[test]
    fn test_registry_pull() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "REGISTRY PULL example".blue().bold());
        let result = conn.query("REGISTRY PULL homebrew").expect("REGISTRY PULL should succeed");
        validate_result(&result, 0, 0);
    }

    // Test 3: Simple SELECT with one row
    #[test]
    fn test_simple_select_one_row() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "Literal SELECT example (one row)".blue().bold());
        let result = conn.query("SELECT 1 as col_name").expect("Simple SELECT should succeed");
        validate_result(&result, 1, 1);
        assert_eq!(result.column_names[0], "col_name");
        assert_eq!(result.rows[0].get("col_name").unwrap().to_string(), "1");
    }

    // Test 4: Simple SELECT with no rows
    #[test]
    fn test_simple_select_no_rows() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "Literal SELECT example (no rows)".blue().bold());
        let result = conn.query("SELECT 1 as col_name WHERE 1=0").expect("Simple SELECT with no rows should succeed");
        validate_result(&result, 1, 0);
        assert_eq!(result.column_names[0], "col_name");
        assert!(result.rows.is_empty());
    }

    // Test 5: Failed command - Expected to error
    #[test]
    fn test_failed_command() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "Failed command example".blue().bold());
        let result = conn.query("NOTACOMMAND");
        assert!(result.is_err(), "Invalid command should return an error");
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("syntax error"), "Error should contain 'syntax error'");
    }

    // Test 6: StackQL provider SELECT after error
    #[test]
    fn test_stackql_select_after_error() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        // First try a failing command (to verify we can recover)
        let _ = conn.query("NOTACOMMAND");
        
        println!("\n{}", "StackQL SELECT example (multiple rows)".blue().bold());
        let result = conn.query("SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name = 'stackql'")
            .expect("StackQL query should succeed after error");
        validate_result(&result, 7, 1);
        
        let expected_columns = [
            "formula_name", "installs_30d", "installs_90d", "installs_365d", 
            "install_on_requests_30d", "install_on_requests_90d", "install_on_requests_365d"
        ];
        
        assert!(result.column_names.iter().all(|col| expected_columns.contains(&col.as_str())),
            "All expected columns should be present");
            
        if !result.rows.is_empty() {
            let row = &result.rows[0];
            validate_row_has_columns(row, &expected_columns);
            assert_eq!(row.get("formula_name").unwrap().to_string(), "stackql", "Formula name should be stackql");
        }
    }

    // Test 7: StackQL provider SELECT with provider error
    #[test]
    fn test_stackql_select_with_provider_error() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "StackQL SELECT example with provider error and no rows".blue().bold());
        let result = conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'us-east-1' AND data__Identifier = 'fred'")
            .expect("Query with provider error should succeed at connection level");
        validate_result(&result, 2, 0);
        assert!(result.notice_count > 0, "Should have notices for provider error");
        
        // Check that notices contain error information
        let has_error_notice = result.notices.iter().any(|notice| {
            notice.fields.get("detail")
                .map(|detail| detail.contains("UnrecognizedClientException") || detail.contains("400"))
                .unwrap_or(false)
        });
        assert!(has_error_notice, "Should have notice with error details");
    }

    // Test 8: Final StackQL SELECT to ensure connection still works
    #[test]
    fn test_final_stackql_select() {
        let conn_mutex = setup_connection();
        let conn_guard = conn_mutex.lock().unwrap();
        let conn = conn_guard.as_ref().unwrap();
        
        println!("\n{}", "Final StackQL SELECT example (multiple rows)".blue().bold());
        let result = conn.query("SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name = 'stackql'")
            .expect("Final StackQL query should succeed");
        validate_result(&result, 7, 1);
        
        // Verify we have results with the expected stackql formula
        if !result.rows.is_empty() {
            let row = &result.rows[0];
            assert_eq!(row.get("formula_name").unwrap().to_string(), "stackql", "Formula name should be stackql");
        }
        
        println!("\nAll tests completed successfully!");
    }
}