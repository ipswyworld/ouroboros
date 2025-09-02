// src/bin/ouro-node.rs
use clap::{Parser, Subcommand};
use std::env;
use yansi::Paint;

#[derive(Parser)]
#[command(name = "ouro-node", about = "Ouroboros node CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a bootstrap node (no peers)
    Start {
        /// API port (default: 8000)
        #[arg(long, default_value_t = 8000)]
        api_port: u16,

        /// P2P listen port (default: 9000)
        #[arg(long, default_value_t = 9000)]
        p2p_port: u16,

        /// Use RocksDB path (optional)
        #[arg(long)]
        rocksdb_path: Option<String>,
    },

    /// Join an existing network via a peer address or bootstrap fetch
    Join {
        /// A peer to connect to (comma-separated list accepted)
        #[arg(long)]
        peer: Option<String>,

        /// Fetch a bootstrap list from this URL (optional)
        #[arg(long)]
        bootstrap_url: Option<String>,

        /// API port to expose locally (default: 8000)
        #[arg(long, default_value_t = 8000)]
        api_port: u16,

        /// P2P listen port (default: 9000)
        #[arg(long, default_value_t = 9000)]
        p2p_port: u16,

        /// RocksDB path (optional)
        #[arg(long)]
        rocksdb_path: Option<String>,
    },
}

fn banner() {
    let name = r#"
 _____ _   _______ ___________  ___________ _____ _____ 
|  _  | | | | ___ \  _  | ___ \ |  _  | ___ \_   _/  ___|
| | | | | | | |_/ / | | | | |_/ / | | | | |_/ /  | | \ `--. 
| | | | | | |    /| | | | ___ \ | | | |    /  | |  `--. \
\ \_/ / |_| | |\ \\ \_/ / |_/ / \ \_/ / |\ \ _| |_/\__/ /
 \___/ \___/\_| \_|\___/\____/   \___/\_| \_|\___/\____/
  
"#;
    println!("{}", Paint::cyan(name).bold());
    println!(
        "{} {}",
        Paint::green("Ouroboros Node").bold(),
        Paint::white("— lightweight local testnet node").dimmed()
    );
    println!();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    banner();

    match cli.command {
        Commands::Start {
            api_port,
            p2p_port,
            rocksdb_path,
        } => {
            // Set env variables the rest of the node expects
            env::set_var("API_ADDR", format!("0.0.0.0:{}", api_port));
            env::set_var("LISTEN_ADDR", format!("0.0.0.0:{}", p2p_port));
            if let Some(p) = rocksdb_path {
                env::set_var("ROCKSDB_PATH", p);
            }
            println!(
                "{} API -> http://127.0.0.1:{}   P2P -> 0.0.0.0:{}",
                Paint::blue("[starting]").bold(),
                api_port,
                p2p_port
            );

            // call into your existing runtime (lib.run())
            ouro_dag::run().await?;
        }

        Commands::Join {
            peer,
            bootstrap_url,
            api_port,
            p2p_port,
            rocksdb_path,
        } => {
            // set envs
            env::set_var("API_ADDR", format!("0.0.0.0:{}", api_port));
            env::set_var("LISTEN_ADDR", format!("0.0.0.0:{}", p2p_port));
            if let Some(p) = rocksdb_path {
                env::set_var("ROCKSDB_PATH", p);
            }

            // If a --peer argument is provided, set PEER_ADDRS directly.
            if let Some(peer_str) = peer {
                // accept comma-separated list
                env::set_var("PEER_ADDRS", peer_str.clone());
                println!("{} Connecting to peer(s): {}", Paint::blue("[peers]"), peer_str);
            } else if let Some(url) = bootstrap_url {
                // Simple bootstrap behavior: fetch a newline-separated list of peers
                // Note: using reqwest for this minimal fetch (non-blocking inside tokio)
                println!("{} Fetching bootstrap list from {}", Paint::blue("[bootstrap]"), url);
                match reqwest::get(&url).await {
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            // pick first non-empty line(s)
                            let peers: Vec<&str> =
                                text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                            if !peers.is_empty() {
                                let joined = peers.join(",");
                                env::set_var("PEER_ADDRS", &joined);
                                println!("{} Got peers: {}", Paint::green("[ok]"), joined);
                            } else {
                                println!("{}", Paint::yellow("[warn] Bootstrap returned no peers"));
                            }
                        } else {
                            println!("{}", Paint::red("[err] Failed to read bootstrap response text"));
                        }
                    }
                    Err(e) => {
                        println!("{} bootstrap fetch failed: {}", Paint::red("[err]"), e);
                    }
                }
            } else {
                println!("{}", Paint::yellow("[warn] No peer or bootstrap URL provided — node will start alone"));
            }

            println!(
                "{} API -> http://127.0.0.1:{}   P2P -> 0.0.0.0:{}",
                Paint::blue("[starting]").bold(),
                api_port,
                p2p_port
            );

            ouro_dag::run().await?;
        }
    }

    Ok(())
}
