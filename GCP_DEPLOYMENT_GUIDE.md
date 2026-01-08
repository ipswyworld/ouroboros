# Ouroboros GCP Deployment Guide

## Current Status

**Old Deployment (PostgreSQL-based):**
- Instance: `ouro-node-1`
- Machine: e2-small
- IP: 136.112.101.176
- Status: RUNNING

**New Deployment (RocksDB-based):**
- Instance: `ouro-node-rocksdb` (to be created)
- Machine: e2-medium (upgraded for better performance)
- Persistent disk: 50GB for blockchain data

## Quick Deployment

### Step 1: Deploy New Instance

```bash
deploy-to-gcp.bat
```

This will:
- Create a persistent disk for RocksDB data (50GB)
- Set up firewall rules (ports 8000, 9000)
- Create a new e2-medium instance
- Install Docker
- Clone repository and build
- Start the Ouroboros node

**Time**: ~5-10 minutes

### Step 2: Verify New Instance

Check if the new node is running:

```bash
gcloud compute ssh ouro-node-rocksdb --zone=us-central1-a --command="docker logs ouro-node"
```

Test the API:
```bash
curl http://<NEW_EXTERNAL_IP>:8000/health
```

### Step 3: Remove Old Instance

Once the new instance is verified working:

```bash
remove-old-deployment.bat
```

This will stop and delete `ouro-node-1`.

## Manual Deployment Steps

If you prefer manual control:

### 1. Build Docker Image Locally (Optional)

```bash
cd ouro_dag
docker build -t ouroboros-node .
docker run -p 8000:8000 -p 9000:9000 ouroboros-node
```

### 2. Deploy to GCP

```bash
# Set project
gcloud config set project ultimate-flame-407206

# Create persistent disk
gcloud compute disks create ouro-node-rocksdb-data \
    --size=50GB \
    --zone=us-central1-a \
    --type=pd-standard

# Create firewall rules
gcloud compute firewall-rules create ouro-p2p --allow=tcp:9000
gcloud compute firewall-rules create ouro-api --allow=tcp:8000

# Create instance (see deploy-to-gcp.bat for full command)
```

### 3. SSH and Check Status

```bash
gcloud compute ssh ouro-node-rocksdb --zone=us-central1-a

# Inside the VM:
docker ps
docker logs ouro-node
```

## Cost Comparison

**Old Instance (e2-small):**
- vCPUs: 2 (shared)
- Memory: 2 GB
- Cost: ~$12/month

**New Instance (e2-medium):**
- vCPUs: 2 (shared)
- Memory: 4 GB
- Cost: ~$24/month
- Persistent disk (50GB): ~$8/month
- **Total**: ~$32/month

**Savings from removing PostgreSQL:**
- No Cloud SQL costs (~$50-100/month saved)
- **Net savings**: ~$30-70/month

## Architecture

### Old (PostgreSQL):
```
VM (e2-small) → Cloud SQL (PostgreSQL) → Storage
```

### New (RocksDB):
```
VM (e2-medium) → Local RocksDB → Persistent Disk
```

**Benefits:**
- ✓ Simpler architecture (single instance)
- ✓ Lower costs (no Cloud SQL)
- ✓ Better performance (local storage)
- ✓ Easier backup (disk snapshots)
- ✓ No network latency to database

## Monitoring

### View Logs
```bash
gcloud compute ssh ouro-node-rocksdb --zone=us-central1-a \
  --command="docker logs -f ouro-node"
```

### Check Resource Usage
```bash
gcloud compute ssh ouro-node-rocksdb --zone=us-central1-a \
  --command="docker stats"
```

### Create Snapshot (Backup)
```bash
gcloud compute disks snapshot ouro-node-rocksdb-data \
  --zone=us-central1-a \
  --snapshot-names=ouro-backup-$(date +%Y%m%d)
```

## Troubleshooting

### Node won't start
```bash
gcloud compute ssh ouro-node-rocksdb --zone=us-central1-a
sudo docker logs ouro-node
sudo docker ps -a
```

### Disk full
```bash
# Check disk usage
df -h

# Expand disk if needed
gcloud compute disks resize ouro-node-rocksdb-data \
  --size=100GB \
  --zone=us-central1-a
```

### Performance issues
Upgrade to larger instance:
```bash
gcloud compute instances stop ouro-node-rocksdb --zone=us-central1-a
gcloud compute instances set-machine-type ouro-node-rocksdb \
  --machine-type=e2-standard-2 \
  --zone=us-central1-a
gcloud compute instances start ouro-node-rocksdb --zone=us-central1-a
```

## Cleanup All Resources

To completely remove everything:

```bash
# Delete instance
gcloud compute instances delete ouro-node-rocksdb --zone=us-central1-a

# Delete disk
gcloud compute disks delete ouro-node-rocksdb-data --zone=us-central1-a

# Delete firewall rules
gcloud compute firewall-rules delete ouro-p2p
gcloud compute firewall-rules delete ouro-api
```

## Next Steps

After deployment:
1. Update your DNS records to point to new IP
2. Set up monitoring/alerting
3. Schedule automatic disk snapshots
4. Configure log aggregation (optional)
5. Test P2P connectivity
6. Verify API endpoints

## Support

For issues, check:
- `docker logs ouro-node` - Application logs
- `journalctl -u docker` - Docker service logs
- `/var/log/syslog` - System logs
