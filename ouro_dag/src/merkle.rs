use sha2::{Sha256, Digest};

pub struct MerkleTree {
    nodes: Vec<Vec<[u8; 32]>>, // levels
}

impl MerkleTree {
    pub fn from_hashes(hashes: &[String]) -> Self {
        let mut level: Vec<[u8; 32]> = hashes.iter().map(|h| {
            let mut d = [0u8; 32];
            d.copy_from_slice(&hex::decode(h).expect("bad hash")[..32]);
            d
        }).collect();
        let mut nodes = vec![level.clone()];
        while nodes.last().unwrap().len() > 1 {
            let prev = nodes.last().unwrap();
            let mut next = vec![];
            for i in (0..prev.len()).step_by(2) {
                let left = prev[i];
                let right = if i+1 < prev.len() { prev[i+1] } else { prev[i] };
                let mut hasher = Sha256::new();
                hasher.update(&left);
                hasher.update(&right);
                let res = hasher.finalize();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&res[..32]);
                next.push(arr);
            }
            nodes.push(next);
        }
        MerkleTree { nodes }
    }

    pub fn root_hex(&self) -> String {
        if let Some(root_level) = self.nodes.last() {
            hex::encode(root_level[0])
        } else {
            hex::encode([0u8;32])
        }
    }

    // inclusion proof returns Vec<(sibling_hash_hex, is_left)>
    pub fn proof_for_index(&self, index: usize) -> Vec<(String, bool)> {
        let mut proof = vec![];
        let mut idx = index;
        for level in &self.nodes {
            if level.len() == 1 { break; }
            let sibling_index = if idx %2 == 0 { idx + 1 } else { idx - 1 };
            let sibling = if sibling_index < level.len() { level[sibling_index] } else { level[idx] };
            proof.push((hex::encode(sibling), idx % 2 == 0)); // is_left = true if current is left
            idx /= 2;
        }
        proof
    }
}
