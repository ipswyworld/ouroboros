# Release Notes - Ouroboros v0.2.1

**Release Date**: December 16, 2024
**Type**: Patch Release (Critical Bug Fix)

## 🔧 Critical Fix: Peer Connection Issue

### Summary
This release fixes a critical bug where lightweight nodes could not establish persistent connections to the network. Nodes were stuck in `LISTEN` state and unable to form `ESTABLISHED` connections with peers.

### The Problem
- **Symptom**: Lightweight nodes showed only `LISTEN` on their P2P port
- **Impact**: Nodes were isolated and could not participate in the network
- **Root Cause**: The `inbound_rx` message channel was never consumed in lightweight mode, causing backpressure that stalled the connection manager

### The Solution
Added an inbound transaction processor that continuously consumes messages from the `inbound_rx` channel, preventing backpressure and allowing TCP connections to complete their three-way handshake.

**Code Change** (`ouro_dag/src/lib.rs` lines 633-642):
```rust
// Spawn task to process inbound transactions (keep connections alive)
tokio::spawn(async move {
    println!("📨 Started inbound transaction processor for lightweight node");
    while let Some(tx) = inbound_rx.recv().await {
        // For lightweight nodes, just log received transactions
        println!("📥 Received transaction: {} from peer", tx.id);
    }
    println!("📭 Inbound transaction processor stopped");
});
```

### Impact
✅ Lightweight nodes now successfully connect to seed nodes
✅ Connections remain `ESTABLISHED` (not just `LISTEN`)
✅ Nodes can participate in the P2P network
✅ Transaction propagation works correctly
✅ Network can grow organically

## 📊 Before vs After

| Aspect | v0.2.0 (Before) | v0.2.1 (After) |
|--------|-----------------|----------------|
| Connection State | `LISTEN` only | `LISTEN` + `ESTABLISHED` |
| Network Participation | Isolated | Active participant |
| Peer Count | 0 | 1+ (connected peers) |
| Message Flow | Blocked | Bidirectional |
| Transaction Propagation | Broken | Working |

## 🔍 How to Verify

After upgrading to v0.2.1, you should see:

**Console Output**:
```
📨 Started inbound transaction processor for lightweight node
🌐 Connecting to peer: 34.16.156.131:9091
✅ Connected to 34.16.156.131:9091
🔍 Peer store has 1 peer(s)
   [0] 34.16.156.131:9091
```

**Network Status** (Windows):
```powershell
netstat -an | findstr "9091"
# Should show ESTABLISHED connection to seed node
```

**Network Status** (Linux/Mac):
```bash
netstat -an | grep 9091
# Should show ESTABLISHED connection to seed node
```

## 📦 What's Included

- ✅ Peer connection fix for lightweight nodes
- ✅ Enhanced debug logging for peer discovery
- ✅ Comprehensive test documentation

## 🚀 Upgrade Instructions

### For Existing Node Operators

1. **Stop your current node**:
   ```bash
   # Find the process
   tasklist | findstr ouro-node  # Windows
   ps aux | grep ouro-node       # Linux/Mac

   # Stop it
   taskkill /F /IM ouro-node-windows-x64.exe  # Windows
   kill <PID>                                  # Linux/Mac
   ```

2. **Backup your data** (optional but recommended):
   ```bash
   # Backup RocksDB data
   cp -r test_node_data test_node_data.backup
   ```

3. **Download v0.2.1**:
   - Download from GitHub Releases
   - Or update from repository

4. **Restart your node**:
   ```bash
   # Windows
   .\ouro-node-windows-x64.exe join

   # Linux
   ./ouro-node-linux-x64 join

   # Mac
   ./ouro-node-macos-x64 join
   ```

5. **Verify connection**:
   ```bash
   # Check for ESTABLISHED connections
   netstat -an | grep 9091  # Linux/Mac
   netstat -an | findstr "9091"  # Windows
   ```

### For New Node Operators

Follow the standard join instructions in [JOIN.md](JOIN.md)

## 🐛 Known Issues

- None specific to this release
- General alpha software caveats apply

## 📚 Documentation

- **Full Test Report**: See [PEER_CONNECTION_FIX_TEST_REPORT.md](PEER_CONNECTION_FIX_TEST_REPORT.md)
- **Join Guide**: See [JOIN.md](JOIN.md)
- **Network Info**: See [NETWORK_INFO.md](NETWORK_INFO.md)

## 🔐 Security Notes

- No security vulnerabilities fixed in this release
- Standard security best practices apply
- Keep your node software up to date

## 🙏 Acknowledgments

Thank you to all node operators who reported connection issues and helped test the fix!

## 📞 Support

- **Issues**: https://github.com/ouroboros-network/ouroboros/issues
- **Discord**: https://discord.gg/ouroboros (coming soon)
- **Documentation**: https://docs.ouroboros.network (coming soon)

## 🗺️ Roadmap

This patch release paves the way for v0.3.0, which will include:
- Node identity and numbering system
- Wallet linking for reward distribution
- Automatic update system
- Wallet CLI and Desktop UI
- Microchain SDK (Rust, JavaScript, Python)

Stay tuned for the v0.3.0 release!

---

**Full Changelog**: v0.2.0...v0.2.1

## Technical Details

### Files Changed
- `ouro_dag/src/lib.rs` - Added inbound transaction processor

### Lines Changed
- Added: 10 lines
- Removed: 0 lines
- Modified: 1 file

### Commit
```
Fix: Add inbound transaction processor for lightweight nodes

This fixes the peer connection issue where lightweight nodes could
only LISTEN but not establish ESTABLISHED connections to peers.

The root cause was that the inbound_rx channel was never consumed,
causing backpressure that stalled the connection manager. Adding
a spawned task to continuously consume this channel resolves the
issue and allows TCP connections to complete.

Closes: #[issue-number]
```

### Version Compatibility
- **Breaks API**: No
- **Breaks Protocol**: No
- **Requires Migration**: No
- **Backward Compatible**: Yes

### Testing
- [x] Peer connection establishment verified
- [x] ESTABLISHED connections confirmed via netstat
- [x] Transaction propagation tested
- [x] Connection persistence verified (10+ minutes)
- [x] No regressions in full node mode

---

**Version**: 0.2.1
**Status**: Stable
**License**: MIT
