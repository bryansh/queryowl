use tauri::{Manager, Emitter, menu::*, PhysicalPosition, PhysicalSize};
use tauri_plugin_store::StoreExt;
use std::fs::File;
use std::io::{Write, BufWriter};
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

#[derive(Debug, Deserialize)]
struct CreateDatabaseRequest {
    // Connection details for the PostgreSQL server
    host: String,
    port: u16,
    username: String,
    password: String,
    ssl: Option<bool>,
    // New database details
    new_database_name: String,
    encoding: Option<String>,
    template: Option<String>,
    owner: Option<String>,
}

#[tauri::command]
async fn create_database(request: CreateDatabaseRequest) -> Result<String, String> {
    println!("Creating database: {}", request.new_database_name);

    // Validate database name - basic SQL injection prevention
    if !request.new_database_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Database name can only contain alphanumeric characters and underscores".to_string());
    }

    let ssl_mode = if request.ssl.unwrap_or(false) { "require" } else { "disable" };

    // Connect to postgres database to create the new database
    let config = format!(
        "host={} port={} dbname=postgres user={} password={} sslmode={}",
        request.host,
        request.port,
        request.username,
        request.password,
        ssl_mode
    );

    let (client, conn) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await
        .map_err(|e| format!("Failed to connect to PostgreSQL server: {}", e))?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Build CREATE DATABASE command with options
    let mut create_db_sql = format!("CREATE DATABASE \"{}\"", request.new_database_name);
    let mut options = Vec::new();

    if let Some(encoding) = request.encoding {
        options.push(format!("ENCODING = '{}'", encoding));
    }

    if let Some(template) = request.template {
        options.push(format!("TEMPLATE = {}", template));
    }

    if let Some(owner) = request.owner {
        options.push(format!("OWNER = {}", owner));
    }

    if !options.is_empty() {
        create_db_sql.push_str(&format!(" WITH {}", options.join(" ")));
    }

    // Execute CREATE DATABASE
    client.execute(&create_db_sql, &[]).await
        .map_err(|e| {
            // Provide user-friendly error messages
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                format!("Database '{}' already exists", request.new_database_name)
            } else if error_msg.contains("permission denied") {
                format!("Permission denied: User '{}' does not have permission to create databases", request.username)
            } else {
                format!("Failed to create database: {}", e)
            }
        })?;

    Ok(format!("Database '{}' created successfully", request.new_database_name))
}

#[tauri::command]
async fn list_databases(
    host: String,
    port: u16,
    username: String,
    password: String,
    ssl: Option<bool>
) -> Result<Vec<String>, String> {
    let ssl_mode = if ssl.unwrap_or(false) { "require" } else { "disable" };

    let config = format!(
        "host={} port={} dbname=postgres user={} password={} sslmode={}",
        host, port, username, password, ssl_mode
    );

    let (client, conn) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Query for all databases the user can connect to
    let query = "SELECT datname FROM pg_database
                 WHERE datistemplate = false
                 AND has_database_privilege(datname, 'CONNECT')
                 ORDER BY datname";

    let rows = client.query(query, &[]).await
        .map_err(|e| format!("Failed to list databases: {}", e))?;

    let databases: Vec<String> = rows.iter()
        .map(|row| row.get::<_, String>(0))
        .collect();

    Ok(databases)
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
async fn execute_query(app: tauri::AppHandle, connection_id: String, sql: String, limit: Option<u32>) -> Result<serde_json::Value, String> {
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
    
    // Detect if this is a SELECT query or a DDL/DML statement
    // Remove comments and extra whitespace first
    let sql_cleaned = sql.lines()
        .map(|line| {
            // Remove single-line comments (-- comment)
            if let Some(pos) = line.find("--") {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_uppercase();

    let is_select = sql_cleaned.starts_with("SELECT") ||
                   sql_cleaned.starts_with("WITH") ||
                   sql_cleaned.starts_with("SHOW") ||
                   sql_cleaned.starts_with("EXPLAIN");
    
    let result_limit = limit.unwrap_or(1000); // Default limit of 1000 rows
    let mut results = Vec::new();
    let mut metadata = serde_json::Map::new();
    
    if is_select {
        // Use query() for SELECT statements that return rows
        let rows = client.query(&sql, &[]).await
            .map_err(|e| format!("Query execution failed: {}", e))?;
        
        let total_rows = rows.len();
        let limited_rows = if total_rows > result_limit as usize {
            &rows[0..result_limit as usize]
        } else {
            &rows[..]
        };
        
        // Add metadata about the results
        metadata.insert("total_rows".to_string(), serde_json::Value::Number(total_rows.into()));
        metadata.insert("returned_rows".to_string(), serde_json::Value::Number(limited_rows.len().into()));
        metadata.insert("limit_applied".to_string(), serde_json::Value::Bool(total_rows > result_limit as usize));
        metadata.insert("result_limit".to_string(), serde_json::Value::Number(result_limit.into()));
        
        // Convert rows to JSON - simplified approach
        for row in limited_rows {
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
    } else {
        // Use execute() for DDL/DML statements that don't return rows
        let affected_rows = client.execute(&sql, &[]).await
            .map_err(|e| format!("Query execution failed: {}", e))?;
        
        // Return a success message with affected row count
        let mut success_map = serde_json::Map::new();
        success_map.insert("status".to_string(), serde_json::Value::String("success".to_string()));
        success_map.insert("message".to_string(), serde_json::Value::String("Query executed successfully".to_string()));
        success_map.insert("affected_rows".to_string(), serde_json::Value::Number(affected_rows.into()));
        
        // Determine query type for better messaging
        let query_type = if sql_cleaned.starts_with("CREATE") {
            "CREATE"
        } else if sql_cleaned.starts_with("DROP") {
            "DROP"
        } else if sql_cleaned.starts_with("ALTER") {
            "ALTER"
        } else if sql_cleaned.starts_with("INSERT") {
            "INSERT"
        } else if sql_cleaned.starts_with("UPDATE") {
            "UPDATE"
        } else if sql_cleaned.starts_with("DELETE") {
            "DELETE"
        } else if sql_cleaned.starts_with("TRUNCATE") {
            "TRUNCATE"
        } else {
            "DDL/DML"
        };
        
        success_map.insert("query_type".to_string(), serde_json::Value::String(query_type.to_string()));
        results.push(serde_json::Value::Object(success_map));
    }
    
    // Return results with metadata
    let mut response = serde_json::Map::new();
    response.insert("results".to_string(), serde_json::Value::Array(results));
    response.insert("metadata".to_string(), serde_json::Value::Object(metadata));
    
    Ok(serde_json::Value::Object(response))
}

#[tauri::command]
async fn export_query_stream(
    app: tauri::AppHandle,
    connection_id: String,
    sql: String,
    output_path: String,
    format: String,
    options: serde_json::Value,
) -> Result<String, String> {
    println!("Streaming export to: {}", output_path);
    
    // Load connection details
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let connections: Vec<DatabaseConnection> = store.get("connections")
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    
    let connection = connections.iter()
        .find(|c| c.id == connection_id)
        .ok_or("Connection not found")?;
    
    // Decrypt password if needed
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
    
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Execute query and stream results to file
    let rows = client.query(&sql, &[]).await
        .map_err(|e| format!("Query execution failed: {}", e))?;
    
    // Create output file
    let file = File::create(&output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);
    
    let include_headers = options.get("includeHeaders")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let quote_all = options.get("quoteAllValues")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    if format == "csv" {
        // Write CSV
        if rows.len() > 0 && include_headers {
            // Write headers
            let headers: Vec<String> = rows[0].columns()
                .iter()
                .map(|col| col.name().to_string())
                .collect();
            writeln!(writer, "{}", headers.join(","))
                .map_err(|e| format!("Failed to write headers: {}", e))?;
        }
        
        // Write rows
        for row in &rows {
            let mut values = Vec::new();
            for (i, _column) in row.columns().iter().enumerate() {
                let value_str = if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                    v.unwrap_or_else(|| "NULL".to_string())
                } else if let Ok(v) = row.try_get::<_, Option<i32>>(i) {
                    v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".to_string())
                } else if let Ok(v) = row.try_get::<_, Option<i64>>(i) {
                    v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".to_string())
                } else if let Ok(v) = row.try_get::<_, Option<f64>>(i) {
                    v.map(|n| n.to_string()).unwrap_or_else(|| "NULL".to_string())
                } else if let Ok(v) = row.try_get::<_, Option<bool>>(i) {
                    v.map(|b| b.to_string()).unwrap_or_else(|| "NULL".to_string())
                } else {
                    "NULL".to_string()
                };
                
                // Quote value if needed
                if quote_all || value_str.contains(',') || value_str.contains('"') || value_str.contains('\n') {
                    values.push(format!("\"{}\"", value_str.replace("\"", "\"\"")));
                } else {
                    values.push(value_str);
                }
            }
            writeln!(writer, "{}", values.join(","))
                .map_err(|e| format!("Failed to write row: {}", e))?;
        }
    } else if format == "json" {
        // Write JSON
        let mut json_rows = Vec::new();
        for row in &rows {
            let mut row_map = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
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
                    serde_json::Value::Null
                };
                row_map.insert(column_name.to_string(), value);
            }
            json_rows.push(serde_json::Value::Object(row_map));
        }
        
        let json_str = serde_json::to_string_pretty(&json_rows)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        writer.write_all(json_str.as_bytes())
            .map_err(|e| format!("Failed to write JSON: {}", e))?;
    }
    
    writer.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;
    
    Ok(format!("Exported {} rows to {}", rows.len(), output_path))
}

#[tauri::command]
async fn export_query_native(
    app: tauri::AppHandle,
    connection_id: String,
    sql: String,
    output_path: String,
    format: String,
    include_headers: bool,
) -> Result<String, String> {
    println!("Native COPY TO export to: {}", output_path);
    
    // Load connection details
    let store = app.store_builder("connections.json").build()
        .map_err(|e| format!("Failed to build store: {}", e))?;
    
    let connections: Vec<DatabaseConnection> = store.get("connections")
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    
    let connection = connections.iter()
        .find(|c| c.id == connection_id)
        .ok_or("Connection not found")?;
    
    // Decrypt password if needed
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
    
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Build COPY TO command
    let copy_sql = if include_headers {
        format!("COPY ({}) TO STDOUT WITH (FORMAT CSV, HEADER)", sql)
    } else {
        format!("COPY ({}) TO STDOUT WITH (FORMAT CSV)", sql)
    };
    
    // Execute COPY TO and write to file
    let copy_reader = client.copy_out(&copy_sql).await
        .map_err(|e| format!("COPY TO failed: {}", e))?;
    
    let file = File::create(&output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);
    
    // Read all data from COPY
    use futures::pin_mut;
    use tokio_postgres::CopyOutStream;
    use futures::StreamExt;
    
    pin_mut!(copy_reader);
    let mut total_bytes = 0;
    
    while let Some(chunk_result) = copy_reader.next().await {
        let chunk = chunk_result
            .map_err(|e| format!("Failed to read COPY data: {}", e))?;
        writer.write_all(&chunk)
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        total_bytes += chunk.len();
    }
    
    writer.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;
    
    Ok(format!("Exported {} bytes to {}", total_bytes, output_path))
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
async fn get_table_create_statement(app: tauri::AppHandle, connection_id: String, table_name: String, schema_name: Option<String>) -> Result<String, String> {
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

    let (client, conn) = tokio_postgres::connect(&config, tokio_postgres::NoTls).await
        .map_err(|e| format!("Connection failed: {}", e))?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let schema_prefix = schema_name.as_ref().map(|s| format!("{}.", s)).unwrap_or_else(|| "public.".to_string());

    // Query for table columns
    let column_query = "
        SELECT
            c.column_name,
            c.udt_name,
            c.character_maximum_length,
            c.numeric_precision,
            c.numeric_scale,
            c.datetime_precision,
            c.is_nullable,
            c.column_default,
            c.data_type
        FROM information_schema.columns c
        WHERE c.table_schema = COALESCE($2, 'public')
            AND c.table_name = $1
        ORDER BY c.ordinal_position
    ";

    let rows = client.query(column_query, &[&table_name, &schema_name.as_ref().unwrap_or(&"public".to_string())]).await
        .map_err(|e| format!("Column query failed: {}", e))?;

    if rows.is_empty() {
        return Err(format!("Table '{}' not found", table_name));
    }

    let mut column_definitions = Vec::new();

    for row in rows {
        let column_name: String = row.get(0);
        let udt_name: String = row.get(1);
        let char_max_length: Option<i32> = row.get(2);
        let numeric_precision: Option<i32> = row.get(3);
        let numeric_scale: Option<i32> = row.get(4);
        let datetime_precision: Option<i32> = row.get(5);
        let is_nullable: String = row.get(6);
        let column_default: Option<String> = row.get(7);
        let data_type: String = row.get(8);

        // Build the data type string
        let mut type_string = match data_type.as_str() {
            "character varying" => {
                if let Some(len) = char_max_length {
                    format!("character varying({})", len)
                } else {
                    "character varying".to_string()
                }
            },
            "character" => {
                if let Some(len) = char_max_length {
                    format!("character({})", len)
                } else {
                    "character".to_string()
                }
            },
            "numeric" => {
                if let (Some(precision), Some(scale)) = (numeric_precision, numeric_scale) {
                    if scale > 0 {
                        format!("numeric({},{})", precision, scale)
                    } else {
                        format!("numeric({})", precision)
                    }
                } else {
                    "numeric".to_string()
                }
            },
            "timestamp without time zone" => {
                if let Some(precision) = datetime_precision {
                    if precision != 6 {
                        format!("timestamp({}) without time zone", precision)
                    } else {
                        "timestamp without time zone".to_string()
                    }
                } else {
                    "timestamp without time zone".to_string()
                }
            },
            "timestamp with time zone" => {
                if let Some(precision) = datetime_precision {
                    if precision != 6 {
                        format!("timestamp({}) with time zone", precision)
                    } else {
                        "timestamp with time zone".to_string()
                    }
                } else {
                    "timestamp with time zone".to_string()
                }
            },
            _ => data_type.clone()
        };

        // Quote column name if it contains special characters or is mixed case
        let quoted_column = if column_name.chars().any(|c| c.is_uppercase()) || column_name.contains(' ') {
            format!("\"{}\"", column_name)
        } else {
            column_name.clone()
        };

        let mut col_def = format!("  {} {}", quoted_column, type_string);

        // Add NOT NULL constraint
        if is_nullable == "NO" {
            col_def.push_str(" not null");
        } else {
            col_def.push_str(" null");
        }

        // Add default value if present
        if let Some(default) = column_default {
            // Clean up the default value (remove type casts for readability where appropriate)
            let cleaned_default = if default.starts_with("nextval(") {
                // Keep sequence defaults as-is
                default
            } else if default.ends_with("::character varying") {
                default.replace("::character varying", "")
            } else if default.ends_with("::text") {
                default.replace("::text", "")
            } else {
                default
            };
            col_def.push_str(&format!(" default {}", cleaned_default));
        }

        column_definitions.push(col_def);
    }

    // Query for constraints
    let constraint_query = "
        SELECT
            tc.constraint_name,
            tc.constraint_type,
            string_agg(kcu.column_name, ', ' ORDER BY kcu.ordinal_position) as columns,
            ccu.table_name as foreign_table,
            string_agg(ccu.column_name, ', ' ORDER BY kcu.ordinal_position) as foreign_columns,
            rc.update_rule,
            rc.delete_rule,
            cc.check_clause
        FROM information_schema.table_constraints tc
        LEFT JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        LEFT JOIN information_schema.constraint_column_usage ccu
            ON ccu.constraint_name = tc.constraint_name
            AND ccu.constraint_schema = tc.constraint_schema
            AND ccu.table_name != tc.table_name
        LEFT JOIN information_schema.referential_constraints rc
            ON rc.constraint_name = tc.constraint_name
            AND rc.constraint_schema = tc.constraint_schema
        LEFT JOIN information_schema.check_constraints cc
            ON cc.constraint_name = tc.constraint_name
            AND cc.constraint_schema = tc.constraint_schema
        WHERE tc.table_schema = COALESCE($2, 'public')
            AND tc.table_name = $1
        GROUP BY tc.constraint_name, tc.constraint_type, ccu.table_name, rc.update_rule, rc.delete_rule, cc.check_clause
        ORDER BY
            CASE tc.constraint_type
                WHEN 'PRIMARY KEY' THEN 1
                WHEN 'UNIQUE' THEN 2
                WHEN 'CHECK' THEN 3
                WHEN 'FOREIGN KEY' THEN 4
                ELSE 5
            END
    ";

    let constraint_rows = client.query(constraint_query, &[&table_name, &schema_name.as_ref().unwrap_or(&"public".to_string())]).await
        .map_err(|e| format!("Constraint query failed: {}", e))?;

    let mut constraints = Vec::new();

    for row in constraint_rows {
        let constraint_name: String = row.get(0);
        let constraint_type: String = row.get(1);
        let columns: Option<String> = row.get(2);
        let foreign_table: Option<String> = row.get(3);
        let foreign_columns: Option<String> = row.get(4);
        let _update_rule: Option<String> = row.get(5);
        let _delete_rule: Option<String> = row.get(6);
        let check_clause: Option<String> = row.get(7);

        match constraint_type.as_str() {
            "PRIMARY KEY" => {
                if let Some(cols) = columns {
                    constraints.push(format!("  constraint {} primary key ({})", constraint_name, cols));
                }
            },
            "UNIQUE" => {
                if let Some(cols) = columns {
                    constraints.push(format!("  constraint {} unique ({})", constraint_name, cols));
                }
            },
            "CHECK" => {
                if let Some(clause) = check_clause {
                    constraints.push(format!("  constraint {} check {}", constraint_name, clause));
                }
            },
            "FOREIGN KEY" => {
                if let (Some(cols), Some(ftable), Some(fcols)) = (columns, foreign_table, foreign_columns) {
                    constraints.push(format!("  constraint {} foreign key ({}) references {} ({})",
                        constraint_name, cols, ftable, fcols));
                }
            },
            _ => {}
        }
    }

    // Combine everything into CREATE TABLE statement
    let mut create_statement = format!("create table {}{} (\n", schema_prefix, table_name);

    let mut all_definitions = column_definitions;
    all_definitions.extend(constraints);

    create_statement.push_str(&all_definitions.join(",\n"));
    create_statement.push_str("\n) TABLESPACE pg_default;");

    Ok(create_statement)
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
        .plugin(tauri_plugin_dialog::init())
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
            create_database,
            list_databases,
            execute_query,
            connect_to_database,
            disconnect_from_database,
            update_last_connected,
            get_database_schema,
            get_table_columns,
            get_table_create_statement,
            save_window_state,
            restore_window_state,
            export_query_stream,
            export_query_native
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
