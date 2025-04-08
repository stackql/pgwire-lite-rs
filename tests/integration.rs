// use pgwire_lite::PgwireLite;

// #[test]
// fn test_query_without_tls() {
//     let conn = PgwireLite::new("localhost", 5444, false);
//     let result = conn.query("SELECT 1;");
//     assert!(result.is_ok());
//     let data = result.unwrap().data;
//     assert_eq!(data[0][0], "1");
// }

// #[test]
// #[ignore = "reason: TLS not set up in test environment"]
// fn test_query_with_tls() {
//     std::env::set_var("PGSSLCERT", "path/to/client.crt");
//     std::env::set_var("PGSSLKEY", "path/to/client.key");
//     std::env::set_var("PGSSLROOTCERT", "path/to/root.crt");

//     let conn = PgwireLite::new("localhost", 5444, true);
//     let result = conn.query("SELECT 1;");
//     assert!(result.is_ok());
//     let data = result.unwrap().data;
//     assert_eq!(data[0][0], "1");
// }

// #[cfg(test)]
// mod tests {
//     use pgwire_lite::PgwireLite;

//     #[test]
//     fn test_query_with_data() {
//         let conn = PgwireLite::new("localhost", 5444, false);
//         let result = conn.query("SELECT 'fred' AS name;").unwrap();
//         assert_eq!(result.data.len(), 2); // Headers + Row
//         assert_eq!(result.data[0], vec!["name"]);
//         assert_eq!(result.data[1], vec!["fred"]);
//     }

//     #[test]
//     fn test_query_with_no_rows() {
//         let conn = PgwireLite::new("localhost", 5444, false);
//         let result = conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'invalid-region';").unwrap();

//         // Check that headers are present even if there are no rows
//         assert_eq!(result.data.len(), 1); // Only headers
//         assert!(result.data[0].contains(&"region".to_string()));
//         assert!(result.data[0].contains(&"function_name".to_string()));
//     }

//     #[test]
//     fn test_query_with_notice() {
//         let conn = PgwireLite::new("localhost", 5444, false);
//         let result = conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'us-east-1' AND data__Identifier = 'fred';").unwrap();

//         assert!(result.notices.len() > 0);
//         assert!(result.notices[0].fields.contains_key("severity"));
//         assert!(result.notices[0].fields.contains_key("message"));
//     }
// }

// #[cfg(test)]
// mod tests {
//     use pgwire_lite::{PgwireLite, Verbosity};

//     #[test]
//     fn test_query_with_data() {
//         let conn = PgwireLite::new("localhost", 5444, false, Some(Verbosity::Verbose)).unwrap();
//         let result = conn.query("SELECT 'fred' AS name;").unwrap();

//         // Check that we have 1 row
//         assert_eq!(result.rows.len(), 1);

//         // Check that the row contains the expected column and value
//         assert!(result.rows[0].contains_key("name"));
//         assert_eq!(result.rows[0]["name"].as_str().unwrap(), "fred");
//     }

//     #[test]
//     fn test_query_with_no_rows() {
//         let conn = PgwireLite::new("localhost", 5444, false, None).unwrap();
//         let result = conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'invalid-region';").unwrap();

//         // Check that there are no rows but the query succeeded
//         assert_eq!(result.rows.len(), 0);
//     }

//     #[test]
//     fn test_query_with_notice() {
//         let conn = PgwireLite::new("localhost", 5444, false, None).unwrap();
//         let result = conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'us-east-1' AND data__Identifier = 'fred';").unwrap();

//         assert!(result.notices.len() > 0);
//         assert!(result.notices[0].fields.contains_key("severity"));
//         assert!(result.notices[0].fields.contains_key("message"));
//     }
// }

// test_query.rs or any other test file name
#[cfg(test)]
mod test_query_with_data {
    use pgwire_lite::{PgwireLite, Verbosity};

    #[test]
    fn runs_successfully() {
        // Create a fresh connection just for this test
        let conn = PgwireLite::new("localhost", 5444, false, Some(Verbosity::Verbose)).unwrap();
        
        // Reset transaction state if needed
        conn.query("ROLLBACK").ok();
        
        // Run the actual test query
        let result = conn.query("SELECT 'fred' AS name").unwrap();
        
        // Check that we have 1 row
        assert_eq!(result.rows.len(), 1);
        
        // Check that column names are present
        assert_eq!(result.column_names.len(), 1);
        assert_eq!(result.column_names[0], "name");
        
        // Check that the row contains the expected column and value
        assert!(result.rows[0].contains_key("name"));
        assert_eq!(result.rows[0]["name"].as_str().unwrap(), "fred");

        // Connection will be cleaned up automatically when it goes out of scope
    }
}