# Ouroboros Blockchain - RocksDB Edition
# Multi-stage build for optimized production deployment

# Build stage
FROM rust:1.83 as builder

# Install RocksDB native dependencies
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    cmake \
    libsnappy-dev \
    zlib1g-dev \
    libbz2-dev \
    libgflags-dev \
    liblz4-dev \
    libzstd-dev \
    libssl-dev \
    pkg-config \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ouro_dag
COPY . .

# Build optimized release binary
RUN cargo build --release --bin ouro_dag

# Runtime stage - minimal footprint
FROM debian:bookworm-slim

# Install only runtime dependencies (no PostgreSQL!)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsnappy1v5 \
    libbz2-1.0 \
    liblz4-1 \
    libzstd1 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /usr/src/ouro_dag/target/release/ouro_dag /app/ouro_dag

# Copy configuration
COPY --from=builder /usr/src/ouro_dag/.env.example /app/.env.example

# Create data directory for RocksDB
RUN mkdir -p /data/rocksdb
VOLUME ["/data"]

# Environment variables
ENV ROCKSDB_PATH=/data/rocksdb
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose ports
EXPOSE 9000 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# Run as non-root user
RUN useradd -m -u 1000 ouro && chown -R ouro:ouro /app /data
USER ouro

CMD ["./ouro_dag"]
