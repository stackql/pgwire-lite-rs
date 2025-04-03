use postgres::{Client, NoTls};
use postgres::SimpleQueryMessage::{Row, CommandComplete};

use libpq;
use libpq_sys;

use std::collections::HashMap;
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use libpq_sys::{
    PGconn, PGresult, PQresultErrorField,
    PG_DIAG_SEVERITY, PG_DIAG_SQLSTATE, PG_DIAG_MESSAGE_PRIMARY,
    PG_DIAG_MESSAGE_DETAIL, PG_DIAG_MESSAGE_HINT, PG_DIAG_STATEMENT_POSITION,
    PG_DIAG_INTERNAL_POSITION, PG_DIAG_INTERNAL_QUERY,
    PG_DIAG_CONTEXT, PG_DIAG_SOURCE_FILE,
    PG_DIAG_SOURCE_LINE, PG_DIAG_SOURCE_FUNCTION,
};

#[derive(Debug)]
pub struct Notice {
    pub fields: HashMap<&'static str, String>,
}

type SharedNotices = Arc<Mutex<Vec<Notice>>>;

/// Custom notice receiver function.
extern "C" fn notice_receiver(arg: *mut c_void, result: *const PGresult) {
    if result.is_null() || arg.is_null() {
        return;
    }

    let field_kinds = [
        (PG_DIAG_SEVERITY, "severity"),
        (PG_DIAG_SQLSTATE, "sqlstate"),
        (PG_DIAG_MESSAGE_PRIMARY, "message"),
        (PG_DIAG_MESSAGE_DETAIL, "detail"),
        (PG_DIAG_MESSAGE_HINT, "hint"),
        (PG_DIAG_STATEMENT_POSITION, "statement_position"),
        (PG_DIAG_INTERNAL_POSITION, "internal_position"),
        (PG_DIAG_INTERNAL_QUERY, "internal_query"),
        (PG_DIAG_CONTEXT, "context"),
        (PG_DIAG_SOURCE_FILE, "source_file"),
        (PG_DIAG_SOURCE_LINE, "source_line"),
        (PG_DIAG_SOURCE_FUNCTION, "source_function"),
    ];

    let mut notice = Notice {
        fields: HashMap::new(),
    };

    for (code, label) in field_kinds.iter() {
        let val_ptr = unsafe { PQresultErrorField(result, *code as i32) };
        if !val_ptr.is_null() {
            let val = unsafe { CStr::from_ptr(val_ptr) }.to_string_lossy().into_owned();
            notice.fields.insert(*label, val);
        }
    }

    let shared = unsafe { &*(arg as *const Mutex<Vec<Notice>>) };
    if let Ok(mut vec) = shared.lock() {
        vec.push(notice);
    }
}

fn pq_query(conn: &libpq::Connection, query: &str, notices: SharedNotices) -> Result<(), Box<dyn std::error::Error>> {
    // Capture notice count before the query
    let old_len = {
        let lock = notices.lock().unwrap();
        lock.len()
    };

    // Run the query
    let res = conn.exec(query);
    let res_status = res.status();

    if res_status == libpq::Status::NonFatalError {
        println!("Query notify in effect: {:?}", conn.error_message());
    } else {
        println!("Query did some non-notify thing.");
    }

    // Extract and print only the new notices
    let mut locked = notices.lock().unwrap();
    let new_notices = locked.split_off(old_len);
    if new_notices.is_empty() {
        println!("No notices captured.");
    } else {
        for (i, notice) in new_notices.iter().enumerate() {
            println!("--- Notice {} ---", i + 1);
            for (k, v) in &notice.fields {
                println!("{}: {}", k, v);
            }
        }
    }

    Ok(())
}

fn pq_main() -> Result<(), Box<dyn std::error::Error>> {
    let conninfo = "host=localhost port=5888";
    let query_str = "\
        SELECT repo, count(*) as has_starred \
        FROM github.activity.repo_stargazers \
        WHERE owner = 'stackql' and repo in ('stackql', 'stackql-deploy') \
        and login = 'generalkroll0' \
        GROUP BY repo;\
    ";

    // Create shared notice storage
    let notices: SharedNotices = Arc::new(Mutex::new(Vec::new()));
    let notices_raw_ptr = Arc::into_raw(notices.clone()) as *mut c_void;

    let conn = libpq::Connection::new(&conninfo)?;
    unsafe {
        libpq_sys::PQsetNoticeReceiver((&conn).into(), Some(notice_receiver), notices_raw_ptr);
    }

    // Execute the query
    pq_query(&conn, query_str, notices)?;

    // Manually drop libpq's reference to avoid leak
    unsafe {
        let _ = Arc::from_raw(notices_raw_ptr as *const Mutex<Vec<Notice>>);
    }

    Ok(())
}

fn main() {
    if let Err(e) = pq_main() {
        eprintln!("Error: {}", e);
    }
}
