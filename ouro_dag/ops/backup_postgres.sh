#!/usr/bin/env bash
set -e
TS=$(date -u +%Y%m%dT%H%M%SZ)
PGHOST=localhost
PGUSER=ouro
PGDATABASE=ouro_db
BACKUP_DIR=/backups
mkdir -p $BACKUP_DIR
pg_dump -h $PGHOST -U $PGUSER -F c -b -v -f ${BACKUP_DIR}/ouro_db_${TS}.dump $PGDATABASE
echo "Postgres dump saved to ${BACKUP_DIR}/ouro_db_${TS}.dump"
