# Peer Connection Issue - Root Cause & Fix

## Problem
Lightweight nodes (RocksDB-only mode) are NOT establishing persistent connections to seed nodes, even though:
- PEER_ADDRS environment variable is set correctly
- Seed node address appears in peers.json
- Connection manager background task is running

## Root Cause

**File**: `ouro_dag/src/lib.rs` (lines 618-652)

The lightweight mode implementation:
1. ✅ Calls `start_network()` - spawns connection manager task
2. ✅ Gets back `(bcast_sender, inbound_rx, peer_store)`
3. ❌ **NEVER USES** `inbound_rx` - doesn't process incoming transactions
4. ❌ Immediately blocks on API server `.await`

**Why this breaks connections:**
- Connection manager attempts outbound connections every 5 seconds
- Handshake completes, peer sends transactions
- But `inbound_rx` is never consumed, channel fills up
- Connection likely stalls or closes

## The Fix

**Added at line 634-643**:
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

**Added debugging (lines 621-632)**:
- Shows PEER_ADDRS environment variable
- Lists all peers in peer_store with connection status
- Shows failures count and last_seen timestamp

## Testing

After rebuilding with this fix:
1. Start node with join command
2. Check logs for:
   - `🔍 PEER_ADDRS env var: '136.112.101.176:9001'`
   - `🔍 Peer store has 1 peer(s):`
   - `📨 Started inbound transaction processor`
3. Run `netstat` - should see ESTABLISHED connection to seed node

## Files Modified
- `ouro_dag/src/lib.rs` - Added inbound message processing + debugging

## Next Steps
1. Build new binary with fix
2. Test locally
3. Verify ESTABLISHED connection appears in netstat
4. Commit and push to GitHub
5. Update v0.2.0 release or create v0.2.1
