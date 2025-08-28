use tauri::{Manager, Emitter, menu::*};
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

mod encryption;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseConnection {
    id: String,
    name: String,
    host: String,
    port: u16,
    database: String,
    username: String,
    password: Option<String>,
    ssl: Option<bool>,
    color: Option<String>,
    created_at: String,
    last_connected: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateConnectionRequest {
    name: String,
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl: Option<bool>,
    color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateConnectionRequest {
    id: String,
    name: String,
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl: Option<bool>,
    color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TestConnectionRequest {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl: Option<bool>,
}

#[derive(Debug, Serialize)]
struct TestConnectionResponse {
    success: bool,
    error: Option<String>,
}

// Global state for connections
static CONNECTIONS: Mutex<Option<HashMap<String, DatabaseConnection>>> = Mutex::new(None);
static ACTIVE_CONNECTION: Mutex<Option<String>> = Mutex::new(None);


// Database command functions
#[tauri::command]
async fn get_stored_connections(app: tauri::AppHandle) -> Result<Vec<DatabaseConnection>, String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    match store.get("connections") {
        Some(value) => {
            serde_json::from_value(value.clone())
                .map_err(|e| format!("Failed to deserialize connections: {}", e))
        },
        None => Ok(vec![])
    }
}

#[tauri::command]
async fn save_connection(app: tauri::AppHandle, connection: CreateConnectionRequest) -> Result<DatabaseConnection, String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let encrypted_password = encryption::encrypt_password(&connection.password)
        .map_err(|e| format!("Failed to encrypt password: {}", e))?;
    
    let new_connection = DatabaseConnection {
        id: Uuid::new_v4().to_string(),
        name: connection.name,
        host: connection.host,
        port: connection.port,
        database: connection.database,
        username: connection.username,
        password: Some(encrypted_password),
        ssl: connection.ssl,
        color: connection.color,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_connected: None,
    };
    
    let mut connections: Vec<DatabaseConnection> = match store.get("connections") {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_default(),
        None => vec![]
    };
    
    connections.push(new_connection.clone());
    
    let value = serde_json::to_value(&connections)
        .map_err(|e| format!("Failed to serialize connections: {}", e))?;
    
    store.set("connections", value);
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(new_connection)
}

#[tauri::command]
async fn delete_connection(app: tauri::AppHandle, id: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let mut connections: Vec<DatabaseConnection> = match store.get("connections") {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_default(),
        None => return Ok(())
    };
    
    connections.retain(|conn| conn.id != id);
    
    let value = serde_json::to_value(&connections)
        .map_err(|e| format!("Failed to serialize connections: {}", e))?;
    
    store.set("connections", value);
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn update_connection(app: tauri::AppHandle, connection: UpdateConnectionRequest) -> Result<DatabaseConnection, String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let mut connections: Vec<DatabaseConnection> = match store.get("connections") {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_default(),
        None => return Err("No connections found".to_string())
    };
    
    let mut updated_connection = None;
    for conn in &mut connections {
        if conn.id == connection.id {
            let encrypted_password = encryption::encrypt_password(&connection.password)
                .map_err(|e| format!("Failed to encrypt password: {}", e))?;
            
            conn.name = connection.name;
            conn.host = connection.host;
            conn.port = connection.port;
            conn.database = connection.database;
            conn.username = connection.username;
            conn.password = Some(encrypted_password);
            conn.ssl = connection.ssl;
            conn.color = connection.color;
            updated_connection = Some(conn.clone());
            break;
        }
    }
    
    match updated_connection {
        Some(conn) => {
            let value = serde_json::to_value(&connections)
                .map_err(|e| format!("Failed to serialize connections: {}", e))?;
            
            store.set("connections", value);
            store.save().map_err(|e| format!("Failed to save store: {}", e))?;
            
            Ok(conn)
        },
        None => Err("Connection not found".to_string())
    }
}

#[tauri::command]
async fn test_database_connection(connection: TestConnectionRequest) -> Result<TestConnectionResponse, String> {
    let ssl_mode = if connection.ssl.unwrap_or(false) { "require" } else { "disable" };
    
    let config = format!(
        "host={} port={} dbname={} user={} password={} sslmode={}",
        connection.host,
        connection.port,
        connection.database,
        connection.username,
        connection.password,
        ssl_mode
    );
    
    match tokio_postgres::connect(&config, tokio_postgres::NoTls).await {
        Ok(_) => Ok(TestConnectionResponse { success: true, error: None }),
        Err(e) => Ok(TestConnectionResponse { 
            success: false, 
            error: Some(e.to_string()) 
        })
    }
}

#[tauri::command]
async fn test_stored_connection(connection: DatabaseConnection) -> Result<TestConnectionResponse, String> {
    println!("Testing stored connection: {}", connection.name);
    
    // Decrypt password if it's encrypted
    let password = match &connection.password {
        Some(pwd) => {
            println!("Password present, checking if encrypted...");
            if encryption::is_encrypted(pwd) {
                println!("Password is encrypted, decrypting...");
                match encryption::decrypt_password(pwd) {
                    Ok(decrypted) => {
                        println!("Password decrypted successfully");
                        decrypted
                    },
                    Err(e) => {
                        println!("Failed to decrypt password: {}", e);
                        return Err(format!("Failed to decrypt password: {}", e));
                    }
                }
            } else {
                println!("Password is not encrypted, using as-is");
                pwd.clone()
            }
        },
        None => String::new(),
    };
    
    let ssl_mode = if connection.ssl.unwrap_or(false) { "require" } else { "disable" };
    
    let config = format!(
        "host={} port={} dbname={} user={} password={} sslmode={}",
        connection.host,
        connection.port,
        connection.database,
        connection.username,
        password,
        ssl_mode
    );
    
    match tokio_postgres::connect(&config, tokio_postgres::NoTls).await {
        Ok(_) => Ok(TestConnectionResponse { success: true, error: None }),
        Err(e) => Ok(TestConnectionResponse { 
            success: false, 
            error: Some(e.to_string()) 
        })
    }
}

#[tauri::command]
async fn execute_query(app: tauri::AppHandle, connection_id: String, sql: String) -> Result<Vec<serde_json::Value>, String> {
    println!("Executing query for connection: {}", connection_id);
    println!("SQL: {}", sql);
    
    // Load connections from store to get the connection details
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let connections: Vec<DatabaseConnection> = store.get("connections")
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    
    let connection = connections.iter()
        .find(|c| c.id == connection_id)
        .ok_or("Connection not found")?;
    
    // Decrypt password if it's encrypted
    let password = match &connection.password {
        Some(encrypted) if encryption::is_encrypted(encrypted) => {
            encryption::decrypt_password(encrypted)?
        },
        Some(plain) => plain.clone(),
        None => String::new(),
    };
    
    let ssl_mode = if connection.ssl.unwrap_or(false) { "require" } else { "disable" };
    
    let config = format!(
        "host={} port={} dbname={} user={} password={} sslmode={}",
        connection.host,
        connection.port,
        connection.database,
        connection.username,
        password,
        ssl_mode
    );
    
    // Connect and execute query
    let (client, conn) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Execute the query
    let rows = client.query(&sql, &[]).await
        .map_err(|e| format!("Query execution failed: {}", e))?;
    
    // Convert rows to JSON - simplified approach
    let mut results = Vec::new();
    for row in rows {
        let mut row_map = serde_json::Map::new();
        
        for (i, column) in row.columns().iter().enumerate() {
            let column_name = column.name();
            
            // Try to get the value as different types, falling back to string
            let value = if let Ok(v) = row.try_get::<_, Option<bool>>(i) {
                v.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.try_get::<_, Option<i32>>(i) {
                v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.try_get::<_, Option<i64>>(i) {
                v.map(|n| serde_json::Value::Number(n.into())).unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.try_get::<_, Option<f64>>(i) {
                v.and_then(|n| serde_json::Number::from_f64(n))
                 .map(serde_json::Value::Number)
                 .unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                v.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
            } else {
                // For any other type, fallback to null (we can enhance this later)
                serde_json::Value::Null
            };
            
            row_map.insert(column_name.to_string(), value);
        }
        
        results.push(serde_json::Value::Object(row_map));
    }
    
    Ok(results)
}

#[tauri::command]
async fn connect_to_database(connection: DatabaseConnection) -> Result<(), String> {
    let password = match &connection.password {
        Some(encrypted) if encryption::is_encrypted(encrypted) => {
            encryption::decrypt_password(encrypted)?
        },
        Some(plain) => plain.clone(),
        None => String::new(),
    };
    let ssl_mode = if connection.ssl.unwrap_or(false) { "require" } else { "disable" };
    
    let config = format!(
        "host={} port={} dbname={} user={} password={} sslmode={}",
        connection.host,
        connection.port,
        connection.database,
        connection.username,
        password,
        ssl_mode
    );
    
    match tokio_postgres::connect(&config, tokio_postgres::NoTls).await {
        Ok(_) => {
            let mut active = ACTIVE_CONNECTION.lock().unwrap();
            *active = Some(connection.id);
            Ok(())
        },
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn disconnect_from_database() -> Result<(), String> {
    let mut active = ACTIVE_CONNECTION.lock().unwrap();
    *active = None;
    Ok(())
}

#[tauri::command]
async fn update_last_connected(app: tauri::AppHandle, id: String) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let mut connections: Vec<DatabaseConnection> = match store.get("connections") {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_default(),
        None => return Ok(())
    };
    
    for conn in &mut connections {
        if conn.id == id {
            conn.last_connected = Some(chrono::Utc::now().to_rfc3339());
            break;
        }
    }
    
    let value = serde_json::to_value(&connections)
        .map_err(|e| format!("Failed to serialize connections: {}", e))?;
    
    store.set("connections", value);
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn open_log_folder(app: tauri::AppHandle) -> Result<(), String> {
    let log_dir = app.path().app_log_dir()
        .map_err(|e| format!("Failed to get log directory: {}", e))?;
    
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&log_dir)
        .spawn()
        .map_err(|e| format!("Failed to open log folder: {}", e))?;
    
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&log_dir)
        .spawn()
        .map_err(|e| format!("Failed to open log folder: {}", e))?;
    
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&log_dir)
        .spawn()
        .map_err(|e| format!("Failed to open log folder: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_log_path(app: tauri::AppHandle) -> Result<String, String> {
    let log_dir = app.path().app_log_dir()
        .map_err(|e| format!("Failed to get log directory: {}", e))?;
    let log_file = log_dir.join("queryowl.log");
    Ok(log_file.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize encryption
            encryption::initialize_encryption(&app.handle())?;
            // Migrate existing unencrypted passwords
            encryption::migrate_existing_connections(&app.handle())?;
            
            // Create menu items
            let about = MenuItemBuilder::new("About QueryOwl").id("about").build(app)?;
            let quit = MenuItemBuilder::new("Quit QueryOwl")
                .id("quit")
                .accelerator("CmdOrCtrl+Q")
                .build(app)?;
            let open_logs = MenuItemBuilder::new("Open Log Folder")
                .id("open_log_folder")
                .accelerator("CmdOrCtrl+Shift+L")
                .build(app)?;
            let show_path = MenuItemBuilder::new("Show Log Path")
                .id("show_log_path")
                .accelerator("CmdOrCtrl+Shift+P")
                .build(app)?;
            
            // Create Edit menu using Tauri's predefined items
            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;
                
            let debug_submenu = SubmenuBuilder::new(app, "Debug")
                .items(&[&open_logs, &show_path])
                .build()?;
                
            let app_submenu = SubmenuBuilder::new(app, "QueryOwl")
                .items(&[&about, &quit])
                .build()?;
            
            // Create main menu
            let menu = MenuBuilder::new(app)
                .items(&[&app_submenu, &edit_submenu, &debug_submenu])
                .build()?;

            app.set_menu(menu)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            open_log_folder, 
            get_log_path,
            get_stored_connections,
            save_connection,
            update_connection,
            delete_connection,
            test_database_connection,
            test_stored_connection,
            execute_query,
            connect_to_database,
            disconnect_from_database,
            update_last_connected
        ])
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "open_log_folder" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = open_log_folder(app_handle).await;
                    });
                },
                "show_log_path" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        match get_log_path(app_handle.clone()).await {
                            Ok(path) => {
                                println!("Emitting log path: {}", path);
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if let Err(e) = window.emit("show_log_path", &path) {
                                        eprintln!("Failed to emit show_log_path: {}", e);
                                    }
                                } else {
                                    eprintln!("No main window found");
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to get log path: {}", e);
                            }
                        }
                    });
                },
                "about" => {
                    #[cfg(target_os = "macos")]
                    {
                        use std::process::Command;
                        let _ = Command::new("osascript")
                            .arg("-e")
                            .arg(r#"tell application "System Events"
                                display dialog "QueryOwl
Version 0.1.9

A powerful PostgreSQL database query tool built with Tauri and Svelte 5

© 2025 Bryan Short" with title "About QueryOwl" buttons {"OK"} default button 1 with icon 1
                            end tell"#)
                            .spawn();
                    }
                },
                "quit" => {
                    app.exit(0);
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
