use std::process::{Command, Stdio};

pub(crate) fn start_services() {
    println!("start services");
    let status = Command::new("./services.sh")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("start")
        .status()
        .expect("failed to start services");
    assert!(status.success());
}

pub(crate) fn stop_services() {
    println!("\nstop services");
    let status = Command::new("./services.sh")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("stop")
        .status()
        .expect("failed to stop services");
    assert!(status.success());
}

pub(crate) fn fund_wallet(address: &str, amount: &str) {
    let status = Command::new("docker")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("compose")
        .args(_bitcoin_cli())
        .arg("-rpcwallet=miner")
        .arg("sendtoaddress")
        .arg(address)
        .arg(amount)
        .status()
        .expect("failed to fund wallet");
    assert!(status.success());
}

pub(crate) fn mine() {
    let status = Command::new("docker")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("compose")
        .args(_bitcoin_cli())
        .arg("-rpcwallet=miner")
        .arg("-generate")
        .arg("1")
        .status()
        .expect("failed to mine");
    assert!(status.success());
}

fn _bitcoin_cli() -> [String; 7] {
    [
        "exec".to_string(),
        "-T".to_string(),
        "-u".to_string(),
        "blits".to_string(),
        "bitcoind".to_string(),
        "bitcoin-cli".to_string(),
        "-regtest".to_string(),
    ]
}
