// src/storage.rs
// sled-backed persistent helpers (aliased as RocksDb)

use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use sled;
use std::sync::Arc;
use std::time::Duration;
use std::thread::sleep;
use once_cell::sync::OnceCell;

/// Type alias kept as `RocksDb` so code that expects RocksDb compiles unchanged.
/// It's actually sled::Db under the hood (Arc so clone cheap).
pub type RocksDb = Arc<sled::Db>;

/// Global storage instance (initialized once during startup)
static GLOBAL_DB: OnceCell<RocksDb> = OnceCell::new();

/// Initialize the global storage instance.
pub fn init_global_storage(db: RocksDb) {
    let _ = GLOBAL_DB.set(db);
}

/// Get the global storage instance.
pub fn get_global_storage() -> Option<RocksDb> {
    GLOBAL_DB.get().cloned()
}

/// Open DB with retry/backoff (helps on transient locks)
pub fn open_db(path: &str) -> RocksDb {
    let mut attempt = 0u32;
    let max_attempts = 8u32;
    let mut wait = 250u64;
    loop {
        match sled::open(path) {
            Ok(db) => return Arc::new(db),
            Err(e) => {
                attempt += 1;
                if attempt >= max_attempts {
                    panic!("Failed to open sled DB at '{}': {} (attempts={})", path, e, attempt);
                }
                eprintln!("open_db attempt {}/{} failed: {} — retrying in {}ms", attempt, max_attempts, e, wait);
                sleep(Duration::from_millis(wait));
                wait = std::cmp::min(wait * 2, 2000);
            }
        }
    }
}

/// Put a serializable value under a byte-key.
pub fn put<K: AsRef<[u8]>, V: Serialize>(db: &RocksDb, key: K, val: &V) -> Result<(), String> {
    let bytes = serde_json::to_vec(val).map_err(|e| e.to_string())?;
    db.insert(key, bytes).map_err(|e| e.to_string())?;
    // flush to disk (optional but safer)
    db.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get and deserialize a value stored under a byte-key.
pub fn get<K: AsRef<[u8]>, T: DeserializeOwned>(db: &RocksDb, key: K) -> Result<Option<T>, String> {
    match db.get(key).map_err(|e| e.to_string())? {
        Some(ivec) => {
            let v = serde_json::from_slice::<T>(&ivec).map_err(|e| e.to_string())?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

/// Delete a key
pub fn delete<K: AsRef<[u8]>>(db: &RocksDb, key: K) -> Result<(), String> {
    db.remove(key).map_err(|e| e.to_string())?;
    db.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Batch put: apply multiple (key, value) entries atomically.
pub fn batch_put<V: Serialize>(db: &RocksDb, entries: Vec<(Vec<u8>, V)>) -> Result<(), String> {
    let mut batch = sled::Batch::default();
    for (k, v) in entries.into_iter() {
        let b = serde_json::to_vec(&v).map_err(|e| e.to_string())?;
        batch.insert(k, b);
    }
    db.apply_batch(batch).map_err(|e| e.to_string())?;
    db.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Iterate values whose keys start with the given prefix and deserialize them into Vec<T>.
pub fn iter_prefix<T: DeserializeOwned>(db: &RocksDb, prefix: &[u8]) -> Result<Vec<T>, String> {
    let mut out = Vec::new();
    let iter = db.scan_prefix(prefix);
    for item in iter {
        let (_k, v) = item.map_err(|e| e.to_string())?;
        let obj = serde_json::from_slice::<T>(&v).map_err(|e| e.to_string())?;
        out.push(obj);
    }
    Ok(out)
}

/// Iterate prefix returning (key_string, value) pairs — useful for listing entries and knowing keys.
pub fn iter_prefix_kv<T: DeserializeOwned>(db: &RocksDb, prefix: &str) -> Result<Vec<(String, T)>, String> {
    let mut out = Vec::new();
    let iter = db.scan_prefix(prefix.as_bytes());
    for item in iter {
        let (k, v) = item.map_err(|e| e.to_string())?;
        let kstr = String::from_utf8_lossy(&k).to_string();
        let obj = serde_json::from_slice::<T>(&v).map_err(|e| e.to_string())?;
        out.push((kstr, obj));
    }
    Ok(out)
}

/// Convenience helpers (string-key versions) so callers can use &str keys without .into_bytes()
pub fn put_str<V: Serialize>(db: &RocksDb, key: &str, val: &V) -> Result<(), String> {
    put(db, key.as_bytes(), val)
}

pub fn get_str<T: DeserializeOwned>(db: &RocksDb, key: &str) -> Result<Option<T>, String> {
    get(db, key.as_bytes())
}

#[cfg(feature = "rocksdb")]
pub mod rocksdb_helper {
    use rocksdb::{Options, DB};
    use num_cpus;

    /// Open a RocksDB instance with tuned options for high-performance local state.
    pub fn open_rocksdb(path: &str) -> DB {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.set_max_background_jobs(4);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_total_wal_size(512 * 1024 * 1024);
        opts.set_level0_file_num_compaction_trigger(8);
        DB::open(&opts, path).unwrap()
    }
}
