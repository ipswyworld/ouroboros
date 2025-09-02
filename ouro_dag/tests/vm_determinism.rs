// tests/vm_determinism.rs
use tempfile::TempDir;
use uuid::Uuid;
use chrono::{Utc, TimeZone};
use ouro_dag::dag::transaction::Transaction;
use ouro_dag::vm;
use ouro_dag::storage::open_db;

fn make_tx(id: Uuid) -> Transaction {
    Transaction {
        id,
        sender: "alice".into(),
        recipient: "bob".into(),
        amount: 10,
        // Use fixed timestamp for deterministic test
        timestamp: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
        parents: vec![],
        signature: "00".into(),
        public_key: "00".into(),
        payload: None,
    }
}

#[test]
fn vm_is_deterministic_for_fixed_input() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();

    let db1 = open_db(dir1.path().to_str().unwrap());
    let db2 = open_db(dir2.path().to_str().unwrap());

    let txs = vec![make_tx(Uuid::new_v4()), make_tx(Uuid::new_v4())];

    let res1 = vm::execute_contracts(&db1, &txs);
    assert!(res1.is_ok());

    let res2 = vm::execute_contracts(&db2, &txs);
    assert!(res2.is_ok());

    // Replace these key names with keys your VM actually writes deterministically.
    let v1 = db1.get(b"balance:alice").unwrap();
    let v2 = db2.get(b"balance:alice").unwrap();
    assert_eq!(v1, v2, "VM produced different state on two runs -> non-deterministic");
}
