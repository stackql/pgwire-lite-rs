// use std::collections::HashMap;
// use std::ffi::{c_void, CStr};
// use std::sync::Arc;

// use libpq::Connection;
// use libpq_sys::ExecStatusType::{PGRES_COMMAND_OK, PGRES_TUPLES_OK};
// use libpq_sys::{
//     PGContextVisibility, PQclear, PQconsumeInput, PQfname, PQgetResult, PQgetvalue, PQlibVersion,
//     PQnfields, PQntuples, PQreset, PQresultStatus, PQresultVerboseErrorMessage, PQsendQuery,
//     PQsetErrorVerbosity, PQsetNoticeReceiver,
// };

// // Use types from the notices module
// use crate::notices::{notice_receiver, Notice, NoticeStorage, Verbosity};
// use crate::value::Value;

// pub struct PgwireLite {
//     conn: Connection,
//     hostname: String,
//     port: u16,
//     use_tls: bool,
//     verbosity: Verbosity,
//     notices: NoticeStorage,
// }

// #[derive(Debug)]
// pub struct QueryResult {
//     pub rows: Vec<HashMap<String, Value>>,
//     pub column_names: Vec<String>, // Store column names separately
//     pub notices: Vec<Notice>,
// }

// impl PgwireLite {
//     pub fn new(
//         hostname: &str,
//         port: u16,
//         use_tls: bool,
//         verbosity: Option<Verbosity>,
//     ) -> Result<Self, Box<dyn std::error::Error>> {
//         let conn_str = format!(
//             "host={} port={} sslmode={}",
//             hostname,
//             port,
//             if use_tls { "require" } else { "disable" }
//         );

//         // Create a long-lived connection
//         let conn = Connection::new(&conn_str)?;
//         let verbosity_val = verbosity.unwrap_or(Verbosity::Default);
//         let notices = Arc::new(std::sync::Mutex::new(Vec::new()));

//         // Apply the desired verbosity level
//         unsafe {
//             PQsetErrorVerbosity((&conn).into(), verbosity_val.into());
//         }

//         // Set up notice receiver once for the connection
//         let notices_ptr = Arc::into_raw(notices.clone()) as *mut c_void;
//         unsafe {
//             PQsetNoticeReceiver((&conn).into(), Some(notice_receiver), notices_ptr);
//         }

//         Ok(PgwireLite {
//             conn,
//             hostname: hostname.to_string(),
//             port,
//             use_tls,
//             verbosity: verbosity_val,
//             notices,
//         })        
//     }

//     pub fn libpq_version(&self) -> String {
//         let version = unsafe { PQlibVersion() };
//         let major = version / 10000;
//         let minor = (version / 100) % 100;
//         let patch = version % 100;
//         format!("{}.{}.{}", major, minor, patch)
//     }

//     pub fn verbosity(&self) -> String {
//         format!("{:?}", self.verbosity)
//     }

//     pub fn reset_connection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         println!("🔄 Resetting connection...");
    
//         unsafe {
//             PQreset((&self.conn).into());
//         }
        
   
//         // Recreate the connection with the same params
//         let conn_str = format!(
//             "host={} port={} sslmode={}",
//             self.hostname,
//             self.port,
//             if self.use_tls { "require" } else { "disable" }
//         );
    
//         let new_conn = Connection::new(&conn_str)?;
//         self.conn = new_conn;
    
//         // Re-apply the verbosity level
//         unsafe {
//             PQsetErrorVerbosity((&self.conn).into(), self.verbosity.into());
//         }
    
//         // Re-set up notice receiver for the connection
//         let notices_ptr = Arc::into_raw(self.notices.clone()) as *mut c_void;
//         unsafe {
//             PQsetNoticeReceiver((&self.conn).into(), Some(notice_receiver), notices_ptr);
//         }
    
//         println!("✅ Connection successfully reset.");
//         Ok(())
//     }
    
    
//     // Helper method to consume any pending results
//     fn consume_pending_results(&self) {
//         unsafe {
//             // First make sure we've read all data available from the server
//             PQconsumeInput((&self.conn).into());

//             // Then clear any pending results
//             loop {
//                 let result = PQgetResult((&self.conn).into());
//                 if result.is_null() {
//                     break;
//                 }
//                 PQclear(result);
//             }
//         }
//     }

//     pub fn query(&mut self, query: &str) -> Result<QueryResult, Box<dyn std::error::Error>> {

//         // Clear any previous notices
//         if let Ok(mut notices) = self.notices.lock() {
//             notices.clear();
//         }
    
//         // Ensure no pending commands/results exist
//         self.consume_pending_results();
    
//         // add ; to `query` if it doesn't end with one
//         let query = if query.ends_with(';') {
//             query.to_string()
//         } else {
//             format!("{};", query)
//         };
    
//         // Use PQsendQuery instead of PQexec
//         let send_success = unsafe { PQsendQuery((&self.conn).into(), query.as_ptr() as *const i8) };
//         if send_success == 0 {
//             // If send failed, the connection might be in a bad state
//             let error = format!(
//                 "Error: {}",
//                 self.conn.error_message().unwrap_or("Unknown error")
//             );
//             let _ = self.reset_connection();
//             return Err(error.into());
//         }
    
//         // Process the first result
//         let result = unsafe { PQgetResult((&self.conn).into()) };
//         if result.is_null() {
//             // If no result, the connection might be in a bad state
//             let _ = self.reset_connection();
//             return Err("No result returned".into());
//         }
    
//         let status = unsafe { PQresultStatus(result) };
    
//         if status != PGRES_TUPLES_OK && status != PGRES_COMMAND_OK {
//             // Try to get a detailed error message
//             let error_msg_ptr = unsafe {
//                 PQresultVerboseErrorMessage(
//                     result,
//                     self.verbosity.into(),
//                     PGContextVisibility::PQSHOW_CONTEXT_ALWAYS,
//                 )
//             };
    
//             let error_msg = if !error_msg_ptr.is_null() {
//                 // Convert the C string to a Rust string
//                 let msg = unsafe { CStr::from_ptr(error_msg_ptr).to_string_lossy().into_owned() };
//                 // Free the C string allocated by PQresultVerboseErrorMessage
//                 unsafe { libpq_sys::PQfreemem(error_msg_ptr as *mut _) };
//                 msg
//             } else {
//                 // Fallback to the standard connection error message if verbose message is not available
//                 self.conn
//                     .error_message()
//                     .unwrap_or("Unknown error")
//                     .to_string()
//             };
    
//             // Clear the result before returning an error
//             unsafe { PQclear(result) };
    
//             // Clear any pending results
//             self.consume_pending_results();

//             // Full connection reset after any error
//             let _ = self.reset_connection();            
    
//             return Err(format!("{}", error_msg.trim_end()).into());
//         }
    
//         // Get column information
//         let col_count = unsafe { PQnfields(result) };
//         println!("Column count: {}", col_count);
    
//         // Create a vector to store column names
//         let mut column_names = Vec::with_capacity(col_count as usize);
//         for col_index in 0..col_count {
//             let col_name_ptr = unsafe { PQfname(result, col_index) };
//             if !col_name_ptr.is_null() {
//                 let col_name =
//                     unsafe { CStr::from_ptr(col_name_ptr).to_string_lossy().into_owned() };
//                 column_names.push(col_name);
//             } else {
//                 column_names.push(String::from("(unknown)"));
//             }
//         }
    
//         // Create the rows vector
//         let mut rows = Vec::new();
    
//         // Get row data if available
//         if status == PGRES_TUPLES_OK {
//             let row_count = unsafe { PQntuples(result) };
//             println!("Row count: {}", row_count);
    
//             // Process each row
//             for row_index in 0..row_count {
//                 let mut row_data = HashMap::new();
    
//                 // Process each column in the row
//                 for col_index in 0..col_count {
//                     let value_ptr = unsafe { PQgetvalue(result, row_index, col_index) };
//                     let value = if !value_ptr.is_null() {
//                         let string_value =
//                             unsafe { CStr::from_ptr(value_ptr).to_string_lossy().into_owned() };
//                         Value::String(string_value)
//                     } else {
//                         Value::Null
//                     };
    
//                     // Insert value into the row map using the column name as key
//                     row_data.insert(column_names[col_index as usize].clone(), value);
//                 }
    
//                 rows.push(row_data);
//             }
//         }
    
//         // Clean up the result
//         unsafe { PQclear(result) };
    
//         // Check for any remaining results and clear them
//         self.consume_pending_results();
    
//         // Get the notices that were collected during the query
//         let notices = if let Ok(mut lock) = self.notices.lock() {
//             lock.drain(..).collect()
//         } else {
//             Vec::new()
//         };
    
//         Ok(QueryResult { 
//             rows, 
//             column_names, // Store column names separately for zero-row results
//             notices 
//         })
//     }

// }

// impl Drop for PgwireLite {
//     fn drop(&mut self) {
//         // Connection will be automatically cleaned up by libpq::Connection's Drop implementation
//     }
// }

use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::sync::Arc;

use libpq::Connection;
use libpq_sys::ExecStatusType::{PGRES_COMMAND_OK, PGRES_TUPLES_OK};
use libpq_sys::{
    PGContextVisibility, PQclear, PQconsumeInput, PQfname, PQgetResult, PQgetvalue, PQlibVersion,
    PQnfields, PQntuples, PQresultStatus, PQresultVerboseErrorMessage, PQsendQuery,
    PQsetErrorVerbosity, PQsetNoticeReceiver,
};

// Use types from the notices module
use crate::notices::{notice_receiver, Notice, NoticeStorage, Verbosity};
use crate::value::Value;

pub struct PgwireLite {
    hostname: String,
    port: u16,
    use_tls: bool,
    verbosity: Verbosity,
    notices: NoticeStorage,
}

#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, Value>>,
    pub column_names: Vec<String>, // Store column names separately
    pub notices: Vec<Notice>,
}

impl PgwireLite {
    pub fn new(
        hostname: &str,
        port: u16,
        use_tls: bool,
        verbosity: Option<Verbosity>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let verbosity_val = verbosity.unwrap_or(Verbosity::Default);
        let notices = Arc::new(std::sync::Mutex::new(Vec::new()));
        
        Ok(PgwireLite {
            hostname: hostname.to_string(),
            port,
            use_tls,
            verbosity: verbosity_val,
            notices,
        })        
    }

    pub fn libpq_version(&self) -> String {
        let version = unsafe { PQlibVersion() };
        let major = version / 10000;
        let minor = (version / 100) % 100;
        let patch = version % 100;
        format!("{}.{}.{}", major, minor, patch)
    }

    pub fn verbosity(&self) -> String {
        format!("{:?}", self.verbosity)
    }
    
    // Helper method to consume any pending results
    fn consume_pending_results(conn: &Connection) {
        unsafe {
            // First make sure we've read all data available from the server
            PQconsumeInput(conn.into());

            // Then clear any pending results
            loop {
                let result = PQgetResult(conn.into());
                if result.is_null() {
                    break;
                }
                PQclear(result);
            }
        }
    }

    // For each query, create a brand new connection
    pub fn query(&self, query: &str) -> Result<QueryResult, Box<dyn std::error::Error>> {
        // Clear any previous notices
        if let Ok(mut notices) = self.notices.lock() {
            notices.clear();
        }
        
        // Create a connection string with ALL the parameters psql would use
        // This is the key difference - more complete connection parameters
        let conn_str = format!(
            "host={} port={} sslmode={} application_name=pgwire-lite-client connect_timeout=10 client_encoding=UTF8",
            self.hostname,
            self.port,
            if self.use_tls { "require" } else { "disable" }
        );
        
        // Create a fresh connection for this query
        let conn = Connection::new(&conn_str)?;
        
        // Apply the desired verbosity level
        unsafe {
            PQsetErrorVerbosity((&conn).into(), self.verbosity.into());
        }
        
        // Set up notice receiver for the connection
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
   
        // Use PQsendQuery instead of PQexec
        let send_success = unsafe { PQsendQuery((&conn).into(), query.as_ptr() as *const i8) };
        if send_success == 0 {
            // If send failed, return the error
            return Err(format!(
                "Error: {}",
                conn.error_message().unwrap_or("Unknown error")
            ).into());
        }
    
        // Process the first result
        let result = unsafe { PQgetResult((&conn).into()) };
        println!("Result: {:?}", result);

        if result.is_null() {
            return Err("No result returned".into());
        }
    
        let status = unsafe { PQresultStatus(result) };

        println!("Result status: {:?}", status);

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
                conn.error_message()
                    .unwrap_or("Unknown error")
                    .to_string()
            };
    
            // Clear the result before returning an error
            unsafe { 
                println!("Clearing error result at {:p}", result);
                PQclear(result);
            }
    
            // Clear any pending results
            Self::consume_pending_results(&conn);   
    
            return Err(format!("{}", error_msg.trim_end()).into());
        }
    
        // Get column information
        let col_count = unsafe { PQnfields(result) };
        println!("Column count: {}", col_count);
    
        // Create a vector to store column names
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
    
        // Create the rows vector
        let mut rows = Vec::new();
    
        // Get row data if available
        if status == PGRES_TUPLES_OK {
            let row_count = unsafe { PQntuples(result) };
            println!("Row count: {}", row_count);
    
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
    
        // NOW clear the result after we've extracted all the information we need
        unsafe { 
            println!("Clearing result at {:p}", result);
            PQclear(result);
            println!("Result cleared successfully");
        } 
    
        let result_after = unsafe { PQgetResult((&conn).into()) };
        let status_after = unsafe { PQresultStatus(result_after) };

        println!("Result status: {:?}", status);


        // Check for any remaining results and clear them
        Self::consume_pending_results(&conn);
    
        // Get the notices that were collected during the query
        let notices = if let Ok(mut lock) = self.notices.lock() {
            lock.drain(..).collect()
        } else {
            Vec::new()
        };
    
        Ok(QueryResult { 
            rows, 
            column_names, // Store column names separately for zero-row results
            notices 
        })
    }
}