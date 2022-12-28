# RGB transfer stress test using rgb-lib

This is a script to test many subsequent transfers of the same RGB20 asset
between two wallets. The consignment size and the time it takes to carry out
the required operations is reported for each transfer. Times for some critical
RGB operations are also reported.

The final output is saved to the `report.csv` file. Consignment file size is in
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

The script will print basic info messages of the steps as they're carried out.
Each transfer will print the current loop number and sender -> receiver wallet
info, followed by the operation times (as they progress) and consignment
size, followed by the wallet fingerprint and log line numbers where each of the
timed operations start and end.

## Report format

The generated `report.csv` file contains the following columns:
- consignment file size
- rgb-lib send time (consignment is produced and posted to the proxy)
- rgb-lib 1st refresh (receiver gets and validates consignment, posts ack)
- rgb-lib 2nd refresh (sender gets ack, broadcasts tx)
- rgb-lib 3rd refresh (receiver sees tx confirmed and completes transfer)
- rgb-lib 4th refresh (sender sees tx confirmed and completes transfer)
- rgb-core receiver contract validate (1st refresh)
- rgb-node receiver contract register (1st refresh)
- rgb-node receiver consume transfer (3rd refresh)
- rgb-node sender consume transfer (4th refresh)
- total time to complete the whole transfer
- fingerprint of the wallet acting as sender in the transfer

## Length of operations

The test shows the consignment size and send times growing with each transfer.

To better locate where time is spent, rgb-lib calls to RGB APIs have been
surrounded with logs that provide timestamps of operations start and stop.

The operations taking longer to complete appear to be rgb-core's `validate` and
rgb-node's `consume_transfer` and `register_contract`.

The `validate` operations is carried out during the first receiver's refresh.
It happens between the log lines `Validating consignment` and `Consignment
validity`.

The `register_contract` operations is carried out during the first receiver's
refresh. It happens between the log lines `Registering contract` and `Contract
registered`.

The `consume_transfer` operation happens during the second refresh of both the
receiver and sender. It happens between the log lines `Consuming RGB transfer`
and `Consumed RGB transfer`.
