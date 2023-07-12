use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use rgb_lib::wallet::{AssetRgb20, BlindData, Online, Recipient, Wallet};
use rgb_lib::TransferStatus;

use crate::constants::{FEE_RATE, TRANSPORT_ENDPOINT};
use crate::regtest;

/// Wrapper for rgb-lib wallet
pub(crate) struct WalletWrapper {
    wallet: RefCell<Wallet>,
    online: Online,
    fingerprint: String,
    wallet_index: u8,
    asset_counter: u8,
}

impl Debug for WalletWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wallet_data = &self.wallet.borrow().get_wallet_data();
        f.debug_struct("WalletInfo")
            .field("data_dir", &wallet_data.data_dir)
            .field("network", &wallet_data.bitcoin_network)
            .field("pubkey", &wallet_data.pubkey)
            .field(
                "mnemnonic",
                &wallet_data.mnemonic.clone().unwrap_or("".to_string()),
            )
            .field("fingerprint", &self.fingerprint)
            .field("wallet_index", &self.wallet_index)
            .field("asset_counter", &self.asset_counter)
            .finish()
    }
}

impl WalletWrapper {
    pub(crate) fn new(
        wallet: Wallet,
        online: Online,
        fingerprint: String,
        wallet_index: u8,
    ) -> Self {
        WalletWrapper {
            wallet: RefCell::new(wallet),
            online,
            fingerprint,
            wallet_index,
            asset_counter: 0,
        }
    }

    pub(crate) fn send(
        &self,
        amount: u64,
        recver: &WalletWrapper,
        asset_ids: &Vec<&str>,
    ) -> (String, HashMap<String, String>) {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut recipient_map = HashMap::new();
        for asset_id in asset_ids {
            let blind_data = recver.blind();
            map.insert(asset_id.to_string(), blind_data.blinded_utxo.clone());
            recipient_map.insert(
                asset_id.to_string(),
                vec![Recipient {
                    amount,
                    blinded_utxo: blind_data.blinded_utxo.to_string(),
                    transport_endpoints: vec![TRANSPORT_ENDPOINT.to_string()],
                }],
            );
        }
        let txid = self
            .wallet
            .borrow_mut()
            .send(self.online.clone(), recipient_map, false, FEE_RATE)
            .unwrap();
        (txid, map)
    }

    fn refresh(&self) -> bool {
        self.wallet
            .borrow_mut()
            .refresh(self.online.clone(), None, vec![])
            .unwrap()
    }

    fn blind(&self) -> BlindData {
        self.wallet
            .borrow_mut()
            .blind(None, None, None, vec![TRANSPORT_ENDPOINT.to_string()])
            .unwrap()
    }

    fn check_transfer(&self, map: HashMap<String, String>) {
        for (asset_id, blinded_utxo) in map {
            let transfers = self
                .wallet
                .borrow_mut()
                .list_transfers(asset_id.to_string())
                .unwrap();
            let transfer = transfers
                .iter()
                .find(|t| t.blinded_utxo == Some(blinded_utxo.to_string()))
                .unwrap();
            assert_eq!(transfer.status, TransferStatus::Settled);
        }
    }

    pub(crate) fn create_utxos(&self, num: u8, size: u32) {
        self.wallet
            .borrow_mut()
            .create_utxos(self.online.clone(), true, Some(num), Some(size), FEE_RATE)
            .unwrap();
    }

    pub(crate) fn fund(&self, amt: u32) {
        let address = self.wallet.borrow().get_address();
        let fund_amount = amt as f32 / 100_000_000f32;
        regtest::fund_wallet(&address, &fund_amount.to_string());
        regtest::mine();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    pub(crate) fn show_unspents_with_allocations(&self) {
        let unspents = self.wallet.borrow_mut().list_unspents(true).unwrap();
        for unspent in unspents {
            let utxo = unspent.utxo;
            if !utxo.colorable {
                continue;
            }
            println!(
                "- outpoint: {}, amount: {} sats",
                utxo.outpoint, utxo.btc_amount
            );
            let allocations = unspent.rgb_allocations;
            for allocation in allocations {
                println!(
                    "    amount: {:4}, asset ID: {}",
                    allocation.amount,
                    allocation.asset_id.unwrap()
                );
            }
        }
    }

    /// Issue asset with unique name
    pub(crate) fn issue_rgb20(&mut self, amounts: Vec<u64>) -> AssetRgb20 {
        self.asset_counter += 1;
        let ticker = format!("T{}{}", self.wallet_index, self.asset_counter);
        self.wallet
            .borrow_mut()
            .issue_asset_rgb20(self.online.clone(), ticker, "name".to_string(), 0, amounts)
            .unwrap()
    }
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

fn get_consignment_size(consignment_path: &str) -> u64 {
    let metadata = std::fs::metadata(consignment_path).unwrap();
    metadata.len()
}

pub(crate) fn send_assets(
    sender: &WalletWrapper,
    recver: &WalletWrapper,
    asset_ids: &Vec<&str>,
    amount: u64,
) -> String {
    let data_dir = &sender.wallet.borrow().get_wallet_data().data_dir;
    let t_begin = timestamp();

    let (txid, map) = sender.send(amount, recver, asset_ids);
    let t_send = timestamp();
    assert!(!txid.is_empty());

    // take transfers from WaitingCounterparty to Settled
    print!(
        "  {}->{} send[{:6}] > refreshing:",
        sender.fingerprint,
        recver.fingerprint,
        (t_send - t_begin).as_millis()
    );
    std::io::stdout().flush().unwrap();
    print!(" receiver");
    std::io::stdout().flush().unwrap();
    recver.refresh();
    let t_ref_recv_1 = timestamp();
    print!("[{:6}]", (t_ref_recv_1 - t_send).as_millis());
    print!(", sender");
    std::io::stdout().flush().unwrap();
    sender.refresh();
    let t_ref_send_1 = timestamp();
    print!("[{:6}]", (t_ref_send_1 - t_ref_recv_1).as_millis());
    print!(", mining");
    std::io::stdout().flush().unwrap();
    regtest::mine();
    let t_mine = timestamp();
    print!(", receiver");
    std::io::stdout().flush().unwrap();
    recver.refresh();
    let t_ref_recv_2 = timestamp();
    print!("[{:6}]", (t_ref_recv_2 - t_mine).as_millis());
    print!(", sender");
    std::io::stdout().flush().unwrap();
    sender.refresh();
    let t_end = timestamp();
    print!("[{:6}]", (t_end - t_ref_recv_2).as_millis());
    print!(" > {:6} total", (t_end - t_begin).as_millis());
    std::io::stdout().flush().unwrap();

    let mut consignment_sizes = Vec::with_capacity(asset_ids.clone().len());
    for asset_id in asset_ids {
        let consignment_path = get_consignment_path(data_dir, &sender.fingerprint, &txid, asset_id);
        let consignment_size = get_consignment_size(&consignment_path);
        consignment_sizes.push(consignment_size);
    }
    let consignment_str = consignment_sizes
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    println!(" > consignment sizes: {}", consignment_str);

    // check transfers have settled
    sender.check_transfer(map.clone());
    recver.check_transfer(map);

    format!(
        "\"{}\",\"{}\",{},{},{},{},{},{},{}\n",
        sender.fingerprint,
        recver.fingerprint,
        (t_send - t_begin).as_millis(),
        (t_ref_recv_1 - t_send).as_millis(),
        (t_ref_send_1 - t_ref_recv_1).as_millis(),
        (t_ref_recv_2 - t_mine).as_millis(),
        (t_end - t_ref_recv_2).as_millis(),
        (t_end - t_begin).as_millis(),
        consignment_str,
    )
}

fn timestamp() -> Instant {
    Instant::now()
}
