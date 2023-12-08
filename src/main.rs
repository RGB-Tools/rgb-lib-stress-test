mod constants;
mod opts;
mod regtest;
mod rgb;
mod scenarios;

use std::fs;

use clap::Parser;
use scenarios::{merge_histories, merge_utxos, random_transfers, random_wallets};

use crate::opts::Opts;
use crate::scenarios::send_loop;

fn main() -> Result<(), String> {
    // setup
    let opts = Opts::parse();
    if !matches!(opts.command, crate::opts::Command::RandomTransfers { .. })
        && opts.allocation_utxos == 1
    {
        return Err(
            "invalid value '1' for '--allocation_utxos <ALLOCATION_UTXOS>': valid range 2..255"
                .to_string(),
        );
    }
    if !opts.force && opts.output.exists() {
        return Err(
            "Report file already exists, abrting. (run with --force to override)".to_string(),
        );
    }
    regtest::start_services();
    let data_dir = opts.data_dir.to_str().unwrap();
    fs::create_dir_all(data_dir).unwrap();

    // command processing
    match opts.command {
        opts::Command::SendLoop { loops } => send_loop(opts, loops),
        opts::Command::MergeHistories { loops } => merge_histories(opts, loops),
        opts::Command::MergeUtxos { assets, loops } => merge_utxos(opts, assets, loops),
        opts::Command::RandomWallets { loops, wallets } => random_wallets(opts, loops, wallets),
        opts::Command::RandomTransfers {
            wallets,
            assets,
            max_allocations_per_utxo,
            loops,
        } => random_transfers(opts, wallets, assets, max_allocations_per_utxo, loops),
    };

    // teardown
    regtest::stop_services();
    Ok(())
}
