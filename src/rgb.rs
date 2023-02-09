use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use time::OffsetDateTime;

use rgb_lib::wallet::{AssetRgb20, Online, Recipient, Wallet};
use rgb_lib::TransferStatus;

use crate::regtest;
use crate::search;

const CONSUME_PATTERNS: (&str, &str) = ("Consuming RGB transfer", "Consumed RGB transfer");
const REGISTER_PATTERNS: (&str, &str) = ("Registering contract", "Contract registered");
const VALIDATE_PATTERNS: (&str, &str) = ("Validating consignment", "Consignment validity");

enum Op {
    Consume,
    Register,
    Validate,
}

fn get_consignment_path(data_dir: &str, fingerprint: &str, txid: &str, asset_id: &str) -> String {
    let mut consignment_path = PathBuf::new();
    consignment_path.push(data_dir);
    consignment_path.push(fingerprint);
    consignment_path.push("transfers");
    consignment_path.push(txid);
    consignment_path.push(asset_id);
    consignment_path.push("consignment_out");
    consignment_path.to_string_lossy().to_string()
}

fn get_log_path(data_dir: &str, fingerprint: &str) -> String {
    let mut consignment_path = PathBuf::new();
    consignment_path.push(data_dir);
    consignment_path.push(fingerprint);
    consignment_path.push("log");
    consignment_path.to_string_lossy().to_string()
}

fn get_consignment_size(consignment_path: &str) -> u64 {
    let metadata = std::fs::metadata(consignment_path).unwrap();
    metadata.len()
}

pub(crate) fn issue_rgb20(wallet: &mut Wallet, online: &Online) -> AssetRgb20 {
    wallet
        .issue_asset_rgb20(
            online.clone(),
            "TICKER".to_string(),
            "name".to_string(),
            0,
            vec![1000],
        )
        .unwrap()
}

pub(crate) fn send_assets(
    sender: (&mut Wallet, &Online, &str),
    recver: (&mut Wallet, &Online, &str),
    asset_id: &str,
    amount: u64,
    data_dir: &str,
    fee_rate: f32,
) -> String {
    let consignment_endpoints = vec!["rgbhttpjsonrpc:http://localhost:3000/json-rpc".to_string()];
    let t_begin = timestamp();
    let blind_data = recver.0.blind(None, None, None, consignment_endpoints.clone()).unwrap();
    let recipient_map = HashMap::from([(
        asset_id.to_string(),
        vec![Recipient {
            amount,
            blinded_utxo: blind_data.blinded_utxo.clone(),
            consignment_endpoints,
        }],
    )]);
    let txid = sender
        .0
        .send(sender.1.clone(), recipient_map, false, fee_rate)
        .unwrap();
    let t_send = timestamp();
    assert!(!txid.is_empty());

    // take transfers from WaitingCounterparty to Settled
    print!("  send[{}s] refreshing wallets:", t_send - t_begin);
    std::io::stdout().flush().unwrap();
    print!(" recv");
    std::io::stdout().flush().unwrap();
    recver.0.refresh(recver.1.clone(), None, vec![]).unwrap();
    let t_ref_recv_1 = timestamp();
    print!("[{}]", t_ref_recv_1 - t_send);
    print!(" send");
    std::io::stdout().flush().unwrap();
    sender.0.refresh(sender.1.clone(), None, vec![]).unwrap();
    let t_ref_send_1 = timestamp();
    print!("[{}]", t_ref_send_1 - t_ref_recv_1);
    print!(" (mining)");
    std::io::stdout().flush().unwrap();
    regtest::mine();
    let t_mine = timestamp();
    print!(" recv");
    std::io::stdout().flush().unwrap();
    recver.0.refresh(recver.1.clone(), None, vec![]).unwrap();
    let t_ref_recv_2 = timestamp();
    print!("[{}]", t_ref_recv_2 - t_mine);
    print!(" send");
    std::io::stdout().flush().unwrap();
    sender.0.refresh(sender.1.clone(), None, vec![]).unwrap();
    let t_end = timestamp();
    print!("[{}]", t_end - t_ref_recv_2);

    let recv_log_path = get_log_path(data_dir, recver.2);
    let (recv_val_time, recv_val_line_b, recv_val_line_e) = time_ops(&recv_log_path, Op::Validate);
    let (recv_reg_time, recv_reg_line_b, recv_reg_line_e) = time_ops(&recv_log_path, Op::Register);
    let (recv_con_time, recv_con_line_b, recv_con_line_e) = time_ops(&recv_log_path, Op::Consume);
    let send_log_path = get_log_path(data_dir, sender.2);
    let (send_con_time, send_con_line_b, send_con_line_e) = time_ops(&send_log_path, Op::Consume);
    print!(" -=recv validate[{recv_val_time}] recv register[{recv_reg_time}]");
    print!(" recv consume[{recv_con_time}] send consume[{send_con_time}]=-");
    print!(" ...{}s total", t_end - t_begin);

    // check transfers
    let transfers_1 = sender.0.list_transfers(asset_id.to_string()).unwrap();
    let transfer_1 = transfers_1
        .iter()
        .find(|t| t.blinded_utxo == Some(blind_data.blinded_utxo.clone()))
        .unwrap();
    assert_eq!(transfer_1.status, TransferStatus::Settled);
    let transfers_2 = recver.0.list_transfers(asset_id.to_string()).unwrap();
    let transfer_2 = transfers_2
        .iter()
        .find(|t| t.blinded_utxo == Some(blind_data.blinded_utxo.clone()))
        .unwrap();
    assert_eq!(transfer_2.status, TransferStatus::Settled);
    let consignment_path = get_consignment_path(data_dir, sender.2, &txid, asset_id);
    let consignment_size = get_consignment_size(&consignment_path);
    println!(" > consignment file size: {consignment_size}");
    println!(
        "  - {}:{}-{} (receiver validate)",
        recver.2, recv_val_line_b, recv_val_line_e
    );
    println!(
        "  - {}:{}-{} (receiver register)",
        recver.2, recv_reg_line_b, recv_reg_line_e
    );
    println!(
        "  - {}:{}-{} (receiver consume)",
        recver.2, recv_con_line_b, recv_con_line_e
    );
    println!(
        "  - {}:{}-{} (sender consume)",
        sender.2, send_con_line_b, send_con_line_e
    );
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{}\n",
        consignment_size,
        t_send - t_begin,
        t_ref_recv_1 - t_send,
        t_ref_send_1 - t_ref_recv_1,
        t_ref_recv_2 - t_mine,
        t_end - t_ref_recv_2,
        recv_val_time,
        recv_reg_time,
        recv_con_time,
        send_con_time,
        t_end - t_begin,
        sender.2,
    )
}

fn time_ops(log_path: &str, op: Op) -> (i64, u64, u64) {
    let begin_pat: &str;
    let end_pat: &str;
    match op {
        Op::Consume => (begin_pat, end_pat) = CONSUME_PATTERNS,
        Op::Register => (begin_pat, end_pat) = REGISTER_PATTERNS,
        Op::Validate => (begin_pat, end_pat) = VALIDATE_PATTERNS,
    }
    let begins = search::grep_log(log_path, begin_pat);
    let ends = search::grep_log(log_path, end_pat);

    let begin = begins.last().unwrap();
    let end = ends.last().unwrap();
    let op_time = search::get_time_diff((begin.clone(), end.clone()));
    (op_time, begin.line, end.line)
}

fn timestamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}
