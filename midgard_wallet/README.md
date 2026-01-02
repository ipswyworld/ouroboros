# Midgard Wallet

A simple CLI wallet for the OVM (Ouroboros) blockchain written in Rust.

## Features

- **Generate new wallets** with BIP39 12-word mnemonic phrases
- **Import wallets** from mnemonic phrase or private key
- **Check balance** for your address
- **Send OURO tokens** to other addresses
- **View wallet information**
- **Check blockchain status**

## Installation

Build the wallet from source:

```bash
cd midgard_wallet
cargo build --release
```

The compiled binary will be located at `target/release/midgard-wallet.exe` (on Windows) or `target/release/midgard-wallet` (on Linux/Mac).

## Usage

### Create a New Wallet

```bash
midgard-wallet create --name "My Wallet"
```

This will generate a new wallet with a 12-word mnemonic phrase. **Save the mnemonic phrase securely** - it's the only way to recover your wallet!

### Import a Wallet

From mnemonic phrase:
```bash
midgard-wallet import --mnemonic "your twelve word mnemonic phrase goes here like this example phrase" --name "Imported Wallet"
```

From private key:
```bash
midgard-wallet import --private-key "your_private_key_hex" --name "Imported Wallet"
```

### View Wallet Information

```bash
midgard-wallet info
```

Shows your wallet address, public key, name, and creation date.

### Check Balance

```bash
midgard-wallet balance
```

Fetches your current OURO balance from the blockchain.

### Send OURO Tokens

```bash
midgard-wallet send <recipient_address> <amount> --fee 1000
```

Example:
```bash
# Nonce is automatically fetched from blockchain
midgard-wallet send ouro1abc123... 1000000000000 --fee 1000

# Or manually specify nonce
midgard-wallet send ouro1abc123... 1000000000000 --fee 1000 --nonce 5
```

**Notes:**
- Amount is in the smallest units (1 OURO = 1,000,000,000,000 units)
- Nonce is automatically fetched from the blockchain (optional override with `--nonce`)
- Chain ID is automatically set to "ouroboros-mainnet-1"

### Check Blockchain Status

```bash
midgard-wallet status
```

Shows if the node is online and the current block height.

### Connect to Custom Node

By default, the wallet connects to `http://localhost:8001`. To use a different node:

```bash
midgard-wallet --node-url http://your-node-ip:8001 balance
```

## Wallet Storage

The wallet is stored in your home directory as `midgard_wallet.json`:
- Windows: `C:\Users\YourName\midgard_wallet.json`
- Linux/Mac: `~/.midgard_wallet.json` or `/home/yourusername/midgard_wallet.json`

## Security Notes

1. **Backup your mnemonic phrase** - Write it down and store it securely offline
2. **Never share your mnemonic or private key** with anyone
3. **The wallet file contains your private key** - keep it secure
4. For production use, consider adding encryption to the wallet file

## Architecture

The wallet consists of four main modules:

- **wallet.rs** - Key generation, address encoding, wallet storage
- **transaction.rs** - Transaction creation and signing
- **client.rs** - API client for blockchain node communication
- **main.rs** - CLI interface

## Transaction Format

Transactions are signed using Ed25519 and include:
- Sender/recipient addresses
- Amount and fee
- Nonce (for replay protection)
- Chain ID (default: "ouroboros-mainnet-1")
- Optional payload for smart contract calls

## Requirements

- OVM blockchain node running at `http://localhost:8001` (or custom URL)
- Rust 1.70+ for building from source

## Compatibility

This wallet is compatible with **Ouroboros Phase 6** blockchain with:
- ✅ Chain ID support ("ouroboros-mainnet-1")
- ✅ Automatic nonce management via `/ouro/nonce/{address}` endpoint
- ✅ Ed25519 signature verification with proper signing message format
- ✅ Replay protection through chain_id + nonce
- ✅ Bech32 address encoding with "ouro" prefix

See `COMPATIBILITY_UPDATE.md` for detailed information about recent compatibility updates.

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
