mod regtest;
mod rgb;
mod search;

use std::fs;
use std::io::Write;

use rgb_lib::wallet::{DatabaseType, Wallet, WalletData};
use rgb_lib::{generate_keys, BitcoinNetwork};

use crate::rgb::issue_rgb20;

fn main() {
    // environment setup
    regtest::start_services();

    let loops = 4;
    let allocation_utxos = 2;

    let data_dir = "./data";
    let electrum_url = "tcp://localhost:50001";
    let proxy_url = "http://localhost:3000";
    fs::create_dir_all(data_dir).unwrap();

    // RGB wallet 1 setup
    print!("setting up wallet 1");
    let keys_1 = generate_keys(BitcoinNetwork::Regtest);
    let fingerprint_1 = keys_1.xpub_fingerprint;
    println!(", fingerprint: {fingerprint_1}, log: {data_dir}/{fingerprint_1}/log");
    let wallet_data_1 = WalletData {
        data_dir: data_dir.to_string(),
        bitcoin_network: BitcoinNetwork::Regtest,
        database_type: DatabaseType::Sqlite,
        pubkey: keys_1.xpub,
        mnemonic: Some(keys_1.mnemonic),
    };
    let mut wallet_1 = Wallet::new(wallet_data_1).unwrap();
    let online_1 = wallet_1
        .go_online(true, electrum_url.to_string(), proxy_url.to_string())
        .unwrap();
    let address = wallet_1.get_address();
    regtest::fund_wallet(&address, "0.001");
    regtest::mine();
    wallet_1
        .create_utxos(online_1.clone(), true, Some(allocation_utxos), Some(1000))
        .unwrap();
    // RGB wallet 2 setup
    print!("setting up wallet 2");
    let keys_2 = generate_keys(BitcoinNetwork::Regtest);
    let fingerprint_2 = keys_2.xpub_fingerprint;
    println!(", fingerprint: {fingerprint_2}, log: {data_dir}/{fingerprint_2}/log");
    let wallet_data_2 = WalletData {
        data_dir: data_dir.to_string(),
        bitcoin_network: BitcoinNetwork::Regtest,
        database_type: DatabaseType::Sqlite,
        pubkey: keys_2.xpub,
        mnemonic: Some(keys_2.mnemonic),
    };
    let mut wallet_2 = Wallet::new(wallet_data_2).unwrap();
    let online_2 = wallet_2
        .go_online(true, electrum_url.to_string(), proxy_url.to_string())
        .unwrap();
    let address = wallet_2.get_address();
    regtest::fund_wallet(&address, "0.001");
    regtest::mine();
    wallet_2
        .create_utxos(online_2.clone(), true, Some(allocation_utxos), Some(1000))
        .unwrap();

    // RGB asset issuance
    println!("issuing asset");
    let asset = issue_rgb20(&mut wallet_1, &online_1);

    // RGB asset send loop
    let amount = 10;
    let mut report_file = fs::File::create("report.csv").expect("file should have been created");
    let report_header = concat!(
        "consignment size",
        ",send,recv refresh 1,send refresh 1,recv refresh 2,send refresh 2",
        ",recv validate,recv register,recv consume,send consume",
        ",total time",
        ",sender wallet\n",
    );
    write_report_line(&mut report_file, report_header);
    for i in 0..loops {
        println!("\n[{i}] sending assets 1 -> 2");
        let result = rgb::send_assets(
            (&mut wallet_1, &online_1, &fingerprint_1),
            (&mut wallet_2, &online_2, &fingerprint_2),
            &asset.asset_id,
            amount,
            data_dir,
        );
        write_report_line(&mut report_file, &result);
        println!("\n[{i}] sending assets 2 -> 1");
        let result = rgb::send_assets(
            (&mut wallet_2, &online_2, &fingerprint_2),
            (&mut wallet_1, &online_1, &fingerprint_1),
            &asset.asset_id,
            amount,
            data_dir,
        );
        write_report_line(&mut report_file, &result);
    }

    // services teardown
    regtest::stop_services();
}

fn write_report_line(report_file: &mut fs::File, line: &str) {
    report_file
        .write_all(line.as_bytes())
        .expect("line should have been written");
}
