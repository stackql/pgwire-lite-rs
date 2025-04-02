// use tokio_postgres::{NoTls, Error, AsyncMessage};
// use tokio::time::{sleep, Duration};
// use futures::future::poll_fn;
// use std::env;

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     // Initialize logger for debugging
//     env::set_var("RUST_LOG", "tokio_postgres=debug");
//     env_logger::init();

//     // 1. Connect to the PostgreSQL-compatible server
//     let (client, mut connection) = tokio_postgres::connect(
//         "host=localhost port=5444 user=stackql dbname=stackql",
//         NoTls,
//     ).await?;
//     println!("✅ Connected to the server");

//     // 2. Spawn a task to continuously poll the connection for async messages
//     tokio::spawn(async move {
//         loop {
//             match poll_fn(|cx| connection.poll_message(cx)).await {
//                 Some(Ok(message)) => {
//                     match message {
//                         AsyncMessage::Notice(notice) => {
//                             println!("⚠️ NOTICE from server: {}", notice.message());
//                             if let Some(detail) = notice.detail() {
//                                 println!("📝 Detail: {}", detail);
//                             }
//                             if let Some(hint) = notice.hint() {
//                                 println!("💡 Hint: {}", hint);
//                             }
//                         }
//                         AsyncMessage::Notification(notification) => {
//                             println!(
//                                 "🔔 Notification received - Channel: {}, Payload: {}",
//                                 notification.channel(),
//                                 notification.payload()
//                             );
//                         }
//                         _ => {
//                             println!("📥 Received unhandled message: {:?}", message);
//                         }
//                     }
//                 }
//                 Some(Err(e)) => {
//                     eprintln!("❌ Connection error: {}", e);
//                     break;
//                 }
//                 _none => break,
//             }
//         }
//     });

//     // 3. Run your query and capture all possible messages
//     let query = "
//         SELECT repo, count(*) as has_starred
//         FROM github.activity.repo_stargazers
//         WHERE owner = 'fred'
//           AND repo in ('stackql', 'stackql-deploy')
//           AND login = 'generalkroll0'
//         GROUP BY repo;
//     ";

//     println!("📥 Executing query...");
//     match client.query(query, &[]).await {
//         Ok(rows) => {
//             if rows.is_empty() {
//                 println!("Query returned no data");
//             } else {
//                 println!("🎉 Query returned {} rows", rows.len());

//                 // Print column names
//                 let columns: Vec<String> = rows[0]
//                     .columns()
//                     .iter()
//                     .map(|col| col.name().to_string())
//                     .collect();
//                 println!("Columns: {:?}", columns);

//                 // Print row data
//                 for row in rows {
//                     let mut row_data = Vec::new();
//                     for (i, col) in row.columns().iter().enumerate() {
//                         let value: Result<String, _> = row.try_get(i);
//                         row_data.push(format!("{}: {:?}", col.name(), value));
//                     }
//                     println!("Row: {:?}", row_data);
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("❌ Query failed: {}", e);
//         }
//     }

//     // Give time for asynchronous messages to be processed (e.g., NOTICES)
//     println!("⌛ Waiting for possible NOTICES from server...");
//     sleep(Duration::from_secs(5)).await;

//     println!("✅ Test harness completed successfully");
//     Ok(())
// }

// use tokio_postgres::{NoTls, Error, AsyncMessage};
// use tokio::time::{sleep, Duration};
// use futures::future::poll_fn;
// use tokio::spawn;
// use std::env;

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     // Set up logging for debugging purposes
//     env::set_var("RUST_LOG", "tokio_postgres=debug");
//     env_logger::init();

//     // 1. Connect to the PostgreSQL server
//     let (client, mut connection) = tokio_postgres::connect(
//         "host=localhost port=5444 user=stackql dbname=stackql",
//         NoTls,
//     ).await?;
//     println!("✅ Connected to the server");

//     // 2. Spawn a task to capture asynchronous messages like `NOTICE`
//     let notice_task = spawn(async move {
//         loop {
//             match poll_fn(|cx| connection.poll_message(cx)).await {
//                 Some(Ok(message)) => match message {
//                     AsyncMessage::Notice(notice) => {
//                         println!("⚠️ NOTICE: {}", notice.message());
//                         if let Some(detail) = notice.detail() {
//                             println!("📝 Detail: {}", detail);
//                         }
//                         if let Some(hint) = notice.hint() {
//                             println!("💡 Hint: {}", hint);
//                         }
//                     }
//                     AsyncMessage::Notification(notification) => {
//                         println!("🔔 Notification: Channel: {}, Payload: {}", notification.channel(), notification.payload());
//                     }
//                     _ => {} // Ignore other message types for now
//                 },
//                 Some(Err(e)) => {
//                     eprintln!("❌ Connection error while capturing notices: {}", e);
//                     break;
//                 }
//                 _none => break, // Connection closed
//             }
//         }
//     });

//     // 3. Execute your query
//     let query = "
//         SELECT repo, count(*) as has_starred
//         FROM github.activity.repo_stargazers
//         WHERE owner = 'fred'
//           AND repo in ('stackql', 'stackql-deploy')
//           AND login = 'generalkroll0'
//         GROUP BY repo;
//     ";

//     println!("📥 Executing query...");
//     match client.query(query, &[]).await {
//         Ok(rows) => {
//             if rows.is_empty() {
//                 println!("Query returned no data");
//             } else {
//                 println!("🎉 Query returned {} rows", rows.len());

//                 for row in rows {
//                     let repo: &str = row.get("repo");
//                     let has_starred: i64 = row.get("has_starred");
//                     println!("repo: {}, has_starred: {}", repo, has_starred);
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("❌ Query failed: {}", e);
//         }
//     }

//     // 4. Give some time for asynchronous messages to be processed
//     println!("⌛ Waiting for possible NOTICES from server...");
//     sleep(Duration::from_secs(5)).await;

//     // 5. Ensure notice task completes
//     if let Err(e) = notice_task.await {
//         eprintln!("❌ Notice task failed: {:?}", e);
//     }

//     println!("✅ Test harness completed successfully");
//     Ok(())
// }

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::debug;
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Connect to the server (no SSL, no authentication)
    let mut stream = TcpStream::connect("127.0.0.1:5444").await.unwrap();
    println!("✅ Connected to server");

    // Send StartupMessage
    let startup_message = create_startup_message("stackql", "stackql");
    stream.write_all(&startup_message).await.unwrap();
    println!("📤 Sent StartupMessage");

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    debug!("📥 Received response: {:?}", &buffer[..n]);

    // Send query
    let query = "SELECT repo, count(*) as has_starred FROM github.activity.repo_stargazers WHERE owner = 'fred' AND repo in ('stackql', 'stackql-deploy') AND login = 'generalkroll0' GROUP BY repo;";
    let query_message = create_query_message(query);
    stream.write_all(&query_message).await.unwrap();
    println!("📤 Sent QueryMessage");

    loop {
        let n = stream.read(&mut buffer).await.unwrap();
        if n == 0 {
            break;
        }

        process_message(&buffer[..n]);
    }

    println!("✅ Done");
}

fn create_startup_message(user: &str, database: &str) -> Vec<u8> {
    let user_string = format!("user\0{}\0", user);
    let database_string = format!("database\0{}\0", database);

    let protocol_version = [0x00, 0x03, 0x00, 0x00]; // Protocol version 3.0
    let payload = [protocol_version.as_ref(), user_string.as_bytes(), database_string.as_bytes(), &[0]].concat();

    let mut message = Vec::new();
    message.extend(&(payload.len() as u32 + 4).to_be_bytes());
    message.extend(&payload);

    message
}

fn create_query_message(query: &str) -> Vec<u8> {
    let mut message = Vec::new();
    message.push(b'Q');
    message.extend(&(query.len() as u32 + 5).to_be_bytes());
    message.extend(query.as_bytes());
    message.push(0); // Null terminator

    message
}

fn process_message(data: &[u8]) {
    let mut offset = 0;

    while offset < data.len() {
        let message_type = data[offset] as char;
        offset += 1;

        let message_length = u32::from_be_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
        offset += 4;

        let content = &data[offset..offset + message_length - 4];
        offset += message_length - 4;

        match message_type {
            'R' => handle_authentication(content),
            'S' => handle_parameter_status(content),
            'T' => handle_row_description(content),
            'D' => handle_data_row(content),
            'C' => println!("✅ Command Complete: {}", String::from_utf8_lossy(content)),
            'E' => println!("❌ Error: {}", String::from_utf8_lossy(content)),
            'N' => println!("⚠️ NOTICE: {}", String::from_utf8_lossy(content)),
            'Z' => println!("🟢 Ready for Query"),
            _ => debug!("📥 Unhandled message type: {} Content: {:?}", message_type, content),
        }
    }
}

fn handle_authentication(content: &[u8]) {
    let auth_type = u32::from_be_bytes(content[0..4].try_into().unwrap());
    if auth_type == 0 {
        println!("✅ Authentication successful");
    } else {
        println!("❌ Authentication required, unsupported by this client");
    }
}

fn handle_parameter_status(content: &[u8]) {
    if let Ok(text) = std::str::from_utf8(content) {
        let parts: Vec<&str> = text.split('\0').collect();
        if parts.len() >= 2 {
            println!("🔧 Parameter Status: {} = {}", parts[0], parts[1]);
        }
    }
}

fn handle_row_description(content: &[u8]) {
    let column_count = u16::from_be_bytes(content[0..2].try_into().unwrap());
    println!("📥 Row Description: {} columns", column_count);
}

fn handle_data_row(content: &[u8]) {
    let column_count = u16::from_be_bytes(content[0..2].try_into().unwrap());
    println!("📥 Data Row: {} columns", column_count);
}
