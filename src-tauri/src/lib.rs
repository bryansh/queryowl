use tauri::{Manager, Emitter, menu::*, PhysicalPosition, PhysicalSize};
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

#[derive(Debug, Serialize)]
struct SchemaTable {
    table_name: String,
    table_type: String,
    column_count: i64,
}

#[derive(Debug, Serialize)]
struct SchemaColumn {
    column_name: String,
    data_type: String,
    is_nullable: String,
    column_default: Option<String>,
    is_primary_key: bool,
}

#[derive(Debug, Serialize)]
struct SchemaIndex {
    index_name: String,
    table_name: String,
    column_names: Vec<String>,
    is_unique: bool,
    is_primary: bool,
    index_type: String,
}

#[derive(Debug, Serialize)]
struct SchemaFunction {
    function_name: String,
    schema_name: String,
    return_type: String,
    parameters: Vec<String>,
    function_type: String, // FUNCTION or PROCEDURE
}

#[derive(Debug, Serialize)]
struct SchemaTrigger {
    trigger_name: String,
    table_name: String,
    event_manipulation: String, // INSERT, UPDATE, DELETE
    action_timing: String, // BEFORE, AFTER
    action_statement: String,
}

#[derive(Debug, Serialize)]
struct SchemaSequence {
    sequence_name: String,
    data_type: String,
    start_value: String,
    increment: String,
    max_value: String,
    min_value: String,
}

#[derive(Debug, Serialize)]
struct SchemaForeignKey {
    constraint_name: String,
    table_name: String,
    column_name: String,
    foreign_table_name: String,
    foreign_column_name: String,
    update_rule: String,
    delete_rule: String,
}

#[derive(Debug, Serialize)]
struct SchemaConstraint {
    constraint_name: String,
    table_name: String,
    constraint_type: String, // CHECK, UNIQUE, PRIMARY KEY, FOREIGN KEY
    column_names: Vec<String>,
    check_clause: Option<String>,
}

#[derive(Debug, Serialize)]
struct SchemaEnum {
    type_name: String,
    enum_values: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SchemaSchema {
    schema_name: String,
    owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

#[derive(Debug, Serialize)]
struct DatabaseSchema {
    tables: Vec<SchemaTable>,
    views: Vec<SchemaTable>,
    materialized_views: Vec<SchemaTable>,
    indexes: Vec<SchemaIndex>,
    functions: Vec<SchemaFunction>,
    triggers: Vec<SchemaTrigger>,
    sequences: Vec<SchemaSequence>,
    foreign_keys: Vec<SchemaForeignKey>,
    constraints: Vec<SchemaConstraint>,
    enums: Vec<SchemaEnum>,
    schemas: Vec<SchemaSchema>,
}

#[tauri::command]
async fn get_database_schema(app: tauri::AppHandle, connection_id: String) -> Result<DatabaseSchema, String> {
    println!("Fetching schema for connection: {}", connection_id);
    
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
    
    // Connect to database
    let (client, conn) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Initialize collections for all entity types
    let mut tables = Vec::new();
    let mut views = Vec::new();
    let mut materialized_views = Vec::new();
    let mut indexes = Vec::new();
    let mut functions = Vec::new();
    let mut triggers = Vec::new();
    let mut sequences = Vec::new();
    let mut foreign_keys = Vec::new();
    let mut constraints = Vec::new();
    let mut enums = Vec::new();
    let mut schemas = Vec::new();

    // 1. Query for tables, views, and materialized views
    let table_query = "
        SELECT 
            t.table_name,
            t.table_type,
            COUNT(c.column_name) as column_count
        FROM information_schema.tables t
        LEFT JOIN information_schema.columns c ON t.table_name = c.table_name 
            AND t.table_schema = c.table_schema
        WHERE t.table_schema NOT IN ('information_schema', 'pg_catalog')
        GROUP BY t.table_name, t.table_type
        ORDER BY t.table_type, t.table_name
    ";
    
    let rows = client.query(table_query, &[]).await
        .map_err(|e| format!("Schema query failed: {}", e))?;
    
    for row in rows {
        let table_name: String = row.get(0);
        let table_type: String = row.get(1);
        let column_count: i64 = row.get(2);
        
        let schema_table = SchemaTable {
            table_name,
            table_type: table_type.clone(),
            column_count,
        };
        
        match table_type.as_str() {
            "BASE TABLE" => tables.push(schema_table),
            "VIEW" => views.push(schema_table),
            "MATERIALIZED VIEW" => materialized_views.push(schema_table),
            _ => {}
        }
    }

    // 2. Query for indexes (simplified)
    let index_query = "
        SELECT 
            indexname as index_name,
            tablename as table_name,
            indexdef
        FROM pg_indexes 
        WHERE schemaname = 'public'
        ORDER BY tablename, indexname
    ";
    
    let rows = client.query(index_query, &[]).await
        .map_err(|e| format!("Index query failed: {}", e))?;
    
    for row in rows {
        let index_name: String = row.get(0);
        let table_name: String = row.get(1);
        let _index_def: String = row.get(2);
        
        // Simple heuristics from index definition
        let is_unique = _index_def.contains("UNIQUE");
        let is_primary = _index_def.contains("PRIMARY KEY") || index_name.ends_with("_pkey");
        let index_type = if is_primary { "btree".to_string() } else { "btree".to_string() };
        
        indexes.push(SchemaIndex {
            index_name,
            table_name,
            column_names: vec![], // Simplified for now
            is_unique,
            is_primary,
            index_type,
        });
    }

    // 3. Query for functions and procedures (simplified)
    let function_query = "
        SELECT 
            routine_name as function_name,
            routine_schema as schema_name,
            COALESCE(data_type, 'void') as return_type,
            routine_type as function_type
        FROM information_schema.routines 
        WHERE routine_schema = 'public'
        ORDER BY routine_name
    ";
    
    let rows = client.query(function_query, &[]).await
        .map_err(|e| format!("Function query failed: {}", e))?;
    
    for row in rows {
        let function_name: String = row.get(0);
        let schema_name: String = row.get(1);
        let return_type: String = row.get(2);
        let function_type: String = row.get(3);
        
        functions.push(SchemaFunction {
            function_name,
            schema_name,
            return_type,
            parameters: vec![], // Simplified for now
            function_type,
        });
    }

    // For now, let's comment out the complex queries to isolate the issue
    // We'll just return tables and views first to see if those work
    
    // TODO: Add back other entity types once basic loading works

    Ok(DatabaseSchema {
        tables,
        views,
        materialized_views,
        indexes,
        functions,
        triggers,
        sequences,
        foreign_keys,
        constraints,
        enums,
        schemas,
    })
}

#[tauri::command]
async fn get_table_columns(app: tauri::AppHandle, connection_id: String, table_name: String) -> Result<Vec<SchemaColumn>, String> {
    println!("Fetching columns for table: {} on connection: {}", table_name, connection_id);
    
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
    
    // Connect to database
    let (client, conn) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Query for table columns with primary key information
    let column_query = "
        SELECT 
            c.column_name,
            c.data_type,
            c.is_nullable,
            c.column_default,
            CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key
        FROM information_schema.columns c
        LEFT JOIN (
            SELECT kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu 
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY'
                AND tc.table_schema NOT IN ('information_schema', 'pg_catalog')
                AND tc.table_name = $1
        ) pk ON c.column_name = pk.column_name
        WHERE c.table_schema NOT IN ('information_schema', 'pg_catalog') 
            AND c.table_name = $1
        ORDER BY c.ordinal_position
    ";
    
    let rows = client.query(column_query, &[&table_name]).await
        .map_err(|e| format!("Column query failed: {}", e))?;
    
    let mut columns = Vec::new();
    
    for row in rows {
        let column_name: String = row.get(0);
        let data_type: String = row.get(1);
        let is_nullable: String = row.get(2);
        let column_default: Option<String> = row.get(3);
        let is_primary_key: bool = row.get(4);
        
        columns.push(SchemaColumn {
            column_name,
            data_type,
            is_nullable,
            column_default,
            is_primary_key,
        });
    }
    
    Ok(columns)
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

#[tauri::command]
async fn save_window_state(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("app_state.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    if let Some(window) = app.get_webview_window("main") {
        let position = window.outer_position()
            .map_err(|e| format!("Failed to get window position: {}", e))?;
        let size = window.outer_size()
            .map_err(|e| format!("Failed to get window size: {}", e))?;
        let maximized = window.is_maximized()
            .map_err(|e| format!("Failed to check if maximized: {}", e))?;
        
        let window_state = WindowState {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            maximized,
        };
        
        let value = serde_json::to_value(&window_state)
            .map_err(|e| format!("Failed to serialize window state: {}", e))?;
        store.set("window_state", value);
        store.save()
            .map_err(|e| format!("Failed to persist window state: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn restore_window_state(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store_builder("app_state.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    if let Some(window_state_value) = store.get("window_state") {
        if let Ok(window_state) = serde_json::from_value::<WindowState>(window_state_value) {
            if let Some(window) = app.get_webview_window("main") {
                // Restore position and size
                let position = PhysicalPosition::new(window_state.x, window_state.y);
                let size = PhysicalSize::new(window_state.width, window_state.height);
                
                let _ = window.set_position(position);
                let _ = window.set_size(size);
                
                // Restore maximized state
                if window_state.maximized {
                    let _ = window.maximize();
                }
            }
        }
    }
    
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
            update_last_connected,
            get_database_schema,
            get_table_columns,
            save_window_state,
            restore_window_state
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
