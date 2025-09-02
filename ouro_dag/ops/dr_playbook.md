DR Playbook (high level)

RTO / RPO targets:
- Validators: RTO = 5 mins for relay, 15-60 mins for full restore; RPO = 1 min of block data desirable
- Postgres: RTO = 15-30 mins; RPO = 5 mins (WAL shipping)
- RocksDB: RTO = 30 mins; RPO = 5-15 mins (snapshot cadence)

Backups to S3:
- Postgres daily dump + hourly WAL archive -> s3://ouro-backups/postgres/
- RocksDB daily snapshot (checkpoint) -> s3://ouro-backups/rocksdb/

Failover steps:
1. Promote standby Postgres read-replica (if running).
2. Restore RocksDB snapshot to /data/rocksdb on new node.
3. Start Ouroboros with --recover --replay-wal options.
4. Monitor blocks and run indexer consistency check.

DR drills:
- Monthly: restore from last backup to staging and verify state.
- Quarterly: full chain restore end-to-end, including validators.

Contacts: ops@yourcompany, oncall pager.
