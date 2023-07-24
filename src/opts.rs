use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[clap(name = "rgb_lib_stress_test", bin_name = "rgb_lib_stress_test")]
pub struct Opts {
    /// Directory where to store wallet data
    #[clap(short, long, default_value = "data")]
    pub data_dir: PathBuf,

    /// Number of wallet allocation UTXOs to be created
    #[clap(short, long, default_value = None)]
    #[arg(value_parser = clap::value_parser!(u8).range(1..))]
    pub allocation_utxos: Option<u8>,

    /// Size, in satoshis, of wallet allocation UTXOs
    #[clap(short, long, default_value = None)]
    #[arg(value_parser = clap::value_parser!(u32).range(1000..))]
    pub utxo_size: Option<u32>,

    /// Asset send amount
    #[clap(short, long, default_value_t = 10)]
    #[arg(value_parser = clap::value_parser!(u64).range(1..))]
    pub send_amount: u64,

    /// CSV report file path
    #[clap(short, long, default_value = "report.csv")]
    pub output: PathBuf,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Eq, PartialEq, Debug)]
pub enum Command {
    /// Send assets back and forth between 2 wallets `loops` times
    SendLoop {
        /// Number of loops
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..))]
        loops: u8,
    },
    /// Issue 1 asset (2 allocations), send to 2 wallets, send and get back `loops` times, send
    /// back to original wallet, spend the resulting allocations (merging histories) and finally
    /// spend the merged histories allocation
    MergeHistories {
        /// Number of loops
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..))]
        loops: u8,
    },

    /// Issue `assets` assets in different wallets, send and get back `loops` times, merge all of
    /// them into one wallet (single UTXO) and spend the resulting allocations
    MergeUtxos {
        /// Number of assets
        #[clap(short, long, default_value_t = 5)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        assets: u8,

        /// Number of loops
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..))]
        loops: u8,
    },

    /// Create `wallets` wallets, issue an asset, then send it to a randomly-selected wallet
    /// `loops` times
    RandomWallets {
        /// Number of loops
        #[clap(short, long, default_value_t = 16)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..))]
        loops: u8,

        /// Number of wallets
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(2..))]
        wallets: u8,
    },
}
