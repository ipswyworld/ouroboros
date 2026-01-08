// tests/merkle_helper.rs
use ouro_dag::crypto::merkle::merkle_root_from_leaves_bytes;
use sha2::{Sha256, Digest};

#[test]
fn test_merkle_root_single_leaf() {
    let leaf = vec![1, 2, 3, 4];
    let leaves = vec![leaf.clone()];
    let expected_root = Sha256::digest(&leaf).to_vec();
    let root = merkle_root_from_leaves_bytes(&leaves).unwrap();
    assert_eq!(root, expected_root);
}

#[test]
fn test_merkle_root_two_leaves() {
    let leaf1 = vec![1, 2, 3, 4];
    let leaf2 = vec![5, 6, 7, 8];
    let leaves = vec![leaf1.clone(), leaf2.clone()];

    let mut hasher = Sha256::new();
    hasher.update(Sha256::digest(&leaf1));
    hasher.update(Sha256::digest(&leaf2));
    let expected_root = hasher.finalize().to_vec();

    let root = merkle_root_from_leaves_bytes(&leaves).unwrap();
    assert_eq!(root, expected_root);
}

#[test]
fn test_merkle_root_three_leaves() {
    let leaf1 = vec![1, 2, 3, 4];
    let leaf2 = vec![5, 6, 7, 8];
    let leaf3 = vec![9, 10, 11, 12];
    let leaves = vec![leaf1.clone(), leaf2.clone(), leaf3.clone()];

    // Expected root calculation: (H(L1) + H(L2)) + (H(L3) + H(L3))
    let h1 = Sha256::digest(&leaf1);
    let h2 = Sha256::digest(&leaf2);
    let h3 = Sha256::digest(&leaf3);

    let mut h12_hasher = Sha256::new();
    h12_hasher.update(h1);
    h12_hasher.update(h2);
    let h12 = h12_hasher.finalize().to_vec();

    let mut h33_hasher = Sha256::new();
    h33_hasher.update(h3);
    h33_hasher.update(h3);
    let h33 = h33_hasher.finalize().to_vec();

    let mut final_hasher = Sha256::new();
    final_hasher.update(h12);
    final_hasher.update(h33);
    let expected_root = final_hasher.finalize().to_vec();

    let root = merkle_root_from_leaves_bytes(&leaves).unwrap();
    assert_eq!(root, expected_root);
}

#[test]
fn test_merkle_root_empty_leaves() {
    let leaves: Vec<Vec<u8>> = vec![];
    let expected_root = Sha256::digest(&[]).to_vec();
    let root = merkle_root_from_leaves_bytes(&leaves).unwrap();
    assert_eq!(root, expected_root);
}
