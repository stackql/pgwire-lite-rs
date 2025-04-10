// src/connection.rs

use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::sync::Arc;
use std::time::Instant;

use log::debug;

use libpq::Connection;
use libpq_sys::ExecStatusType::{PGRES_COMMAND_OK, PGRES_TUPLES_OK};
use libpq_sys::{
    PGContextVisibility, PQclear, PQconsumeInput, PQfname, PQgetResult, PQgetvalue, PQlibVersion,
    PQnfields, PQntuples, PQresultStatus, PQresultVerboseErrorMessage, PQsendQuery,
    PQsetErrorVerbosity, PQsetNoticeReceiver,
};

use crate::notices::{notice_receiver, Notice, NoticeStorage, Verbosity};
use crate::value::Value;

/// Main client for interacting with PostgreSQL-compatible servers.
///
/// This struct provides the core functionality for establishing connections
/// and executing queries against a PostgreSQL-compatible server.
pub struct PgwireLite {
    hostname: String,
    port: u16,
    use_tls: bool,
    verbosity: Verbosity,
    notices: NoticeStorage,
}

/// Contains the complete result of a query execution.
///
/// This struct provides access to all aspects of a query result,
/// including rows, columns, notices, and execution statistics.
#[derive(Debug)]
pub struct QueryResult {
    /// Rows returned by the query, represented as maps of column names to values.
    pub rows: Vec<HashMap<String, Value>>,

    /// Names of the columns in the result set.
    pub column_names: Vec<String>,

    /// Notices generated during query execution.
    pub notices: Vec<Notice>,

    /// Number of rows in the result set.
    pub row_count: i32,

    /// Number of columns in the result set.
    pub col_count: i32,

    /// Number of notices generated during query execution.
    pub notice_count: usize,

    /// Status of the query execution.
    pub status: libpq_sys::ExecStatusType,

    /// Elapsed time for the query execution in milliseconds.
    pub elapsed_time_ms: u64,
}

// Helper function to safely clear a PGresult and log it
fn clear_pg_result(result: *mut libpq_sys::PGresult) {
    if !result.is_null() {
        unsafe {
            debug!("Clearing PGresult at {:p}", result);
            PQclear(result);
            debug!("PGresult cleared successfully");
        }
    }
}

impl PgwireLite {
    /// Creates a new PgwireLite client with the specified connection parameters.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname or IP address of the PostgreSQL server
    /// * `port` - The port number the PostgreSQL server is listening on
    /// * `use_tls` - Whether to use TLS encryption for the connection
    /// * `verbosity` - Error/notice verbosity level, one of: "terse", "default", "verbose", "sqlstate"
    ///
    /// # Returns
    ///
    /// A Result containing the new PgwireLite instance or an error
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::PgwireLite;
    ///
    /// let client = PgwireLite::new("localhost", 5432, false, "default")
    ///     .expect("Failed to create client");
    /// ```
    pub fn new(
        hostname: &str,
        port: u16,
        use_tls: bool,
        verbosity: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let verbosity_val = match verbosity.to_lowercase().as_str() {
            "default" => Verbosity::Default,
            "verbose" => Verbosity::Verbose,
            "terse" => Verbosity::Terse,
            "sqlstate" => Verbosity::Sqlstate,
            "" => Verbosity::Default,
            _ => Verbosity::Default,
        };

        // Set the log filter level based on verbosity
        match verbosity_val {
            Verbosity::Terse => log::set_max_level(log::LevelFilter::Warn),
            Verbosity::Default => log::set_max_level(log::LevelFilter::Info),
            Verbosity::Verbose => log::set_max_level(log::LevelFilter::Debug),
            Verbosity::Sqlstate => log::set_max_level(log::LevelFilter::Debug),
        }

        let notices = Arc::new(std::sync::Mutex::new(Vec::new()));

        Ok(PgwireLite {
            hostname: hostname.to_string(),
            port,
            use_tls,
            verbosity: verbosity_val,
            notices,
        })
    }

    /// Returns the version of the underlying libpq library.
    ///
    /// # Returns
    ///
    /// A string representing the version in the format "major.minor.patch"
    pub fn libpq_version(&self) -> String {
        let version = unsafe { PQlibVersion() };
        let major = version / 10000;
        let minor = (version / 100) % 100;
        let patch = version % 100;
        format!("{}.{}.{}", major, minor, patch)
    }

    /// Returns the current verbosity setting.
    ///
    /// # Returns
    ///
    /// A string representation of the current verbosity level
    pub fn verbosity(&self) -> String {
        format!("{:?}", self.verbosity)
    }

    // Helper method to consume any pending results
    fn consume_pending_results(conn: &Connection) {
        debug!("Consuming pending results");
        unsafe {
            // First make sure we've read all data available from the server
            PQconsumeInput(conn.into());

            // Then clear any pending results
            loop {
                let result = PQgetResult(conn.into());
                if result.is_null() {
                    break;
                }
                clear_pg_result(result);
            }
        }
    }

    /// Executes a SQL query and returns the results.
    ///
    /// This method creates a fresh connection for each query, executes the query,
    /// and processes the results. It handles all aspects of connection management
    /// and error handling.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query to execute
    ///
    /// # Returns
    ///
    /// A Result containing a QueryResult with the query results or an error
    ///
    /// # Example
    ///
    /// ```
    /// use pgwire_lite::PgwireLite;
    ///
    /// let client = PgwireLite::new("localhost", 5444, false, "default")
    ///     .expect("Failed to create client");
    ///     
    /// let result = client.query("SELECT 1 as value")
    ///     .expect("Query failed");
    ///     
    /// println!("Number of rows: {}", result.row_count);
    /// ```
    pub fn query(&self, query: &str) -> Result<QueryResult, Box<dyn std::error::Error>> {
        // Clear any previous notices
        debug!("Clearing previous notices");
        if let Ok(mut notices) = self.notices.lock() {
            notices.clear();
        }

        let start_time = Instant::now();

        // Create a connection string
        let conn_str = format!(
            "host={} port={} sslmode={} application_name=pgwire-lite-client connect_timeout=10 client_encoding=UTF8",
            self.hostname,
            self.port,
            if self.use_tls { "verify-full" } else { "disable" }
        );
        debug!("Establishing connection using: {}", conn_str);

        // Create a fresh connection for this query
        let conn = Connection::new(&conn_str)?;

        // Apply the desired verbosity level
        debug!("Setting error verbosity to: {:?}", self.verbosity);
        unsafe {
            PQsetErrorVerbosity((&conn).into(), self.verbosity.into());
        }

        // Set up notice receiver for the connection
        debug!("Setting up notice receiver");
        let notices_ptr = Arc::into_raw(self.notices.clone()) as *mut c_void;
        unsafe {
            PQsetNoticeReceiver((&conn).into(), Some(notice_receiver), notices_ptr);
        }

        // add ; to `query` if it doesn't end with one
        let query = if query.ends_with(';') {
            query.to_string()
        } else {
            format!("{};", query)
        };

        // Use PQsendQuery
        debug!("Sending query: {}", query);
        let send_success = unsafe { PQsendQuery((&conn).into(), query.as_ptr() as *const i8) };
        if send_success == 0 {
            // If send failed, return the error
            return Err(
                format!("Error: {}", conn.error_message().unwrap_or("Unknown error")).into(),
            );
        }

        // Process the result
        debug!("Processing the result");
        let result = unsafe { PQgetResult((&conn).into()) };

        if result.is_null() {
            return Err("No result returned".into());
        }

        let status = unsafe { PQresultStatus(result) };

        if status != PGRES_TUPLES_OK && status != PGRES_COMMAND_OK {
            // Try to get a detailed error message
            let error_msg_ptr = unsafe {
                PQresultVerboseErrorMessage(
                    result,
                    self.verbosity.into(),
                    PGContextVisibility::PQSHOW_CONTEXT_ALWAYS,
                )
            };

            let error_msg = if !error_msg_ptr.is_null() {
                // Convert the C string to a Rust string
                let msg = unsafe { CStr::from_ptr(error_msg_ptr).to_string_lossy().into_owned() };
                // Free the C string allocated by PQresultVerboseErrorMessage
                unsafe { libpq_sys::PQfreemem(error_msg_ptr as *mut _) };
                msg
            } else {
                // Fallback to the standard connection error message if verbose message is not available
                conn.error_message().unwrap_or("Unknown error").to_string()
            };

            clear_pg_result(result);

            // Clear any pending results
            Self::consume_pending_results(&conn);

            // return Err(format!("{}", error_msg.trim_end()).into());
            return Err(error_msg.trim_end().to_string().into());
        }

        // Get column information
        debug!("Getting column count");
        let col_count = unsafe { PQnfields(result) };

        // Create a vector to store column names
        debug!("Getting column names");
        let mut column_names = Vec::with_capacity(col_count as usize);
        for col_index in 0..col_count {
            let col_name_ptr = unsafe { PQfname(result, col_index) };
            if !col_name_ptr.is_null() {
                let col_name =
                    unsafe { CStr::from_ptr(col_name_ptr).to_string_lossy().into_owned() };
                column_names.push(col_name);
            } else {
                column_names.push(String::from("(unknown)"));
            }
        }

        // Initialize row_count here
        debug!("Getting row count");
        let row_count = if status == PGRES_TUPLES_OK {
            unsafe { PQntuples(result) }
        } else {
            0
        };

        // Create the rows vector
        let mut rows = Vec::new();

        // Get row data if available
        if status == PGRES_TUPLES_OK {
            debug!("Processing rows");

            // Process each row
            for row_index in 0..row_count {
                let mut row_data = HashMap::new();

                // Process each column in the row
                for col_index in 0..col_count {
                    let value_ptr = unsafe { PQgetvalue(result, row_index, col_index) };
                    let value = if !value_ptr.is_null() {
                        let string_value =
                            unsafe { CStr::from_ptr(value_ptr).to_string_lossy().into_owned() };
                        Value::String(string_value)
                    } else {
                        Value::Null
                    };

                    // Insert value into the row map using the column name as key
                    row_data.insert(column_names[col_index as usize].clone(), value);
                }

                rows.push(row_data);
            }
        }
        debug!("Rows processed: {}", rows.len());

        clear_pg_result(result);

        // Check for any remaining results and clear them
        Self::consume_pending_results(&conn);

        // Get the notices that were collected during the query
        debug!("Collecting notices");
        let notices = if let Ok(mut lock) = self.notices.lock() {
            lock.drain(..).collect()
        } else {
            Vec::new()
        };
        let notice_count = notices.len();

        let elapsed_time_ms = start_time.elapsed().as_millis() as u64;

        drop(conn);

        Ok(QueryResult {
            rows,
            column_names,
            notices,
            row_count,
            col_count,
            notice_count,
            status,
            elapsed_time_ms,
        })
    }
}
