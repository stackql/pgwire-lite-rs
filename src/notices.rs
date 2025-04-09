// src/notices.rs

use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::sync::{Arc, Mutex};

use libpq_sys::{
    PGVerbosity, PGresult, PQresultErrorField, PG_DIAG_COLUMN_NAME, PG_DIAG_CONSTRAINT_NAME,
    PG_DIAG_CONTEXT, PG_DIAG_DATATYPE_NAME, PG_DIAG_INTERNAL_POSITION, PG_DIAG_INTERNAL_QUERY,
    PG_DIAG_MESSAGE_DETAIL, PG_DIAG_MESSAGE_HINT, PG_DIAG_MESSAGE_PRIMARY, PG_DIAG_SCHEMA_NAME,
    PG_DIAG_SEVERITY, PG_DIAG_SEVERITY_NONLOCALIZED, PG_DIAG_SOURCE_FILE, PG_DIAG_SOURCE_FUNCTION,
    PG_DIAG_SOURCE_LINE, PG_DIAG_SQLSTATE, PG_DIAG_STATEMENT_POSITION, PG_DIAG_TABLE_NAME,
};

/// Error/notice verbosity level for PostgreSQL connections.
///
/// Controls the amount of detail included in error and notice messages.
#[derive(Debug, Clone, Copy)]
pub enum Verbosity {
    Terse,
    Default,
    Verbose,
    Sqlstate,
}

impl From<Verbosity> for PGVerbosity {
    fn from(verbosity: Verbosity) -> Self {
        match verbosity {
            Verbosity::Terse => PGVerbosity::PQERRORS_TERSE,
            Verbosity::Default => PGVerbosity::PQERRORS_DEFAULT,
            Verbosity::Verbose => PGVerbosity::PQERRORS_VERBOSE,
            Verbosity::Sqlstate => PGVerbosity::PQERRORS_SQLSTATE,
        }
    }
}

/// Represents a notice or warning message from PostgreSQL.
///
/// Notices are informational messages that don't cause a query to fail
/// but provide important context about the execution.
#[derive(Debug, Clone)]
pub struct Notice {
    /// A map of field identifiers to their values
    ///
    /// Common fields include:
    /// - "severity": The severity level (e.g., "NOTICE", "WARNING")
    /// - "message": The primary message text
    /// - "detail": Additional detail about the problem
    /// - "hint": Suggestion on how to fix the problem    
    pub fields: HashMap<&'static str, String>,
}

/// Thread-safe storage for collected notices
pub type NoticeStorage = Arc<Mutex<Vec<Notice>>>;

/// C callback function that receives notices from PostgreSQL.
///
/// This function is called by libpq whenever a notice is generated.
/// It extracts the relevant fields based on the verbosity level and
/// stores them in the shared notice storage.
///
/// # Safety
///
/// This function is called directly by C code and must follow C calling conventions.
/// It carefully handles null pointers and performs proper memory management.
pub extern "C" fn notice_receiver(arg: *mut c_void, result: *const PGresult) {
    if result.is_null() || arg.is_null() {
        return;
    }

    let shared_notices = unsafe { &*(arg as *const Mutex<Vec<Notice>>) };

    // Retrieve verbosity level from the connection
    let verbosity = match shared_notices.lock() {
        Ok(notices) => notices
            .get(0)
            .map(|_| Verbosity::Verbose)
            .unwrap_or(Verbosity::Default),
        Err(_) => Verbosity::Default,
    };

    let field_kinds: Vec<(i32, &'static str)> = match verbosity {
        Verbosity::Terse => vec![
            (PG_DIAG_SEVERITY as i32, "severity"),
            (PG_DIAG_MESSAGE_PRIMARY as i32, "message"),
            (PG_DIAG_SQLSTATE as i32, "sqlstate"),
        ],
        Verbosity::Default => vec![
            (PG_DIAG_SEVERITY as i32, "severity"),
            (PG_DIAG_SQLSTATE as i32, "sqlstate"),
            (PG_DIAG_MESSAGE_PRIMARY as i32, "message"),
            (PG_DIAG_MESSAGE_DETAIL as i32, "detail"),
            (PG_DIAG_MESSAGE_HINT as i32, "hint"),
        ],
        Verbosity::Verbose => vec![
            (PG_DIAG_SEVERITY as i32, "severity"),
            (
                PG_DIAG_SEVERITY_NONLOCALIZED as i32,
                "severity_nonlocalized",
            ),
            (PG_DIAG_SQLSTATE as i32, "sqlstate"),
            (PG_DIAG_MESSAGE_PRIMARY as i32, "message"),
            (PG_DIAG_MESSAGE_DETAIL as i32, "detail"),
            (PG_DIAG_MESSAGE_HINT as i32, "hint"),
            (PG_DIAG_STATEMENT_POSITION as i32, "statement_position"),
            (PG_DIAG_INTERNAL_POSITION as i32, "internal_position"),
            (PG_DIAG_INTERNAL_QUERY as i32, "internal_query"),
            (PG_DIAG_CONTEXT as i32, "context"),
            (PG_DIAG_SCHEMA_NAME as i32, "schema_name"),
            (PG_DIAG_TABLE_NAME as i32, "table_name"),
            (PG_DIAG_COLUMN_NAME as i32, "column_name"),
            (PG_DIAG_DATATYPE_NAME as i32, "datatype_name"),
            (PG_DIAG_CONSTRAINT_NAME as i32, "constraint_name"),
            (PG_DIAG_SOURCE_FILE as i32, "source_file"),
            (PG_DIAG_SOURCE_LINE as i32, "source_line"),
            (PG_DIAG_SOURCE_FUNCTION as i32, "source_function"),
        ],
        Verbosity::Sqlstate => vec![
            (PG_DIAG_SEVERITY as i32, "severity"),
            (PG_DIAG_SQLSTATE as i32, "sqlstate"),
        ],
    };

    let mut notice = Notice {
        fields: HashMap::new(),
    };

    for (code, label) in &field_kinds {
        let val_ptr = unsafe { PQresultErrorField(result, *code) };
        if !val_ptr.is_null() {
            let val = unsafe { CStr::from_ptr(val_ptr) }
                .to_string_lossy()
                .into_owned();
            notice.fields.insert(*label, val);
        }
    }

    if let Ok(mut vec) = shared_notices.lock() {
        vec.push(notice);
    }
}
