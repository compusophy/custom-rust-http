use axum::{extract::Path, extract::State, Json};
use crate::state::AppState;
use crate::types::{CreateRecordRequest, DbListResponse, DbRecord, DbResponse};

pub async fn db_create(
    State(state): State<AppState>,
    Json(payload): Json<CreateRecordRequest>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] POST /api/db - Creating record");

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    // Generate ID
    let id = format!("db:{}", uuid::Uuid::new_v4().to_string());
    let now = chrono::Utc::now().to_rfc3339();

    let record = DbRecord {
        id: id.clone(),
        data: payload.data,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    let record_json =
        serde_json::to_string(&record).map_err(|e| format!("Failed to serialize record: {}", e))?;

    redis::cmd("SET")
        .arg(&id)
        .arg(&record_json)
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| format!("Failed to SET record: {}", e))?;

    // Add to index set
    redis::cmd("SADD")
        .arg("db:index")
        .arg(&id)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to add to index: {}", e))?;

    Ok(Json(DbResponse {
        success: true,
        record: Some(record),
        message: "Record created successfully".to_string(),
    }))
}

pub async fn db_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] GET /api/db/{} - Reading record", id);

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    // Handle both "db:uuid" format and just "uuid" format
    let key = if id.starts_with("db:") {
        id.clone()
    } else {
        format!("db:{}", id)
    };

    let record_json: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to GET record: {}", e))?;

    match record_json {
        Some(json) => {
            let mut record: DbRecord =
                serde_json::from_str(&json).map_err(|e| format!("Failed to parse record: {}", e))?;
            // Ensure ID matches the requested format
            record.id = key.clone();
            Ok(Json(DbResponse {
                success: true,
                record: Some(record),
                message: "Record found".to_string(),
            }))
        }
        None => Ok(Json(DbResponse {
            success: false,
            record: None,
            message: "Record not found".to_string(),
        })),
    }
}

pub async fn db_update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateRecordRequest>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] PUT /api/db/{} - Updating record", id);

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    // Handle both "db:uuid" format and just "uuid" format
    let key = if id.starts_with("db:") {
        id.clone()
    } else {
        format!("db:{}", id)
    };

    // Check if record exists
    let existing_json: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to GET record: {}", e))?;

    let (created_at, updated_at) = match existing_json {
        Some(json) => {
            let existing: DbRecord =
                serde_json::from_str(&json).map_err(|e| format!("Failed to parse existing record: {}", e))?;
            (existing.created_at, Some(chrono::Utc::now().to_rfc3339()))
        }
        None => (
            Some(chrono::Utc::now().to_rfc3339()),
            Some(chrono::Utc::now().to_rfc3339()),
        ),
    };

    let record = DbRecord {
        id: id.clone(),
        data: payload.data,
        created_at,
        updated_at,
    };

    let record_json =
        serde_json::to_string(&record).map_err(|e| format!("Failed to serialize record: {}", e))?;

    redis::cmd("SET")
        .arg(&key)
        .arg(&record_json)
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| format!("Failed to SET record: {}", e))?;

    // Ensure in index
    redis::cmd("SADD")
        .arg("db:index")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to add to index: {}", e))?;

    Ok(Json(DbResponse {
        success: true,
        record: Some(record),
        message: "Record updated successfully".to_string(),
    }))
}

pub async fn db_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] DELETE /api/db/{} - Deleting record", id);

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    // Handle both "db:uuid" format and just "uuid" format
    let key = if id.starts_with("db:") {
        id.clone()
    } else {
        format!("db:{}", id)
    };

    // Check if exists
    let exists: bool = redis::cmd("EXISTS")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to check existence: {}", e))?;

    if !exists {
        return Ok(Json(DbResponse {
            success: false,
            record: None,
            message: "Record not found".to_string(),
        }));
    }

    redis::cmd("DEL")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to DELETE record: {}", e))?;

    // Remove from index
    redis::cmd("SREM")
        .arg("db:index")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to remove from index: {}", e))?;

    Ok(Json(DbResponse {
        success: true,
        record: None,
        message: "Record deleted successfully".to_string(),
    }))
}

pub async fn db_list(State(state): State<AppState>) -> Result<Json<DbListResponse>, String> {
    println!("[API] GET /api/db - Listing all records");

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    // Get all IDs from index
    let ids: Vec<String> = redis::cmd("SMEMBERS")
        .arg("db:index")
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to get index: {}", e))?;

    let mut records = Vec::new();

    for id in ids {
        let record_json: Option<String> = redis::cmd("GET")
            .arg(&id)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Failed to GET record {}: {}", id, e))?;

        if let Some(json) = record_json {
            if let Ok(record) = serde_json::from_str::<DbRecord>(&json) {
                records.push(record);
            }
        }
    }

    let count = records.len();
    Ok(Json(DbListResponse {
        success: true,
        records,
        count,
    }))
}


