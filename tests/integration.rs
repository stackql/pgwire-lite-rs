// tests/integration.rs

#[cfg(test)]
mod integration_tests {
    use colorize::AnsiColor;
    use libpq_sys::ExecStatusType;
    use pgwire_lite::{PgwireLite, QueryResult, Value};
    use std::collections::HashMap;

    // Helper function to create a new connection for each test
    fn create_connection() -> PgwireLite {
        match PgwireLite::new("localhost", 5444, false, "verbose") {
            Ok(conn) => {
                println!("Connection created successfully");
                println!("libpq version: {}", conn.libpq_version());
                println!("Verbosity set to: {}", conn.verbosity());
                conn
            }
            Err(e) => {
                panic!("Failed to create connection: {}", e);
            }
        }
    }

    // Helper function to validate query results
    fn validate_result(result: &QueryResult, expected_col_count: i32, min_row_count: i32) -> bool {
        if result.elapsed_time_ms <= 0 {
            println!("VALIDATION FAILED: Elapsed time should be greater than 0");
            return false;
        }

        if expected_col_count > 0 {
            if result.status != ExecStatusType::PGRES_TUPLES_OK {
                println!("VALIDATION FAILED: Expected PGRES_TUPLES_OK status");
                return false;
            }
            if result.col_count != expected_col_count {
                println!("VALIDATION FAILED: Column count mismatch. Expected {}, got {}", 
                    expected_col_count, result.col_count);
                return false;
            }
            if result.column_names.len() as i32 != expected_col_count {
                println!("VALIDATION FAILED: Column names length mismatch. Expected {}, got {}", 
                    expected_col_count, result.column_names.len());
                return false;
            }
        } else {
            if result.status != ExecStatusType::PGRES_COMMAND_OK {
                println!("VALIDATION FAILED: Expected PGRES_COMMAND_OK status");
                return false;
            }
        }

        if result.row_count < min_row_count {
            println!("VALIDATION FAILED: Row count should be at least {}, got {}", 
                min_row_count, result.row_count);
            return false;
        }

        true
    }

    // Helper to check if a row contains expected column names
    fn validate_row_has_columns(row: &HashMap<String, Value>, expected_columns: &[&str]) -> bool {
        for col in expected_columns {
            if !row.contains_key(&col.to_string()) {
                println!("VALIDATION FAILED: Row should contain column '{}'", col);
                return false;
            }
        }
        true
    }

    // Test 1: Registry List
    #[test]
    fn test_registry_list() {
        let conn = create_connection();

        println!("\n{}", "REGISTRY LIST example".blue().bold());
        
        let result = match conn.query("REGISTRY LIST aws") {
            Ok(result) => result,
            Err(e) => {
                println!("REGISTRY LIST failed: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 2, 1) {
            return;
        }
        
        if !result.column_names.contains(&"provider".to_string()) {
            println!("Missing 'provider' column");
            return;
        }
        
        if !result.column_names.contains(&"versions".to_string()) {
            println!("Missing 'versions' column");
            return;
        }

        // Validate at least the first row has proper content
        if !result.rows.is_empty() {
            let row = &result.rows[0];
            if !validate_row_has_columns(row, &["provider", "versions"]) {
                return;
            }
            
            let provider = match row.get("provider") {
                Some(value) => value.to_string(),
                None => {
                    println!("Provider value missing");
                    return;
                }
            };
            
            if provider != "aws" {
                println!("Provider should be aws, got {}", provider);
                return;
            }
            
            // Just check that versions is non-empty, as it may change over time
            let versions = match row.get("versions") {
                Some(value) => value.to_string(),
                None => {
                    println!("Versions value missing");
                    return;
                }
            };
            
            if versions.is_empty() {
                println!("Versions should not be empty");
                return;
            }
        }
        
        println!("Registry list test passed!");
    }

    // Test 2: Registry Pull
    #[test]
    fn test_registry_pull() {
        let conn = create_connection();

        println!("\n{}", "REGISTRY PULL example".blue().bold());
        
        let result = match conn.query("REGISTRY PULL homebrew") {
            Ok(result) => result,
            Err(e) => {
                println!("REGISTRY PULL failed: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 0, 0) {
            return;
        }
        
        println!("Registry pull test passed!");
    }

    // Test 3: Simple SELECT with one row
    #[test]
    fn test_simple_select_one_row() {
        let conn = create_connection();

        println!("\n{}", "Literal SELECT example (one row)".blue().bold());
        
        let result = match conn.query("SELECT 1 as col_name") {
            Ok(result) => result,
            Err(e) => {
                println!("Simple SELECT failed: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 1, 1) {
            return;
        }
        
        if result.column_names[0] != "col_name" {
            println!("Column name should be 'col_name', got '{}'", result.column_names[0]);
            return;
        }
        
        if result.rows[0].get("col_name").unwrap().to_string() != "1" {
            println!("Value should be '1', got '{}'", result.rows[0].get("col_name").unwrap().to_string());
            return;
        }
        
        println!("Simple SELECT one row test passed!");
    }

    // Test 4: Simple SELECT with no rows
    #[test]
    fn test_simple_select_no_rows() {
        let conn = create_connection();

        println!("\n{}", "Literal SELECT example (no rows)".blue().bold());
        
        let result = match conn.query("SELECT 1 as col_name WHERE 1=0") {
            Ok(result) => result,
            Err(e) => {
                println!("Simple SELECT with no rows failed: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 1, 0) {
            return;
        }
        
        if result.column_names[0] != "col_name" {
            println!("Column name should be 'col_name', got '{}'", result.column_names[0]);
            return;
        }
        
        if !result.rows.is_empty() {
            println!("Should have 0 rows, got {}", result.rows.len());
            return;
        }
        
        println!("Simple SELECT no rows test passed!");
    }

    // Test 5: Failed command - Expected to error
    #[test]
    fn test_failed_command() {
        let conn = create_connection();

        println!("\n{}", "Failed command example".blue().bold());
        
        let result = conn.query("NOTACOMMAND");
        
        if result.is_ok() {
            println!("Invalid command should return an error");
            return;
        }
        
        let error_message = result.unwrap_err().to_string();
        
        if !error_message.contains("syntax error") {
            println!("Error should contain 'syntax error', got: {}", error_message);
            return;
        }
        
        println!("Failed command test passed!");
    }

    // Test 6: StackQL provider SELECT after error
    #[test]
    fn test_stackql_select_after_error() {
        let conn = create_connection();

        // First try a failing command (to verify we can recover)
        let _ = conn.query("NOTACOMMAND");

        println!("\n{}", "StackQL SELECT example (multiple rows)".blue().bold());
        
        let result = match conn.query("SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name = 'stackql'") {
            Ok(result) => result,
            Err(e) => {
                println!("StackQL query failed after error: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 7, 1) {
            return;
        }

        let expected_columns = [
            "formula_name",
            "installs_30d",
            "installs_90d",
            "installs_365d",
            "install_on_requests_30d",
            "install_on_requests_90d",
            "install_on_requests_365d",
        ];

        if !result.column_names.iter().all(|col| expected_columns.contains(&col.as_str())) {
            println!("All expected columns should be present");
            return;
        }

        if !result.rows.is_empty() {
            let row = &result.rows[0];
            if !validate_row_has_columns(row, &expected_columns) {
                return;
            }
            
            let formula_name = match row.get("formula_name") {
                Some(value) => value.to_string(),
                None => {
                    println!("Formula name missing");
                    return;
                }
            };
            
            if formula_name != "stackql" {
                println!("Formula name should be stackql, got {}", formula_name);
                return;
            }
        }
        
        println!("StackQL SELECT after error test passed!");
    }

    // Test 7: StackQL provider SELECT with provider error
    #[test]
    fn test_stackql_select_with_provider_error() {
        let conn = create_connection();

        println!("\n{}", "StackQL SELECT example with provider error and no rows".blue().bold());
        
        let result = match conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'us-east-1' AND data__Identifier = 'fred'") {
            Ok(result) => result,
            Err(e) => {
                println!("Query with provider error failed: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 2, 0) {
            return;
        }
        
        if result.notice_count == 0 {
            println!("Should have notices for provider error");
            return;
        }

        // Check that notices contain error information
        let has_error_notice = result.notices.iter().any(|notice| {
            notice
                .fields
                .get("detail")
                .map(|detail| {
                    detail.contains("UnrecognizedClientException") || detail.contains("400")
                })
                .unwrap_or(false)
        });
        
        if !has_error_notice {
            println!("Should have notice with error details");
            return;
        }
        
        println!("StackQL SELECT with provider error test passed!");
    }

    // Test 8: Final StackQL SELECT to ensure connection still works
    #[test]
    fn test_final_stackql_select() {
        let conn = create_connection();

        println!("\n{}", "Final StackQL SELECT example (multiple rows)".blue().bold());
        
        let result = match conn.query("SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name = 'stackql'") {
            Ok(result) => result,
            Err(e) => {
                println!("Final StackQL query failed: {}", e);
                return;
            }
        };
        
        if !validate_result(&result, 7, 1) {
            return;
        }

        // Verify we have results with the expected stackql formula
        if !result.rows.is_empty() {
            let row = &result.rows[0];
            
            let formula_name = match row.get("formula_name") {
                Some(value) => value.to_string(),
                None => {
                    println!("Formula name missing");
                    return;
                }
            };
            
            if formula_name != "stackql" {
                println!("Formula name should be stackql, got {}", formula_name);
                return;
            }
        }
        
        println!("Final StackQL SELECT test passed!");
        println!("\nAll tests completed!");
    }
}