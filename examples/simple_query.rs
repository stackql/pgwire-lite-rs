// example/simple_query.rs

use colorize::AnsiColor;
use pgwire_lite::{PgwireLite, Value};

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

fn execute_query(conn: &PgwireLite, query: &str) {
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
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Create a connection configuration
    let conn = PgwireLite::new("localhost", 5444, false, "verbose")?;

    println!();
    println!("libpq version: {}", conn.libpq_version());
    println!("Verbosity set to: {}", conn.verbosity());
    println!();

    //
    // registry list example
    //
    print_heading("REGISTRY LIST example");
    execute_query(&conn, "REGISTRY LIST aws");

    //
    // registry pull examples
    //
    print_heading("REGISTRY PULL examples");
    execute_query(&conn, "REGISTRY PULL homebrew");

    //
    // simple select with one row
    //
    print_heading("Literal SELECT example (one row)");
    execute_query(&conn, "SELECT 1 as col_name");

    //
    // simple select with no rows
    //
    print_heading("Literal SELECT example (no rows)");
    execute_query(&conn, "SELECT 1 as col_name WHERE 1=0");

    //
    // failed command - handle expected error
    //
    print_heading("Failed command example");
    execute_query(&conn, "NOTACOMMAND");

    //
    // stackql provider select, multiple rows
    //
    print_heading("StackQL SELECT example (multiple rows)");
    execute_query(
        &conn,
        "SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name IN ('stackql','steampipe')",
    );

    //
    // stackql provider select, provider error, no rows
    //
    print_heading("StackQL SELECT example with provider error and no rows");
    execute_query(&conn, "SELECT id, name, description, stargazers_count FROM github.repos.repos WHERE org = 'nonexistent-org'");

    //
    // another stackql provider select, should succeed
    //
    print_heading("StackQL SELECT example");
    execute_query(
        &conn,
        "SELECT * FROM homebrew.formula.vw_info WHERE formula_name = 'stackql'",
    );

    Ok(())
}
