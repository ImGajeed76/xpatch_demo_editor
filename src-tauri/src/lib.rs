use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, State};
use uuid::Uuid;
// Data

pub struct AppState {
    db: Mutex<Connection>,
    // Cache: (doc_uuid, patch_uuid) -> reconstructed content
    cache: Mutex<HashMap<(String, String), Vec<u8>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub uuid: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patch {
    pub uuid: String,
    pub document_uuid: String,
    pub timestamp: i64,
    pub delta: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentStats {
    pub total_patches: i64,
    pub total_delta_bytes: i64,
    pub total_uncompressed_bytes: i64,
    pub compression_ratio: f64,
}

// Setup

pub fn init_database(app: &tauri::App) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("xpatch.db");

    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS patches (
            uuid TEXT PRIMARY KEY,
            document_uuid TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            delta BLOB,
            FOREIGN KEY (document_uuid) REFERENCES documents(uuid)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_patches_doc_time
         ON patches(document_uuid, timestamp)",
        [],
    )?;

    Ok(conn)
}

// Commands

#[tauri::command]
fn load_document_at_timestamp(
    state: State<AppState>,
    doc_uuid: String,
    timestamp: i64,
) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let mut cache = state.cache.lock().unwrap();

    let mut stmt = db
        .prepare(
            "SELECT uuid, timestamp, delta
             FROM patches
             WHERE document_uuid = ? AND timestamp <= ?
             ORDER BY timestamp ASC",
        )
        .map_err(|e| e.to_string())?;

    let patches: Vec<Patch> = stmt
        .query_map(params![&doc_uuid, timestamp], |row| {
            Ok(Patch {
                uuid: row.get(0)?,
                document_uuid: doc_uuid.clone(),
                timestamp: row.get(1)?,
                delta: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if patches.is_empty() {
        return Ok("".to_string());
    }

    // Build a map of timestamps to patch content for quick lookup
    let mut content_by_timestamp: HashMap<i64, Vec<u8>> = HashMap::new();
    let mut sorted_timestamps: Vec<i64> = patches.iter().map(|p| p.timestamp).collect();
    sorted_timestamps.sort_unstable();

    for patch in patches {
        let cache_key = (doc_uuid.clone(), patch.uuid.clone());

        // Check cache first
        if let Some(cached_content) = cache.get(&cache_key) {
            content_by_timestamp.insert(patch.timestamp, cached_content.clone());
            continue;
        }

        // Extract the tag from the delta to know which base to use
        let tag = if let Some(ref delta) = patch.delta {
            xpatch::get_tag(delta).unwrap_or(0)
        } else {
            0
        };

        // Find the base content based on the tag
        let base_content = if tag == 0 {
            // tag 0 means use previous version (N-1)
            let pos = sorted_timestamps.iter().position(|&t| t == patch.timestamp).unwrap();
            if pos > 0 {
                content_by_timestamp
                    .get(&sorted_timestamps[pos - 1])
                    .cloned()
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            // tag N means use version N steps back
            let pos = sorted_timestamps.iter().position(|&t| t == patch.timestamp).unwrap();
            if pos > tag {
                content_by_timestamp
                    .get(&sorted_timestamps[pos - tag - 1])
                    .cloned()
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        };

        // Decode the delta
        let current_content = if let Some(delta) = patch.delta {
            xpatch::decode(&base_content, &delta)
                .map_err(|e| format!("Delta decode error: {:?}", e))?
        } else {
            base_content
        };

        content_by_timestamp.insert(patch.timestamp, current_content.clone());
        cache.insert(cache_key, current_content);
    }

    // Return the content at the requested timestamp
    let final_content = content_by_timestamp
        .get(sorted_timestamps.last().unwrap())
        .ok_or("Failed to reconstruct content")?;

    String::from_utf8(final_content.clone())
        .map_err(|e| format!("UTF-8 conversion error: {}", e))
}

fn find_optimal_base(
    state: &State<AppState>,
    doc_uuid: &str,
    current_timestamp: i64,
    new_content: &[u8],
    max_depth: usize,
    enable_zstd: bool,
) -> Result<(usize, Vec<u8>), String> {
    let db = state.db.lock().unwrap();

    // Get timestamps of previous versions
    let mut stmt = db
        .prepare(
            "SELECT DISTINCT timestamp
             FROM patches
             WHERE document_uuid = ? AND timestamp < ?
             ORDER BY timestamp DESC
             LIMIT ?",
        )
        .map_err(|e| e.to_string())?;

    let previous_timestamps: Vec<i64> = stmt
        .query_map(params![doc_uuid, current_timestamp, max_depth], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    drop(stmt);
    drop(db);

    if previous_timestamps.is_empty() {
        // No previous versions, encode against empty
        let delta = xpatch::encode(0, &[], new_content, enable_zstd);
        return Ok((0, delta));
    }

    // Try encoding against each previous version and find the smallest
    let mut best_tag = 0;
    let mut best_delta: Option<Vec<u8>> = None;
    let mut best_size = usize::MAX;

    for (tag, &timestamp) in previous_timestamps.iter().enumerate() {
        // Load the base version
        let base_content = load_document_at_timestamp(
            state.clone(),
            doc_uuid.to_string(),
            timestamp,
        )?;
        let base_bytes = base_content.as_bytes();

        // Encode against this base
        let delta = xpatch::encode(tag, base_bytes, new_content, enable_zstd);

        // Check if this is the best so far
        if delta.len() < best_size {
            best_size = delta.len();
            best_delta = Some(delta);
            best_tag = tag;
        }
    }

    Ok((best_tag, best_delta.unwrap()))
}

#[tauri::command]
fn create_patch(
    state: State<AppState>,
    doc_uuid: String,
    current_content: String,
    timestamp: i64,
) -> Result<String, String> {
    let new_content = current_content.as_bytes().to_vec();

    // Load the last content for comparison
    let last_content = load_document_at_timestamp(
        state.clone(),
        doc_uuid.clone(),
        timestamp,
    )?;

    // If content is identical, return early without creating a patch
    if last_content.as_bytes() == new_content {
        return Err("Content identical to last version - patch not created".to_string());
    }

    // Find the optimal base version to encode against
    // Try up to 16 previous versions (you can adjust this)
    let max_depth = 16;
    let enable_zstd = true;

    let (_best_tag, delta) = find_optimal_base(
        &state,
        &doc_uuid,
        timestamp,
        &new_content,
        max_depth,
        enable_zstd,
    )?;

    let db = state.db.lock().unwrap();
    let mut cache = state.cache.lock().unwrap();

    let patch_uuid = Uuid::new_v4().to_string();

    db.execute(
        "INSERT INTO patches (uuid, document_uuid, timestamp, delta) VALUES (?, ?, ?, ?)",
        params![&patch_uuid, &doc_uuid, timestamp, delta.as_slice()],
    )
        .map_err(|e| e.to_string())?;

    cache.insert((doc_uuid.clone(), patch_uuid.clone()), new_content);

    Ok(patch_uuid)
}

#[tauri::command]
fn create_document(state: State<AppState>, name: String) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let doc_uuid = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().timestamp_millis();

    db.execute(
        "INSERT INTO documents (uuid, name, created_at) VALUES (?, ?, ?)",
        params![&doc_uuid, &name, created_at],
    )
    .map_err(|e| e.to_string())?;

    Ok(doc_uuid)
}

#[tauri::command]
fn get_documents(state: State<AppState>) -> Result<Vec<Document>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT uuid, name, created_at FROM documents ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let docs = stmt
        .query_map([], |row| {
            Ok(Document {
                uuid: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(docs)
}

#[tauri::command]
fn get_patch_timestamps(
    state: State<AppState>,
    doc_uuid: String,
) -> Result<Vec<i64>, String> {
    let db = state.db.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT timestamp FROM patches WHERE document_uuid = ? ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let timestamps = stmt
        .query_map(params![&doc_uuid], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(timestamps)
}

#[tauri::command]
fn clear_cache(state: State<AppState>) -> Result<(), String> {
    let mut cache = state.cache.lock().unwrap();
    cache.clear();
    Ok(())
}

#[tauri::command]
fn get_document_stats(
    state: State<AppState>,
    doc_uuid: String,
) -> Result<DocumentStats, String> {
    let db = state.db.lock().unwrap();

    // Get total patches and delta size
    let (total_patches, total_delta_bytes): (i64, i64) = db
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(LENGTH(delta)), 0)
             FROM patches
             WHERE document_uuid = ?",
            params![&doc_uuid],
            |row| Ok((row.get(0)?, row.get(1)?))
        )
        .map_err(|e| e.to_string())?;

    // Get all timestamps
    let mut stmt = db
        .prepare("SELECT timestamp FROM patches WHERE document_uuid = ? ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let timestamps: Vec<i64> = stmt
        .query_map(params![&doc_uuid], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    drop(stmt);
    drop(db);

    // Calculate actual uncompressed size by reconstructing each version
    let mut total_uncompressed_bytes = 0i64;

    for timestamp in timestamps {
        let content = load_document_at_timestamp(
            state.clone(),
            doc_uuid.clone(),
            timestamp,
        )?;
        total_uncompressed_bytes += content.len() as i64;
    }

    // Calculate true compression ratio
    let compression_ratio = if total_delta_bytes > 0 {
        total_uncompressed_bytes as f64 / total_delta_bytes as f64
    } else {
        1.0
    };

    Ok(DocumentStats {
        total_patches,
        total_delta_bytes,
        total_uncompressed_bytes,
        compression_ratio,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = init_database(app)?;
            app.manage(AppState {
                db: Mutex::new(conn),
                cache: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_document_at_timestamp,
            create_patch,
            create_document,
            get_documents,
            get_patch_timestamps,
            clear_cache,
            get_document_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
