use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use time::OffsetDateTime;

use rgb_lib::wallet::{AssetRgb20, Online, Recipient, TransferStatus, Wallet};

use crate::regtest;

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
) -> String {
    let t_begin = timestamp();
    let blind_data = recver.0.blind(None, None, None).unwrap();
    let recipient_map = HashMap::from([(
        asset_id.to_string(),
        vec![Recipient {
            amount,
            blinded_utxo: blind_data.blinded_utxo.clone(),
        }],
    )]);
    let txid = sender
        .0
        .send(sender.1.clone(), recipient_map, false)
        .unwrap();
    let t_send = timestamp();
    assert!(!txid.is_empty());

    // take transfers from WaitingCounterparty to Settled
    print!(" - done[{}s] refreshing wallets:", t_send - t_begin);
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
    println!(" ...done in {}s", t_end - t_begin);

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
    println!(" > consignment file size: {}", consignment_size);
    format!(
        "{},{},{},{},{},{},{}\n",
        consignment_size,
        t_end - t_begin,
        t_send - t_begin,
        t_ref_recv_1 - t_send,
        t_ref_send_1 - t_ref_recv_1,
        t_ref_recv_2 - t_mine,
        t_end - t_ref_recv_2
    )
}

fn timestamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}
