// use colorize::AnsiColor;
// use pgwire_lite::{PgwireLite, Value, Verbosity};

// fn print_heading(title: &str) {
//     let title_owned = title.to_string(); // Convert &str to String
//     println!("{}", title_owned.blue().bold());
// }

// // Pretty print a row with formatting
// fn print_row(row: &std::collections::HashMap<String, Value>, index: usize) {
//     if index == 0 {
//         // Print header
//         println!("Row {}: {{", index);
//     } else {
//         println!("\nRow {}: {{", index);
//     }

//     for (key, value) in row {
//         println!(
//             "  {}: {}",
//             key.clone().green(),
//             format!("{}", value).yellow()
//         );
//     }
//     println!("}}");
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Create a long-lived connection
//     let mut conn = PgwireLite::new("localhost", 5444, false, Some(Verbosity::Verbose))?;
//     //added mut

//     println!();
//     println!("libpq version: {}", conn.libpq_version());
//     println!("Verbosity set to: {}", conn.verbosity());
//     println!();

//     //
//     // registry list example
//     //
//     print_heading("REGISTRY LIST example");
//     match conn.query("REGISTRY LIST aws") {
//         Ok(result) => {
//             println!(
//                 "Found {} rows with notices: {}",
//                 result.rows.len(),
//                 result.notices.len()
//             );
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//         }
//         Err(e) => eprintln!("Error: {}", e),
//     }
//     println!();

//     //
//     // registry pull example
//     //
//     print_heading("REGISTRY PULL example");
//     match conn.query("REGISTRY PULL homebrew") {
//         Ok(result) => {
//             println!(
//                 "Found {} rows with notices: {}",
//                 result.rows.len(),
//                 result.notices.len()
//             );
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//         }
//         Err(e) => eprintln!("Error: {}", e),
//     }
//     println!();

//     // simple select with one row
//     print_heading("Literal SELECT example (one row)");
//     match conn.query("SELECT 1 as col_name") {
//         Ok(result) => {
//             println!(
//                 "Found {} rows with notices: {}",
//                 result.rows.len(),
//                 result.notices.len()
//             );
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//         }
//         Err(e) => eprintln!("Error: {}", e),
//     }
//     println!();

//     // simple select with no rows
//     print_heading("Literal SELECT example (no rows)");
//     match conn.query("SELECT 1 as col_name WHERE 1=0") {
//         Ok(result) => {
//             println!(
//                 "Found {} rows with notices: {}",
//                 result.rows.len(),
//                 result.notices.len()
//             );
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//             if result.rows.is_empty() {
//                 println!("No rows returned");
//             }
//         }
//         Err(e) => eprintln!("Error: {}", e),
//     }
//     println!();

//     // failed command
//     print_heading("Failed command example");
//     match conn.query("NOTACOMMAND") {
//         Ok(result) => {
//             println!(
//                 "Found {} rows with notices: {}",
//                 result.rows.len(),
//                 result.notices.len()
//             );
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//         }
//         Err(e) => eprintln!("{}", e),
//     }
//     println!();

//     // stackql provider select, multiple rows
//     print_heading("StackQL SELECT example (multiple rows)");
//     match conn
//         .query("SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name = 'stackql'")
//     {
//         Ok(result) => {
//             println!(
//                 "Found {} rows with notices: {}",
//                 result.rows.len(),
//                 result.notices.len()
//             );
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//         }
//         Err(e) => eprintln!("Error: {}", e),
//     }
//     println!();

//     // stackql provider select, provider error, no rows
//     print_heading("StackQL SELECT example with provider error and no rows");
//     match conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'us-east-1' AND data__Identifier = 'fred'") {
//         Ok(result) => {
//             println!("Found {} rows with notices: {}", result.rows.len(), result.notices.len());
//             for (i, row) in result.rows.iter().enumerate() {
//                 print_row(row, i);
//             }
//             // Print any notices
//             if !result.notices.is_empty() {
//                 println!("\nNotices:");
//                 for (i, notice) in result.notices.iter().enumerate() {
//                     println!("Notice {}: {:?}", i, notice);
//                 }
//             }
//         },
//         Err(e) => eprintln!("Error: {}", e),
//     }

//     Ok(())
// }

use colorize::AnsiColor;
use pgwire_lite::{PgwireLite, Value, Verbosity};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a connection configuration
    let conn = PgwireLite::new("localhost", 5444, false, Some(Verbosity::Verbose))?;

    println!();
    println!("libpq version: {}", conn.libpq_version());
    println!("Verbosity set to: {}", conn.verbosity());
    println!();

    //
    // registry list example
    //
    print_heading("REGISTRY LIST example");
    match conn.query("REGISTRY LIST aws") {
        Ok(result) => {
            println!(
                "Found {} rows with notices: {}",
                result.rows.len(),
                result.notices.len()
            );
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    println!();

    //
    // registry pull example
    //
    print_heading("REGISTRY PULL example");
    match conn.query("REGISTRY PULL homebrew") {
        Ok(result) => {
            println!(
                "Found {} rows with notices: {}",
                result.rows.len(),
                result.notices.len()
            );
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    println!();

    // simple select with one row
    print_heading("Literal SELECT example (one row)");
    match conn.query("SELECT 1 as col_name") {
        Ok(result) => {
            println!(
                "Found {} rows with notices: {}",
                result.rows.len(),
                result.notices.len()
            );
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    println!();

    // simple select with no rows
    print_heading("Literal SELECT example (no rows)");
    match conn.query("SELECT 1 as col_name WHERE 1=0") {
        Ok(result) => {
            println!(
                "Found {} rows with notices: {}",
                result.rows.len(),
                result.notices.len()
            );
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
            if result.rows.is_empty() {
                println!("No rows returned");
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    println!();

    // failed command - handle expected error
    print_heading("Failed command example");
    match conn.query("NOTACOMMAND") {
        Ok(result) => {
            println!(
                "Found {} rows with notices: {}",
                result.rows.len(),
                result.notices.len()
            );
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            println!("Error handled as expected, continuing...");
        }
    }
    println!();

    // Use the same connection object for the remaining queries
    // This is the critical test - can we continue using the same connection
    // after a syntax error, just like psql does?

    // stackql provider select, multiple rows
    print_heading("StackQL SELECT example (multiple rows)");
    match conn.query("SELECT * FROM homebrew.formula.vw_usage_metrics WHERE formula_name = 'stackql'") {
        Ok(result) => {
            println!(
                "Found {} rows with notices: {}",
                result.rows.len(),
                result.notices.len()
            );
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    println!();

    // Still using the same connection
    // stackql provider select, provider error, no rows
    print_heading("StackQL SELECT example with provider error and no rows");
    match conn.query("SELECT region, function_name FROM aws.lambda.functions WHERE region = 'us-east-1' AND data__Identifier = 'fred'") {
        Ok(result) => {
            println!("Found {} rows with notices: {}", result.rows.len(), result.notices.len());
            for (i, row) in result.rows.iter().enumerate() {
                print_row(row, i);
            }
            // Print any notices
            if !result.notices.is_empty() {
                println!("\nNotices:");
                for (i, notice) in result.notices.iter().enumerate() {
                    println!("Notice {}: {:?}", i, notice);
                }
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}