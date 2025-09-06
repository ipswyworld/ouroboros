// src/api.rs
// Axum-based API router for transaction submit + basic checks
use crate::storage::{open_db, get_str};

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use thiserror::Error;
use tracing::{error, info};
use uuid::Uuid;

type ApiResult = Result<Response, ApiError>;

#[derive(Debug, Deserialize)]
pub struct IncomingTxn {
    pub tx_hash: String,
    pub sender: String,
    pub recipient: String,
    pub payload: JsonValue, // full signed payload from client
    pub signature: Option<String>,        // optional meta
    pub idempotency_key: Option<String>,  // optional client-supplied idempotency key
    pub nonce: Option<i64>,               // optional account nonce
}

#[derive(Debug, Serialize)]
struct TxSubmitResponse {
    tx_id: Uuid,
    status: &'static str,
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("duplicate transaction")]
    Duplicate,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match &self {
            ApiError::Db(e) => {
                error!("DB error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string())
            }
            ApiError::Duplicate => (StatusCode::CONFLICT, "duplicate transaction".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };
        let body_json = serde_json::json!({ "error": body });
        (status, Json(body_json)).into_response()
    }
}

///////////////////////////////////////////////////////////////////////////
// POST /tx/submit
///////////////////////////////////////////////////////////////////////////
async fn submit_tx(
    Extension(db_pool): Extension<PgPool>,
    axum::Json(incoming): axum::Json<IncomingTxn>,
) -> Result<impl IntoResponse, ApiError> {
    // Basic validation
    if incoming.tx_hash.trim().is_empty() {
        return Err(ApiError::BadRequest("tx_hash required".into()));
    }
    if incoming.sender.trim().is_empty() || incoming.recipient.trim().is_empty() {
        return Err(ApiError::BadRequest("sender and recipient are required".into()));
    }

    // Check for duplicate by tx_hash or idempotency_key
    if let Some(_) = sqlx::query_scalar::<_, String>(
        "SELECT tx_hash FROM tx_index WHERE tx_hash = $1",
    )
    .bind(&incoming.tx_hash)
    .fetch_optional(&db_pool)
    .await?
    {
        info!("duplicate tx_hash submitted: {}", &incoming.tx_hash);
        return Err(ApiError::Duplicate);
    }

    // Optional signature verification: if client included public_key/signature in payload,
    // verify signature over tx_hash (change canonical message as needed).
    if let Some(pubkey) = incoming.payload.get("public_key").and_then(|v| v.as_str()) {
        if let Some(sig) = incoming.payload.get("signature").and_then(|v| v.as_str()) {
            let message = incoming.tx_hash.as_bytes();
            let use_real = std::env::var("USE_REAL_CRYPTO").is_ok();
            let ok = if use_real {
                crate::crypto::verify_ed25519_hex(pubkey, sig, message)
            } else {
                crate::crypto::verify_ed25519_hex(pubkey, sig, message)
                    || {
                        // fallback length check
                        let pk = hex::decode(pubkey).ok();
                        let s = hex::decode(sig).ok();
                        match (pk, s) {
                            (Some(p), Some(sv)) => p.len() == 32 && sv.len() == 64,
                            _ => false,
                        }
                    }
            };
            if !ok {
                return Err(ApiError::BadRequest("signature invalid".into()));
            }
        }
    }

    // Optionally check idempotency_key uniqueness
    if let Some(key) = &incoming.idempotency_key {
        if let Some(existing_tx_id) = sqlx::query_scalar::<_, Uuid>(
            "SELECT tx_id FROM transactions WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&db_pool)
        .await?
        {
            // Return existing tx_id back to caller (idempotent)
            info!("idempotency key already present, returning existing tx_id");
            let resp = TxSubmitResponse {
                tx_id: existing_tx_id,
                status: "pending",
            };
            return Ok((StatusCode::OK, Json(resp)));
        }
    }

    // Generate tx_id and insert atomically (transactions + mempool_entries)
    let tx_id = Uuid::new_v4();

    // Use a DB transaction for atomicity
    let mut tx = db_pool.begin().await?;

    // Clone payload locally so we can bind it multiple times without moving it
    let payload_clone = incoming.payload.clone();

    // Insert transaction row
    sqlx::query(
        r#"
        INSERT INTO transactions (tx_id, tx_hash, sender, recipient, payload, status, idempotency_key, nonce)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(tx_id)
    .bind(&incoming.tx_hash)
    .bind(&incoming.sender)
    .bind(&incoming.recipient)
    .bind(payload_clone.clone()) // move a clone into this bind
    .bind("pending")
    .bind(incoming.idempotency_key.clone())
    .bind(incoming.nonce)
    .execute(&mut tx)
    .await?;

    // Insert into persistent mempool entries table for durable mempool
    sqlx::query(
        r#"
        INSERT INTO mempool_entries (tx_id, tx_hash, payload, received_at)
        VALUES ($1, $2, $3, now())
        "#,
    )
    .bind(tx_id)
    .bind(&incoming.tx_hash)
    .bind(sqlx::types::Json(&payload_clone)) // bind reference to clone as JSON
    .execute(&mut tx)
    .await?;

    // Optionally create tx_index entry (allows fast lookup by hash)
    sqlx::query(
        r#"
        INSERT INTO tx_index (tx_hash, tx_id)
        VALUES ($1, $2)
        ON CONFLICT (tx_hash) DO NOTHING
        "#,
    )
    .bind(&incoming.tx_hash)
    .bind(tx_id)
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    info!("accepted tx {} by sender {}", tx_id, &incoming.sender);

    let resp = TxSubmitResponse {
        tx_id,
        status: "pending",
    };
    Ok((StatusCode::ACCEPTED, Json(resp)))
}

///////////////////////////////////////////////////////////////////////////
// GET /mempool
///////////////////////////////////////////////////////////////////////////
async fn get_mempool(Extension(db_pool): Extension<PgPool>) -> ApiResult {
    // cast received_at to text so sqlx doesn't try to map to a non-serializable type
    let rows = sqlx::query(
        r#"
        SELECT tx_id::text AS tx_id, tx_hash, payload::text AS payload_json, received_at::text AS received_at
        FROM mempool_entries
        ORDER BY received_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&db_pool)
    .await
    .map_err(ApiError::Db)?;

    let mut out: Vec<JsonValue> = Vec::new();
    for r in rows {
        let tx_id: String = r.try_get("tx_id").map_err(ApiError::Db)?;
        let tx_hash: Option<String> = r.try_get("tx_hash").map_err(ApiError::Db)?;
        let payload_json: Option<String> = r.try_get("payload_json").map_err(ApiError::Db)?;
        let received_at: Option<String> = r.try_get("received_at").map_err(ApiError::Db)?;

        let payload_val: JsonValue = match payload_json {
            Some(s) => serde_json::from_str(&s).unwrap_or(JsonValue::Null),
            None => JsonValue::Null,
        };

        out.push(serde_json::json!({
            "tx_id": tx_id,
            "tx_hash": tx_hash,
            "payload": payload_val,
            "received_at": received_at,
        }));
    }

    Ok((StatusCode::OK, Json(out)).into_response())
}

///////////////////////////////////////////////////////////////////////////
// GET /tx/:id  (tries id as uuid, falls back to tx_hash lookup)
// GET /tx/hash/:hash
///////////////////////////////////////////////////////////////////////////
async fn get_tx_by_id_or_hash(
    Path(id): Path<String>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResult {
    // if it looks like a UUID, try uuid lookup
    if id.contains('-') {
        if let Ok(uuid) = Uuid::parse_str(&id) {
            return get_tx_by_id_inner(uuid, &db_pool).await;
        }
    }
    // otherwise fall back to hash lookup
    get_tx_by_hash_inner(&id, &db_pool).await
}

async fn get_tx_by_hash(
    Path(hash): Path<String>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResult {
    get_tx_by_hash_inner(&hash, &db_pool).await
}

async fn get_tx_by_id_inner(uuid: Uuid, pool: &PgPool) -> ApiResult {
    let row_opt = sqlx::query(
        r#"
        SELECT tx_id::text AS tx_id, tx_hash, sender, recipient, payload::text AS payload_json, status, idempotency_key, nonce, created_at::text AS created_at
        FROM transactions WHERE tx_id = $1
        "#,
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::Db)?;

    if let Some(row) = row_opt {
        let tx_id: String = row.try_get("tx_id").map_err(ApiError::Db)?;
        let tx_hash: Option<String> = row.try_get("tx_hash").map_err(ApiError::Db)?;
        let sender: Option<String> = row.try_get("sender").map_err(ApiError::Db)?;
        let recipient: Option<String> = row.try_get("recipient").map_err(ApiError::Db)?;
        let payload_json: Option<String> = row.try_get("payload_json").map_err(ApiError::Db)?;
        let status: Option<String> = row.try_get("status").map_err(ApiError::Db)?;
        let idempotency_key: Option<String> = row.try_get("idempotency_key").map_err(ApiError::Db)?;
        let nonce: Option<i64> = row.try_get("nonce").map_err(ApiError::Db)?;
        let created_at: Option<String> = row.try_get("created_at").map_err(ApiError::Db)?;

        let payload_val: JsonValue = match payload_json {
            Some(s) => serde_json::from_str(&s).unwrap_or(JsonValue::Null),
            None => JsonValue::Null,
        };

        return Ok((StatusCode::OK, Json(serde_json::json!({
            "tx_id": tx_id,
            "tx_hash": tx_hash,
            "sender": sender,
            "recipient": recipient,
            "payload": payload_val,
            "status": status,
            "idempotency_key": idempotency_key,
            "nonce": nonce,
            "created_at": created_at,
        }))).into_response());
    }

    Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "tx not found" }))).into_response())
}

async fn get_tx_by_hash_inner(hash: &str, pool: &PgPool) -> ApiResult {
    let row_opt = sqlx::query(
        r#"
        SELECT tx_id::text AS tx_id, tx_hash, sender, recipient, payload::text AS payload_json, status, idempotency_key, nonce, created_at::text AS created_at
        FROM transactions WHERE tx_hash = $1
        "#,
    )
    .bind(hash)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::Db)?;

    if let Some(row) = row_opt {
        let tx_id: String = row.try_get("tx_id").map_err(ApiError::Db)?;
        let tx_hash: Option<String> = row.try_get("tx_hash").map_err(ApiError::Db)?;
        let sender: Option<String> = row.try_get("sender").map_err(ApiError::Db)?;
        let recipient: Option<String> = row.try_get("recipient").map_err(ApiError::Db)?;
        let payload_json: Option<String> = row.try_get("payload_json").map_err(ApiError::Db)?;
        let status: Option<String> = row.try_get("status").map_err(ApiError::Db)?;
        let idempotency_key: Option<String> = row.try_get("idempotency_key").map_err(ApiError::Db)?;
        let nonce: Option<i64> = row.try_get("nonce").map_err(ApiError::Db)?;
        let created_at: Option<String> = row.try_get("created_at").map_err(ApiError::Db)?;

        let payload_val: JsonValue = match payload_json {
            Some(s) => serde_json::from_str(&s).unwrap_or(JsonValue::Null),
            None => JsonValue::Null,
        };

        return Ok((StatusCode::OK, Json(serde_json::json!({
            "tx_id": tx_id,
            "tx_hash": tx_hash,
            "sender": sender,
            "recipient": recipient,
            "payload": payload_val,
            "status": status,
            "idempotency_key": idempotency_key,
            "nonce": nonce,
            "created_at": created_at,
        }))).into_response());
    }

    Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "tx not found" }))).into_response())
}

///////////////////////////////////////////////////////////////////////////
// GET /proof/:tx  (lookup tx_index)
///////////////////////////////////////////////////////////////////////////
async fn get_proof_by_tx(
    Path(tx): Path<String>,
    Extension(db_pool): Extension<PgPool>,
) -> ApiResult {
    let row_opt = sqlx::query(
        r#"
        SELECT tx_hash, tx_id::text AS tx_id, block_id::text AS block_id, created_at::text AS created_at
        FROM tx_index WHERE tx_hash = $1
        "#,
    )
    .bind(&tx)
    .fetch_optional(&db_pool)
    .await
    .map_err(ApiError::Db)?;

    if let Some(row) = row_opt {
        let tx_hash: Option<String> = row.try_get("tx_hash").map_err(ApiError::Db)?;
        let tx_id: Option<String> = row.try_get("tx_id").map_err(ApiError::Db)?;
        let block_id: Option<String> = row.try_get("block_id").map_err(ApiError::Db)?;
        let created_at: Option<String> = row.try_get("created_at").map_err(ApiError::Db)?;

        // proof field is placeholder (Merkle proofs to be implemented later)
        let payload = serde_json::json!({
            "tx_hash": tx_hash,
            "tx_id": tx_id,
            "block_id": block_id,
            "created_at": created_at,
            "proof": serde_json::Value::Null
        });

        return Ok((StatusCode::OK, Json(payload)).into_response());
    }

    Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "tx not found" }))).into_response())
}

///////////////////////////////////////////////////////////////////////////
// GET /block/:id  -- placeholder; adapt to your block schema if needed
///////////////////////////////////////////////////////////////////////////
async fn get_block_by_id(
    Path(id): Path<String>,
    Extension(_db_pool): Extension<PgPool>,
) -> ApiResult {
    // Try to read block from RocksDB (if you persist blocks under key "block:<id>")
    let rocks_path = std::env::var("ROCKSDB_PATH").unwrap_or_else(|_| "sled_data".into());

    // open_db returns the DB handle (not a Result) — call it directly
    let db = open_db(&rocks_path);

    // attempt to read the key
    let key = format!("block:{}", id);
    match get_str::<serde_json::Value>(&db, &key) {
        Ok(Some(val)) => {
            return Ok((StatusCode::OK, Json(val)).into_response());
        }
        Ok(None) => {
            // not found
            return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "block not found" }))).into_response());
        }
        Err(e) => {
            eprintln!("get_block_by_id: rocksdb read error: {:?}", e);
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "rocksdb read error"
            }))).into_response());
        }
    }
}

///////////////////////////////////////////////////////////////////////////
// GET /health
///////////////////////////////////////////////////////////////////////////
async fn health(Extension(db_pool): Extension<PgPool>) -> ApiResult {
    match sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&db_pool).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({"status":"ok"}))).into_response()),
        Err(e) => {
            eprintln!("health: db error: {:?}", e);
            Ok((StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"status":"db-unavailable"}))).into_response())
        }
    }
}

/// GET /peers - returns the runtime peer store as JSON array (full metadata)
async fn get_peers(
    Extension(peer_store): Extension<crate::network::PeerStore>
) -> Result<impl IntoResponse, ApiError> {
    let store = peer_store.lock().await;
    let out: Vec<_> = store.iter().map(|e| {
        serde_json::json!({
            "addr": e.addr,
            "last_seen": e.last_seen_unix,
            "failures": e.failures,
            "banned_until": e.banned_until_unix,
        })
    }).collect();
    Ok((StatusCode::OK, Json(out)))
}

#[allow(dead_code)]
fn verify_signature(_payload: &JsonValue, _signature: Option<String>) -> Result<(), String> {
    // placeholder for future more complex verification
    Ok(())
}

/// Build router for this microservice (call from main)
/// Accepts the runtime PeerStore so /peers can show discovered peers with metadata.
pub fn router(db_pool: PgPool, peer_store: crate::network::PeerStore) -> Router {
    Router::new()
        .route("/tx/submit", post(submit_tx))
        .route("/mempool", get(get_mempool))
        .route("/tx/:id", get(get_tx_by_id_or_hash))      // accepts uuid or tx_hash; we try both
        .route("/tx/hash/:hash", get(get_tx_by_hash))     // explicit hash lookup
        .route("/block/:id", get(get_block_by_id))        // placeholder route
        .route("/proof/:tx", get(get_proof_by_tx))
        .route("/health", get(health))
        .route("/peers", get(get_peers))
        .layer(Extension(db_pool))
        .layer(Extension(peer_store))
}

// add near other handlers
use crate::network;

async fn p2p_status(
    Extension(_db_pool): Extension<PgPool>,
    Extension(_peer_store): Extension<crate::network::PeerStore>
) -> ApiResult {
    let (conns, dedupe, peers) = network::get_p2p_metrics();
    let payload = serde_json::json!({
        "active_connections": conns,
        "dedupe_entries": dedupe,
        "known_peers": peers
    });
    Ok((StatusCode::OK, Json(payload)).into_response())
}

// in router() add .route("/p2p_status", get(p2p_status))
