use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::constants::{DEFAULT_MAX_ALLOCATIONS_PER_UTXO, MIN_TX_SATS, WITNESS_SATS};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[clap(name = "rgb_lib_stress_test", bin_name = "rgb_lib_stress_test")]
pub struct Opts {
    /// Override output file existence check
    #[clap(short, long, action)]
    pub force: bool,

    /// Directory where to store wallet data
    #[clap(short, long, default_value = "data")]
    pub data_dir: PathBuf,

    /// Number of wallet allocation UTXOs to be created
    #[clap(short, long, default_value_t = 5)]
    #[arg(value_parser = clap::value_parser!(u8).range(1..))]
    pub allocation_utxos: u8,

    /// Size, in satoshis, of wallet allocation UTXOs
    #[clap(short, long, default_value_t = WITNESS_SATS + MIN_TX_SATS)]
    #[arg(value_parser = clap::value_parser!(u32).range(294..))]
    pub utxo_size: u32,

    /// Asset send amount
    #[clap(short, long, default_value_t = 10)]
    #[arg(value_parser = clap::value_parser!(u64).range(1..))]
    pub send_amount: u64,

    /// CSV report file path
    #[clap(short, long, default_value = "report.csv")]
    pub output: PathBuf,

    /// Enable verbose output
    #[clap(short, long, action)]
    pub verbose: bool,

    /// Enable receiving via witness
    #[clap(short, long, action)]
    pub witness: bool,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Eq, PartialEq, Debug)]
pub enum Command {
    /// Send assets back and forth between 2 wallets `loops` times
    SendLoop {
        /// Number of loops (1-65535)
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        loops: u16,
    },
    /// Issue 1 asset (2 allocations), send to 2 wallets, send and get back `loops` times, send
    /// back to original wallet, spend the resulting allocations (merging histories) and finally
    /// spend the merged histories allocation
    MergeHistories {
        /// Number of loops (1-65535)
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        loops: u16,
    },

    /// Issue `assets` assets in different wallets, send and get back `loops` times, merge all of
    /// them into one wallet (single UTXO) and spend the resulting allocations
    MergeUtxos {
        /// Number of assets (1-5)
        #[clap(short, long, default_value_t = 5)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        assets: u8,

        /// Number of loops (1-65535)
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        loops: u16,
    },

    /// Create `wallets` wallets, issue an asset, then send it to a randomly-selected wallet
    /// `loops` times
    RandomWallets {
        /// Number of loops (1-65535)
        #[clap(short, long, default_value_t = 16)]
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        loops: u16,

        /// Number of wallets (2-255)
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(2..))]
        wallets: u8,
    },

    /// Randomly issues and transfers assets between random wallets.
    RandomTransfers {
        /// Number of total assets (1-255)
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(1..))]
        assets: u8,

        /// Maximum number of allocations per UTXO
        #[clap(short, long, default_value_t = DEFAULT_MAX_ALLOCATIONS_PER_UTXO)]
        #[arg(value_parser = clap::value_parser!(u32).range(1..))]
        max_allocations_per_utxo: u32,

        /// Number of total transfers (1-65535)
        #[clap(short, long, default_value_t = 16)]
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        loops: u16,

        /// Number of wallets (2-255)
        #[clap(short, long, default_value_t = 4)]
        #[arg(value_parser = clap::value_parser!(u8).range(2..))]
        wallets: u8,
    },
}
