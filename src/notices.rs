use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::sync::{Arc, Mutex};

use libpq_sys::{
    PGVerbosity, PGresult, PQresultErrorField, PG_DIAG_MESSAGE_DETAIL, PG_DIAG_MESSAGE_HINT,
    PG_DIAG_MESSAGE_PRIMARY, PG_DIAG_SEVERITY, PG_DIAG_SOURCE_FILE, PG_DIAG_SOURCE_FUNCTION,
    PG_DIAG_SOURCE_LINE, PG_DIAG_SQLSTATE,
};

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

#[derive(Debug, Clone)]
pub struct Notice {
    pub fields: HashMap<&'static str, String>,
}

pub type NoticeStorage = Arc<Mutex<Vec<Notice>>>;

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
            (PG_DIAG_SQLSTATE as i32, "sqlstate"),
            (PG_DIAG_MESSAGE_PRIMARY as i32, "message"),
            (PG_DIAG_MESSAGE_DETAIL as i32, "detail"),
            (PG_DIAG_MESSAGE_HINT as i32, "hint"),
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
