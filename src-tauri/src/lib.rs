use tauri::{Manager, Emitter, menu::*};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

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
    
    let new_connection = DatabaseConnection {
        id: Uuid::new_v4().to_string(),
        name: connection.name,
        host: connection.host,
        port: connection.port,
        database: connection.database,
        username: connection.username,
        password: Some(connection.password),
        ssl: connection.ssl,
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
            conn.name = connection.name;
            conn.host = connection.host;
            conn.port = connection.port;
            conn.database = connection.database;
            conn.username = connection.username;
            conn.password = Some(connection.password);
            conn.ssl = connection.ssl;
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
async fn connect_to_database(connection: DatabaseConnection) -> Result<(), String> {
    let password = connection.password.as_deref().unwrap_or("");
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
        .setup(|app| {
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
            
            // Create submenus
            let debug_submenu = SubmenuBuilder::new(app, "Debug")
                .items(&[&open_logs, &show_path])
                .build()?;
                
            let app_submenu = SubmenuBuilder::new(app, "QueryOwl")
                .items(&[&about, &quit])
                .build()?;
            
            // Create main menu
            let menu = MenuBuilder::new(app)
                .items(&[&app_submenu, &debug_submenu])
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
