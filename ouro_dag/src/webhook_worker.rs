// src/webhook_worker.rs (sketch)
use crate::sled_storage::RocksDb;
use reqwest::Client;
use std::thread;
use std::time::Duration;

pub fn start_worker(db: RocksDb) {
    let db = db.clone();
    thread::spawn(move || {
        let client = Client::new();
        loop {
            // pop one item from webhook queue: iterate prefix "webhook_q:"
            if let Ok(list) = crate::sled_storage::iter_prefix_kv::<serde_json::Value>(&db, "webhook_q:") {
                for (k, v) in list {
                    // each key like webhook_q:<uuid>
                    let url = v.get("url").and_then(|u| u.as_str()).unwrap_or("");
                    let body = v.get("body").cloned().unwrap_or(serde_json::Value::Null);
                    let resp = client.post(url).json(&body).send();
                    match resp {
                        Ok(r) if r.status().is_success() => {
                            // remove queue item
                            let _ = crate::sled_storage::delete(&db, k);
                        }
                        _ => {
                            // leave item for retry and rely on backoff via sleep
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}
