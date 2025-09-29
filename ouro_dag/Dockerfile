# Build stage
FROM rust:1.85 as builder

# Install RocksDB native dependencies (if building with rocksdb)
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential cmake libsnappy-dev zlib1g-dev libbz2-dev libgflags-dev liblz4-dev libzstd-dev \
    libssl-dev pkg-config curl ca-certificates librocksdb-dev postgresql-client && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ouro_dag
COPY . .

# Build release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libsnappy1v5 libbz2-1.0 liblz4-1 libzstd1 postgresql-client && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# copy binary
COPY --from=builder /usr/src/ouro_dag/target/release/ouro_dag /app/ouro_dag
# copy migrations for convenience
COPY --from=builder /usr/src/ouro_dag/migrations ./migrations
# copy scripts
COPY --from=builder /usr/src/ouro_dag/scripts ./scripts
RUN chmod +x ./scripts/run_migrations.sh || true
RUN chmod +x ./ops/wait-for-db-and-start.sh || true

# Create data dir for RocksDB
VOLUME ["/data"]
ENV ROCKSDB_PATH=/data/rocksdb

EXPOSE 9000
EXPOSE 8000

CMD ["./ouro_dag"]
