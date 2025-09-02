use crate::keymgmt::KeyManager;
use serde_json::json;

pub async fn anchor_super_mapping(km: &KeyManager, sbt_id: &str, address: &str, ouro_client: &OuroClient) -> Result<String, Error> {
    let payload = json!({
        "op":"anchor_super",
        "sbt_id": sbt_id,
        "address": address,
        "ts": chrono::Utc::now().to_rfc3339()
    });
    let signed = km.sign_payload("operator-anchor-key", payload.to_string().as_bytes()).await?;
    let resp = ouro_client.submit_anchor_tx(signed).await?;
    // resp contains tx_hash
    sqlx::query!("INSERT INTO anchors (anchor_id, anchor_type, payload, tx_hash) VALUES ($1,$2,$3,$4)",
        Uuid::new_v4(), "super_mapping", payload.to_string(), resp.tx_hash
    ).execute(&db_pool).await?;
    Ok(resp.tx_hash)
}
