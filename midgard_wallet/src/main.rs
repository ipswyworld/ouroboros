mod client;
mod transaction;
mod wallet;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::OuroClient;
use colored::Colorize;
use transaction::Transaction;
use wallet::Wallet;

#[derive(Parser)]
#[command(name = "midgard-wallet")]
#[command(about = "Midgard Wallet - CLI wallet for OVM Blockchain", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Node API URL
    #[arg(long, global = true, default_value = "http://localhost:8001")]
    node_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(short, long, default_value = "My Wallet")]
        name: String,
    },

    /// Import wallet from mnemonic or private key
    Import {
        /// Import from mnemonic phrase
        #[arg(short, long, conflicts_with = "private_key")]
        mnemonic: Option<String>,

        /// Import from private key (hex)
        #[arg(short, long, conflicts_with = "mnemonic")]
        private_key: Option<String>,

        /// Wallet name
        #[arg(short, long, default_value = "My Wallet")]
        name: String,
    },

    /// Show wallet information
    Info,

    /// Check wallet balance
    Balance,

    /// Send OURO tokens
    Send {
        /// Recipient address
        to: String,

        /// Amount in smallest units (1 OURO = 1,000,000,000,000 units)
        amount: u64,

        /// Transaction fee (default: 1000)
        #[arg(short, long, default_value_t = 1000)]
        fee: u64,

        /// Transaction nonce (optional, will fetch from blockchain if not provided)
        #[arg(short, long)]
        nonce: Option<u64>,
    },

    /// Show blockchain status
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = OuroClient::new(Some(cli.node_url.clone()));

    match cli.command {
        Commands::Create { name } => {
            if Wallet::exists() {
                println!("{}", "⚠️  Wallet already exists!".yellow());
                println!("Use 'midgard-wallet info' to view your wallet");
                return Ok(());
            }

            println!("{}", "🔐 Creating new wallet...".cyan());
            let (wallet, mnemonic) = Wallet::generate(name)?;
            wallet.save()?;

            println!("\n{}", "✅ Wallet created successfully!".green());
            println!("\n{}", "📝 IMPORTANT: Save your mnemonic phrase securely!".yellow().bold());
            println!("{}", "This is the ONLY way to recover your wallet.".yellow());
            println!("\n{}", mnemonic.bright_white().bold());
            println!("\n{}", format!("Address: {}", wallet.address).cyan());
            println!("{}", format!("Public Key: {}", wallet.public_key).cyan());
        }

        Commands::Import {
            mnemonic,
            private_key,
            name,
        } => {
            if Wallet::exists() {
                println!("{}", "⚠️  Wallet already exists!".yellow());
                println!("Delete the existing wallet first to import a new one");
                return Ok(());
            }

            let wallet = if let Some(mnemonic_phrase) = mnemonic {
                println!("{}", "🔑 Importing wallet from mnemonic...".cyan());
                Wallet::from_mnemonic(&mnemonic_phrase, name)?
            } else if let Some(priv_key) = private_key {
                println!("{}", "🔑 Importing wallet from private key...".cyan());
                Wallet::from_private_key(&priv_key, name)?
            } else {
                println!("{}", "❌ Please provide either --mnemonic or --private-key".red());
                return Ok(());
            };

            wallet.save()?;
            println!("\n{}", "✅ Wallet imported successfully!".green());
            println!("{}", format!("Address: {}", wallet.address).cyan());
            println!("{}", format!("Public Key: {}", wallet.public_key).cyan());
        }

        Commands::Info => {
            let wallet = Wallet::load()?;
            println!("\n{}", "👛 Wallet Information".cyan().bold());
            println!("{}", "═".repeat(50).cyan());
            println!("{}: {}", "Name".bright_white(), wallet.name);
            println!("{}: {}", "Address".bright_white(), wallet.address.green());
            println!("{}: {}", "Public Key".bright_white(), wallet.public_key);
            println!("{}: {}", "Created".bright_white(), wallet.created_at);
        }

        Commands::Balance => {
            let wallet = Wallet::load()?;
            println!("{}", "💰 Fetching balance...".cyan());

            match client.get_balance(&wallet.address) {
                Ok(balance) => {
                    let ouro_balance = balance as f64 / 1_000_000_000_000.0;
                    println!("\n{}", format!("Balance: {} OURO", ouro_balance).green().bold());
                    println!("{}", format!("({} units)", balance).bright_black());
                }
                Err(e) => {
                    println!("{}", format!("❌ Failed to fetch balance: {}", e).red());
                    println!("{}", "Make sure the node is running and accessible".yellow());
                }
            }
        }

        Commands::Send {
            to,
            amount,
            fee,
            nonce,
        } => {
            let wallet = Wallet::load()?;
            println!("{}", "📤 Preparing transaction...".cyan());

            // Fetch nonce from blockchain if not provided
            let tx_nonce = match nonce {
                Some(n) => n,
                None => {
                    println!("{}", "🔍 Fetching nonce from blockchain...".cyan());
                    match client.get_nonce(&wallet.address) {
                        Ok(n) => {
                            println!("{}", format!("Current nonce: {}", n).bright_black());
                            n
                        }
                        Err(e) => {
                            println!("{}", format!("⚠️  Failed to fetch nonce: {}", e).yellow());
                            println!("{}", "Using default nonce: 0".yellow());
                            0
                        }
                    }
                }
            };

            // Create transaction
            let mut tx = Transaction::new(
                wallet.address.clone(),
                to.clone(),
                amount,
                fee,
                tx_nonce,
                wallet.public_key.clone(),
            );

            // Sign transaction
            let signing_key = wallet.get_signing_key()?;
            tx.sign(&signing_key)?;

            println!("\n{}", "Transaction Details:".bright_white().bold());
            println!("{}", "─".repeat(50).bright_black());
            println!("{}: {}", "From".bright_white(), wallet.address.yellow());
            println!("{}: {}", "To".bright_white(), to.green());
            println!(
                "{}: {} OURO",
                "Amount".bright_white(),
                amount as f64 / 1_000_000_000_000.0
            );
            println!("{}: {}", "Fee".bright_white(), fee);
            println!("{}: {}", "Nonce".bright_white(), tx_nonce);
            println!("{}: {}", "Chain ID".bright_white(), "ouroboros-mainnet-1".cyan());
            println!("{}", "─".repeat(50).bright_black());

            // Submit transaction
            println!("\n{}", "📡 Submitting transaction...".cyan());
            match client.submit_transaction(tx.to_api_format()) {
                Ok(tx_id) => {
                    println!("\n{}", "✅ Transaction submitted successfully!".green().bold());
                    println!("{}: {}", "Transaction ID".bright_white(), tx_id.cyan());
                }
                Err(e) => {
                    println!("{}", format!("❌ Transaction failed: {}", e).red());
                }
            }
        }

        Commands::Status => {
            println!("{}", "🔍 Checking node status...".cyan());

            match client.health_check() {
                Ok(true) => {
                    println!("{}", "✅ Node is online".green());

                    if let Ok(height) = client.get_status() {
                        println!("{}: {}", "Block Height".bright_white(), height.to_string().cyan());
                    }
                }
                _ => {
                    println!("{}", "❌ Node is offline or unreachable".red());
                    println!("{}", format!("Trying to connect to: {}", cli.node_url).yellow());
                }
            }
        }
    }

    Ok(())
}
