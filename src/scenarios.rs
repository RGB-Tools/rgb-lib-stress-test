use rand::prelude::*;
use std::cmp;
use std::fs;
use std::io::Write;

use crate::constants::{DEFAULT_MAX_ALLOCATIONS_PER_UTXO, ELECTRUM_URL, FEE_AMT};
use crate::opts::Opts;
use crate::rgb;
use crate::rgb::{TestMode, WalletWrapper};
use rgb_lib::wallet::{AssetNIA, DatabaseType, Wallet, WalletData};
use rgb_lib::{generate_keys, BitcoinNetwork};

struct ScenarioOpts {
    data_dir: String,
    output: String,
    send_amount: u64,
    utxo_num: u8,
    utxo_size: u32,
    verbose: bool,
}

fn get_scenario_opts(opts: Opts) -> ScenarioOpts {
    ScenarioOpts {
        data_dir: opts.data_dir.to_str().unwrap().to_string(),
        output: opts.output.to_str().unwrap().to_string(),
        send_amount: opts.send_amount,
        utxo_num: opts.allocation_utxos,
        utxo_size: opts.utxo_size,
        verbose: opts.verbose,
    }
}

fn get_wallet(
    data_dir: &str,
    wallet_index: u8,
    utxo_num: u8,
    utxo_size: u32,
    max_allocations_per_utxo: Option<u32>,
) -> WalletWrapper {
    print!("setting up wallet {wallet_index}");
    let keys = generate_keys(BitcoinNetwork::Regtest);
    let fingerprint = keys.xpub_fingerprint;
    println!(", fingerprint: {fingerprint}, log: {data_dir}/{fingerprint}/log");
    let wallet_data = WalletData {
        data_dir: data_dir.to_string(),
        bitcoin_network: BitcoinNetwork::Regtest,
        database_type: DatabaseType::Sqlite,
        pubkey: keys.xpub,
        mnemonic: Some(keys.mnemonic),
        max_allocations_per_utxo: max_allocations_per_utxo
            .unwrap_or(DEFAULT_MAX_ALLOCATIONS_PER_UTXO),
        vanilla_keychain: None,
    };
    let mut wallet = Wallet::new(wallet_data).unwrap();
    let online = wallet.go_online(true, ELECTRUM_URL.to_string()).unwrap();
    let wallet_wrapper = WalletWrapper::new(wallet, online, fingerprint, wallet_index);

    let fund_amount = (utxo_num as u32 * utxo_size) + (utxo_num as u32 * FEE_AMT);
    wallet_wrapper.fund(fund_amount);
    wallet_wrapper.create_utxos(utxo_num, utxo_size, true);

    wallet_wrapper
}

fn write_report_header(report_file: &mut fs::File) {
    let report_header = concat!(
        "sender",
        ",receiver",
        ",send,recv refresh 1,send refresh 1,recv refresh 2,send refresh 2",
        ",total time",
        ",consignment sizes\n",
    );
    write_report_line(report_file, report_header);
}

fn write_report_line(report_file: &mut fs::File, line: &str) {
    report_file
        .write_all(line.as_bytes())
        .expect("line should have been written");
}

pub(crate) fn send_loop(opts: Opts, loops: u16) {
    let ScenarioOpts {
        data_dir,
        output,
        send_amount,
        utxo_num: utxos,
        utxo_size,
        verbose: _,
    } = get_scenario_opts(opts);
    let mut report_file = fs::File::create(output).expect("file should have been created");
    write_report_header(&mut report_file);

    let mut wallet_1 = get_wallet(&data_dir, 1, utxos, utxo_size * loops as u32, None);
    let wallet_2 = get_wallet(&data_dir, 2, utxos, utxo_size * loops as u32, None);

    // RGB asset issuance
    println!("issuing asset");
    let asset = wallet_1.issue_nia(vec![send_amount], &TestMode::NoErrorHandling);

    // RGB asset send loop
    println!("\nsend loops");
    let asset_ids = vec![asset.asset_id.as_str()];
    for i in 1..=loops {
        println!("loop {i}/{loops}");
        let result = rgb::send_assets(
            &wallet_1,
            &wallet_2,
            &asset_ids,
            send_amount,
            &TestMode::NoErrorHandling,
        );
        write_report_line(&mut report_file, &result);
        let result = rgb::send_assets(
            &wallet_2,
            &wallet_1,
            &asset_ids,
            send_amount,
            &TestMode::NoErrorHandling,
        );
        write_report_line(&mut report_file, &result);
    }
}

pub(crate) fn merge_histories(opts: Opts, loops: u16) {
    let ScenarioOpts {
        data_dir,
        output,
        send_amount,
        utxo_num: utxos,
        utxo_size,
        verbose,
    } = get_scenario_opts(opts);
    let mut report_file = fs::File::create(output).expect("file should have been created");
    write_report_header(&mut report_file);

    println!("\nsetup wallets");
    let num_wallets = 6u8;
    let mut wallets = Vec::with_capacity(num_wallets as usize);
    for i in 0..num_wallets {
        let wallet = get_wallet(&data_dir, i, utxos, utxo_size * loops as u32, None);
        wallets.push(wallet);
    }

    // issue asset and split between initial pair of wallets
    println!("\nissue asset (2 allocations)");
    let asset = wallets[0].issue_nia(vec![send_amount, send_amount], &TestMode::NoErrorHandling);
    let asset_id = asset.asset_id;
    println!("asset ID: {}", asset_id.as_str());
    let asset_id_str = asset_id.as_str();
    let asset_ids = vec![asset_id_str];

    println!("\nsend issued assets to 2 empty wallets");
    let result = rgb::send_assets(
        &wallets[0],
        &wallets[1],
        &asset_ids,
        send_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);
    let result = rgb::send_assets(
        &wallets[0],
        &wallets[2],
        &asset_ids,
        send_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);

    // RGB asset send loop to create asset transition histories
    println!("\nsend loops to extend the transition history");
    for i in 1..=loops {
        println!("loop {i}/{loops}");
        for wallet_pair in [(&wallets[1], &wallets[3]), (&wallets[2], &wallets[4])] {
            let result = rgb::send_assets(
                wallet_pair.0,
                wallet_pair.1,
                &asset_ids,
                send_amount,
                &TestMode::NoErrorHandling,
            );
            write_report_line(&mut report_file, &result);
            let result = rgb::send_assets(
                wallet_pair.1,
                wallet_pair.0,
                &asset_ids,
                send_amount,
                &TestMode::NoErrorHandling,
            );
            write_report_line(&mut report_file, &result);
        }
    }

    // send asset back to issuer wallet
    println!("\nsend assets back to issuer wallet");
    let (wallet_last_1, wallet_last_2) = (&wallets[1], &wallets[2]);

    let result = rgb::send_assets(
        wallet_last_1,
        &wallets[0],
        &asset_ids,
        send_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);
    let result = rgb::send_assets(
        wallet_last_2,
        &wallets[0],
        &asset_ids,
        send_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);

    let merge_amount = send_amount * 2;

    // spend from issuer wallet (merged histories)
    println!("\nspend from issuer wallet, merging histories");
    let result = rgb::send_assets(
        &wallets[0],
        &wallets[5],
        &asset_ids,
        merge_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);

    // send back to issuer wallet (spend merged histories)
    println!("\nspend merged histories");
    let result = rgb::send_assets(
        &wallets[5],
        &wallets[0],
        &asset_ids,
        merge_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);

    if verbose {
        println!("\nfinal wallet unspents and related RGB allocations:");
        wallets[0].show_unspents_with_allocations();
    };
}

pub(crate) fn merge_utxos(opts: Opts, num_assets: u8, loops: u16) {
    let ScenarioOpts {
        data_dir,
        output,
        send_amount,
        utxo_num: utxos,
        utxo_size,
        verbose,
    } = get_scenario_opts(opts);
    let mut report_file = fs::File::create(output).expect("file should have been created");
    write_report_header(&mut report_file);

    println!("\nsetup wallets and issue assets");
    let mut issue_wallets = Vec::with_capacity(num_assets as usize);
    let mut asset_ids = Vec::with_capacity(num_assets as usize);
    for i in 0..num_assets {
        let mut wallet = get_wallet(&data_dir, i, utxos, utxo_size * loops as u32, None);
        let asset = wallet.issue_nia(vec![send_amount], &TestMode::NoErrorHandling);

        issue_wallets.push(wallet);
        asset_ids.push(asset.asset_id);
    }

    // create state transition history for wallets
    println!("\nsend loops to extend the transition history");
    let receiver = get_wallet(
        &data_dir,
        num_assets + 1,
        utxos,
        utxo_size * num_assets as u32 * loops as u32, // enough to support all loop transfers
        None,
    );
    for i in 1..=loops {
        println!("loop {i}/{loops}");
        for j in 0..num_assets {
            let sender = &issue_wallets[j as usize];
            let asset_id = vec![asset_ids[j as usize].as_str()];
            let result = rgb::send_assets(
                sender,
                &receiver,
                &asset_id,
                send_amount,
                &TestMode::NoErrorHandling,
            );
            write_report_line(&mut report_file, &result);
            let result = rgb::send_assets(
                &receiver,
                sender,
                &asset_id,
                send_amount,
                &TestMode::NoErrorHandling,
            );
            write_report_line(&mut report_file, &result);
        }
    }

    println!("\nsend all assets to a single wallet (single UTXO)");
    let merger = get_wallet(
        &data_dir,
        num_assets + 2,
        1, // so all allocations will go to the same UTXO
        utxo_size,
        None,
    );
    for i in 0..num_assets {
        let result = rgb::send_assets(
            &issue_wallets[i as usize],
            &merger,
            &vec![asset_ids[i as usize].as_str()],
            send_amount,
            &TestMode::NoErrorHandling,
        );
        write_report_line(&mut report_file, &result);
    }

    println!("\nspend all assets (single UTXO)");
    // create 1 more UTXO as rgb-lib needs one even if there's no change
    merger.fund(utxo_size + FEE_AMT);
    merger.create_utxos(1, utxo_size, true);
    if verbose {
        println!("\nmerger wallet unspents (single UTXO) and related allocations");
        merger.show_unspents_with_allocations();
    };
    let result = rgb::send_assets(
        &merger,
        &receiver,
        &asset_ids.iter().map(|a| a.as_str()).collect(),
        send_amount,
        &TestMode::NoErrorHandling,
    );
    write_report_line(&mut report_file, &result);

    if verbose {
        println!("\nfinal wallet unspents and related RGB allocations:");
        receiver.show_unspents_with_allocations();
    };
}

pub(crate) fn random_wallets(opts: Opts, loops: u16, num_wallets: u8) {
    let ScenarioOpts {
        data_dir,
        output,
        send_amount,
        utxo_num: utxos,
        utxo_size,
        verbose: _,
    } = get_scenario_opts(opts);
    let mut report_file = fs::File::create(output).expect("file should have been created");
    write_report_header(&mut report_file);

    println!("\nsetup wallets");
    let mut wallets = Vec::with_capacity(num_wallets as usize);
    for i in 0..num_wallets {
        let walletinfo = get_wallet(&data_dir, i, utxos, utxo_size * loops as u32, None);
        wallets.push(walletinfo);
    }

    println!("\nissue asset");
    let asset = wallets[0].issue_nia(vec![send_amount], &TestMode::NoErrorHandling);
    let asset_ids = vec![asset.asset_id.as_str()];

    println!("\nsend assets to randomly-selected wallets");
    let mut last_index = 0;
    let len = loops.to_string().len();
    for i in 1..=loops {
        let mut index = rand::thread_rng().gen_range(0..num_wallets as usize);
        while index == last_index {
            index = rand::thread_rng().gen_range(0..num_wallets as usize);
        }
        print!("[{i:len$}/{loops}] ");
        let result = rgb::send_assets(
            &wallets[last_index],
            &wallets[index],
            &asset_ids,
            send_amount,
            &TestMode::NoErrorHandling,
        );
        last_index = index;
        write_report_line(&mut report_file, &result);
    }
}

pub(crate) fn random_transfers(
    opts: Opts,
    num_wallets: u8,
    num_assets: u8,
    max_allocations_per_utxo: u32,
    loops: u16,
) {
    let ScenarioOpts {
        data_dir,
        output,
        send_amount,
        utxo_num: utxos,
        utxo_size,
        verbose: _,
    } = get_scenario_opts(opts);
    let do_handle_errors = &TestMode::HandleUtxoErrors { utxos, utxo_size };
    let mut report_file = fs::File::create(output).expect("file should have been created");
    write_report_header(&mut report_file);

    println!("\nsetup {num_wallets} wallets");
    let mut wallets: Vec<WalletWrapper> = Vec::with_capacity(num_wallets as usize);
    for i in 0..num_wallets {
        let wallet = get_wallet(
            &data_dir,
            i,
            utxos,
            utxo_size,
            Some(max_allocations_per_utxo),
        );
        wallets.push(wallet);
    }

    print!("\nissue {num_assets} asset(s)");
    std::io::stdout().flush().unwrap();
    let mut asset_ids: Vec<AssetNIA> = Vec::new();
    for _i in 0..num_assets {
        let wallet_index = rand::thread_rng().gen_range(0..wallets.len());
        let new_asset = wallets[wallet_index].issue_nia(vec![send_amount], do_handle_errors);
        print!(" {},", new_asset.ticker);
        std::io::stdout().flush().unwrap();
        asset_ids.push(new_asset);
    }

    println!("\ntransfers");
    let len = loops.to_string().len();
    for i in 1..=loops {
        let mut sender: Option<&WalletWrapper> = None;
        let mut asset_id: Option<String> = None;
        let mut asset_balance: Option<u64> = None;
        let mut sender_index = usize::MAX;
        while sender.is_none() {
            sender_index = rand::thread_rng().gen_range(0..wallets.len());
            let wallet = &wallets[sender_index];
            let wallet_assets = wallet.list_assets();
            let wallet_rgb20_assets = wallet_assets.nia.unwrap();
            for rgb20_asset in wallet_rgb20_assets.iter() {
                if rgb20_asset.balance.spendable > 0 {
                    sender = Some(wallet);
                    asset_id = Some(rgb20_asset.asset_id.clone());
                    asset_balance = Some(rgb20_asset.balance.spendable);
                    break;
                }
            }
        }
        if sender.is_none() {
            unreachable!("There should be at least one wallet with assets.")
        }

        let mut receiver = None;
        while receiver.is_none() {
            let receiver_index = rand::thread_rng().gen_range(0..wallets.len());
            if sender_index == receiver_index {
                continue;
            }
            let wallet = &wallets[receiver_index];
            receiver = Some(wallet);
        }

        print!("[{i:len$}/{loops}] ");
        std::io::stdout().flush().unwrap();

        let p = rand::thread_rng().gen_range(1..=10);
        let balance_frac = asset_balance.unwrap() / p;
        let tx_amount = cmp::max(1, balance_frac);

        let result = rgb::send_assets(
            sender.unwrap(),
            receiver.unwrap(),
            &vec![asset_id.as_ref().expect("ID should be present").as_str()],
            tx_amount,
            do_handle_errors,
        );

        write_report_line(&mut report_file, result.as_str());
    }
}
