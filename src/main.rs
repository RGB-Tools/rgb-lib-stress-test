mod constants;
mod opts;
mod regtest;
mod rgb;
mod scenarios;

use std::fs;

use clap::Parser;
use scenarios::{merge_histories, merge_utxos, random_wallets};

use crate::opts::Opts;
use crate::scenarios::send_loop;

fn main() -> Result<(), String> {
    // setup
    let opts = Opts::parse();
    regtest::start_services();
    let data_dir = opts.data_dir.to_str().unwrap();
    fs::create_dir_all(data_dir).unwrap();

    // command processing
    match opts.command {
        opts::Command::SendLoop { loops } => send_loop(opts, loops),
        opts::Command::MergeHistories { loops } => merge_histories(opts, loops),
        opts::Command::MergeUtxos { assets, loops } => merge_utxos(opts, assets, loops),
        opts::Command::RandomWallets { loops, wallets } => random_wallets(opts, loops, wallets),
    };

    // teardown
    regtest::stop_services();
    Ok(())
}
