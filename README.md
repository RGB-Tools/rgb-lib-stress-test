# RGB transfer stress test using rgb-lib

This a script to test many subsequent transfers of the same RGB20 asset between
two wallets. The consignment size and the time it takes to carry out the
required operations is reported for each transfer.

The final output is saved in the `report.csv` file. Consignment file size is in
bytes and times are in seconds.


## Description

The script will start local copies of the required services (bitcoind, electrs,
proxy) in docker, setup two rgb-lib wallets, issue an asset using the first
wallet, then send a small amount from wallet 1 to wallet 2 and back multiple
times and finally stop the services.

## Usage

To run the test execute:
```sh
cargo run
```
