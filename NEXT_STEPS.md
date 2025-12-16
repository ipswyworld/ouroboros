# Immediate Next Steps - Ouroboros Development

## 🚀 Current Status

✅ **Completed**:
- Peer connection fix implemented and committed
- `source-code` branch created with full source
- GitHub Actions CI/CD workflow created
- v0.3.0 roadmap finalized (includes Microchain SDK)
- Development task list created

🔄 **In Progress**:
- GitHub Actions building v0.2.1 binaries (Linux, Windows, macOS)
- Build URL: https://github.com/ipswyworld/ouroboros/actions

---

## 📋 Immediate Tasks (Next 24-48 Hours)

### 1. Complete v0.2.1 Release ⭐ HIGH PRIORITY

**Wait for GitHub Actions build to complete** (~15-20 minutes)

Once complete:
```bash
# Download artifacts from GitHub Actions
# Test on Windows
.\ouro-node-windows-x64.exe join --peer 136.112.101.176:9001 --storage rocksdb --rocksdb-path test_data --api-port 8001 --p2p-port 9001

# Verify in logs:
# 🔍 PEER_ADDRS env var: '136.112.101.176:9001'
# 🔍 Peer store has 1 peer(s):
# 📨 Started inbound transaction processor

# Verify in netstat:
netstat -an | findstr "9001"
# Should show ESTABLISHED connection to 136.112.101.176:9001
```

**If successful**:
1. Create git tag: `git tag v0.2.1 -m "Fix: Peer connections for lightweight nodes"`
2. Push tag: `git push origin v0.2.1`
3. GitHub Actions will automatically create release with binaries
4. Update join scripts to use v0.2.1

**Files to update**:
- `join_ouroboros.ps1` - Change download URL to v0.2.1
- `join_ouroboros.sh` - Change download URL to v0.2.1
- `README.md` - Update installation instructions

---

### 2. Set Up Development Environment for v0.3.0

**Create workspace structure**:
```bash
ouroboros/
├── ouro_dag/           # Node implementation (exists)
├── ouro_wallet/        # NEW - Wallet implementation
├── ouro_sdk/           # NEW - Microchain SDK
│   ├── rust/           # Rust SDK
│   ├── js/             # JavaScript/TypeScript SDK
│   └── python/         # Python SDK
└── docs/               # NEW - Documentation
```

**Initialize new crates**:
```bash
cd ouroboros
cargo new --lib ouro_wallet
cargo new --lib ouro_sdk

# Create workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "ouro_dag",
    "ouro_wallet",
    "ouro_sdk"
]
resolver = "2"
EOF
```

---

### 3. Start Phase 1: Node Identity System

**Implementation Plan** (6 hours):

**A. Create node identity module** (`ouro_dag/src/node_identity.rs`):
```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeIdentity {
    pub node_number: u64,
    pub node_id: String,
    pub first_joined: String,
    pub public_name: Option<String>,
}

impl NodeIdentity {
    pub fn load_or_create(path: &Path) -> Result<Self, Box<dyn Error>> {
        if path.exists() {
            let json = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&json)?)
        } else {
            let identity = Self::generate_new();
            identity.save(path)?;
            Ok(identity)
        }
    }

    fn generate_new() -> Self {
        Self {
            node_number: Self::assign_node_number(),
            node_id: uuid::Uuid::new_v4().to_string(),
            first_joined: chrono::Utc::now().to_rfc3339(),
            public_name: None,
        }
    }

    fn assign_node_number() -> u64 {
        // TODO: Get from network consensus or hash-based assignment
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        uuid::Uuid::new_v4().hash(&mut hasher);
        hasher.finish() % 1_000_000 // Node numbers 0-999,999
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
```

**B. Integrate into lightweight mode** (`ouro_dag/src/lib.rs`):
```rust
// After RocksDB init
let identity_path = std::env::var("IDENTITY_PATH")
    .unwrap_or_else(|_| format!("{}/.node_identity.json", db_path));
let identity = NodeIdentity::load_or_create(Path::new(&identity_path))?;

println!("🆔 Node Identity:");
println!("   Node Number: #{}", identity.node_number);
println!("   Node ID: {}", identity.node_id);
if let Some(name) = &identity.public_name {
    println!("   Name: {}", name);
}
```

**C. Add API endpoints**:
```rust
// GET /identity
async fn get_identity(
    State(identity): State<Arc<NodeIdentity>>
) -> Json<NodeIdentity> {
    Json(identity.as_ref().clone())
}

// POST /identity/name
async fn set_name(
    State(identity): State<Arc<Mutex<NodeIdentity>>>,
    Json(payload): Json<SetNameRequest>,
) -> Result<Json<NodeIdentity>, StatusCode> {
    let mut id = identity.lock().await;
    id.public_name = Some(payload.name);
    id.save(Path::new(&payload.path))?;
    Ok(Json(id.clone()))
}
```

**Testing**:
1. Start node, verify identity is created
2. Restart node, verify same identity is loaded
3. Call `GET /identity` API, verify response
4. Call `POST /identity/name`, verify name is saved

**Estimated time**: 6 hours
- Core implementation: 3h
- API endpoints: 2h
- Testing: 1h

---

## 📊 v0.3.0 Development Schedule

### Week 1-2: Foundation
- [ ] Day 1-2: Node Identity & Number (6h)
- [ ] Day 3-4: Wallet Linking System (10h)
- [ ] Day 5: Testing & Documentation (4h)

### Week 3: Automation
- [ ] Day 1-3: Automatic Update System (12h)
- [ ] Day 4-5: Testing & Refinement (8h)

### Week 4-5: Wallet
- [ ] Week 4: Wallet CLI (15h)
- [ ] Week 5: Wallet Desktop UI (15h)

### Week 6-8: Microchain SDK
- [ ] Week 6: Rust SDK + Core (15h)
- [ ] Week 7: JS/TS SDK (12h)
- [ ] Week 8: Python SDK + Docs (13h)

**Total**: 8 weeks part-time OR 2.5 weeks full-time

---

## 🎯 Success Metrics

**v0.2.1**:
- ✅ Peer connections stay ESTABLISHED in netstat
- ✅ Inbound transactions are logged
- ✅ Node runs stable for 24+ hours

**v0.3.0 Phase 1**:
- ✅ Every node has persistent identity
- ✅ Nodes can link to wallet addresses
- ✅ Identity shown in logs and API

**v0.3.0 Phase 2**:
- ✅ Nodes check for updates daily
- ✅ Auto-update on restart
- ✅ Rollback if update fails

**v0.3.0 Phase 3**:
- ✅ CLI wallet can create/import/export
- ✅ Desktop wallet has GUI
- ✅ Can send/receive OURO tokens
- ✅ Can link to running node

**v0.3.0 Phase 4**:
- ✅ SDK works in Rust/JS/Python
- ✅ Can create microchains via SDK
- ✅ Can submit transactions to microchains
- ✅ Documentation with 3+ examples

---

## 🤔 Decision Points

1. **Node Number Assignment**:
   - Option A: Hash-based (deterministic, no coordination needed)
   - Option B: Sequential (requires validator coordination)
   - **Recommendation**: Hash-based for MVP, sequential later

2. **Wallet Tech Stack**:
   - CLI: Pure Rust with `clap` + `tui-rs`
   - Desktop: Tauri (Rust backend + HTML/CSS/JS frontend)
   - **Recommendation**: Both in parallel (shared Rust core)

3. **SDK Language Priority**:
   - Rust first (native integration)
   - Then JS/TS (web/Node.js developers)
   - Then Python (data science/ML developers)
   - **Recommendation**: This order makes sense

4. **Update Strategy**:
   - Download from GitHub Releases API
   - Verify SHA256 checksum
   - Restart service automatically
   - **Recommendation**: Manual approval first, auto later

---

## 📝 Next Session Plan

When you return:

1. **Check build status**: https://github.com/ipswyworld/ouroboros/actions
2. **Download and test** v0.2.1 binary with peer connection fix
3. **If tests pass**: Create v0.2.1 release tag
4. **Start Phase 1**: Implement node identity system
5. **Set up workspace**: Create ouro_wallet and ouro_sdk crates

**Questions to resolve**:
- Preferred wallet UI framework? (Tauri vs Electron vs native)
- SDK documentation format? (mdBook vs docs.rs vs separate site)
- Update check frequency? (Daily vs on startup vs configurable)

Ready to start Phase 1 implementation once v0.2.1 is validated! 🚀
