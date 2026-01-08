# Ouroboros - Quick Command Reference

## Simple Commands (from project root)

### Testing
```bash
test.bat          # Run all unit tests
```
Or from `ouro_dag/` directory:
```bash
cargo t           # Run tests (short alias)
cargo ta          # Run all tests including integration
```

### Building
```bash
build.bat         # Build release version
check.bat         # Quick compilation check
```
Or from `ouro_dag/` directory:
```bash
cargo b           # Build (short alias)
cargo br          # Build release
cargo c           # Check only
```

### Running
```bash
run.bat           # Run the node (release mode)
```
Or from `ouro_dag/` directory:
```bash
cargo r           # Run (short alias)
cargo rr          # Run release
```

### Code Quality
From `ouro_dag/` directory:
```bash
cargo lint        # Run clippy linter
cargo fix         # Auto-fix issues
cargo fmt-all     # Format all code
```

### Cleanup
```bash
cleanup.bat       # Remove unnecessary files and build artifacts
```
Or from `ouro_dag/` directory:
```bash
cargo clean-all   # Clean build artifacts
```

## Full Command Reference

All cargo aliases defined in `ouro_dag/.cargo/config.toml`:

| Alias | Full Command | Description |
|-------|-------------|-------------|
| `t` | `test --lib` | Run library tests |
| `ta` | `test --all` | Run all tests |
| `b` | `build` | Build debug |
| `br` | `build --release` | Build release |
| `c` | `check` | Quick check |
| `r` | `run` | Run debug |
| `rr` | `run --release` | Run release |
| `lint` | `clippy -- -D warnings` | Lint code |
| `fix` | `fix --allow-dirty` | Auto-fix |
| `fmt-all` | `fmt --all` | Format code |
| `clean-all` | `clean` | Clean build |
| `d` | `doc --no-deps --open` | Build docs |

## Project Structure

```
ouroboros/
├── ouro_dag/           # Main blockchain implementation
│   ├── src/           # Source code
│   ├── tests/         # Integration tests
│   └── Cargo.toml     # Dependencies
├── ouro_sdk/          # SDK for developers
├── ouro_wallet/       # Wallet implementation
├── test.bat           # Quick test
├── build.bat          # Quick build
├── run.bat            # Quick run
└── cleanup.bat        # Cleanup script
```

## Common Workflows

### Development Cycle
```bash
# 1. Make changes to code
# 2. Quick check
check.bat

# 3. Run tests
test.bat

# 4. Build release
build.bat
```

### Before Commit
```bash
cd ouro_dag
cargo fmt-all          # Format code
cargo lint             # Check for issues
cargo t                # Run tests
```

### Clean Build
```bash
cleanup.bat            # Remove all artifacts
build.bat              # Fresh build
```

## Notes

- All `.bat` files run from the project root
- Cargo aliases work from the `ouro_dag/` directory
- Use `cargo help <alias>` to see what an alias does
