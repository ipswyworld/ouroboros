#!/usr/bin/env bash
set -e
FILE=$1
if [ -z "$FILE" ]; then
  echo "Usage: restore_postgres.sh /path/to/dump"
  exit 1
fi
pg_restore -h localhost -U ouro -d ouro_db -v $FILE
