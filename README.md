# Ouroboros DAG

Ouroboros is a lightweight, experimental DAG-based distributed ledger.

## Building and Running

The project is built with Rust and can be run directly using Cargo.

### Prerequisites

- Rust toolchain (stable)
- Docker (for running the PostgreSQL database)
- `psql` client

### Running a Single Node

1.  **Start the database:**
    ```bash
    docker compose up -d postgres
    ```

2.  **Navigate to the `ouro_dag` directory:**
    ```bash
    cd ouro_dag
    ```

3.  **Set the database URL:**
    ```bash
    export DATABASE_URL="postgres://ouro:ouro_pass@127.0.0.1:15432/ouro_db"
    ```

4.  **Run the main node:**
    ```bash
    cargo run --release --bin ouro_dag
    ```

### Running a Second Node

1.  **Open a new terminal.**

2.  **Navigate to the `ouro_dag` directory:**
    ```bash
    cd ouro_dag
    ```

3.  **Set the environment variables for the second node:**
    ```bash
    export API_ADDR="0.0.0.0:8002"
    export LISTEN_ADDR="0.0.0.0:9002"
    export PEER_ADDRS="127.0.0.1:9001"
    export ROCKSDB_PATH="sled_data_2"
    ```

4.  **Run the second node:**
    ```bash
    cargo run --release --bin ouro-node -- join --peer 127.0.0.1:9001 --api-port 8002 --p2p-port 9002
    ```

## Testing

To run the tests, navigate to the `ouro_dag` directory and run:

```bash
cargo test
```

## Releases

This project uses GitHub Actions to automatically build and release binaries for Linux, Windows, and macOS.

When a new version is tagged (e.g., `v0.1.0`), the workflow will create a new GitHub Release and attach the compiled binaries as downloadable artifacts.

### Installer Scripts

Installer scripts are available to easily download and install the latest release.

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/your-org/ouroboros/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iex (Invoke-RestMethod -Uri https://raw.githubusercontent.com/your-org/ouroboros/main/install.ps1)
```
