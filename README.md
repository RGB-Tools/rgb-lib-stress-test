# RGB transfer stress test using rgb-lib

This is a CLI command to test a few scenarios involving transfers of RGB20
assets between multiple wallets.

The time it takes to carry out the required operations and the size of the
resulting consignment files is reported for each transfer.

Times are in milliseconds and consignment sizes are in bytes.

Multiple test scenarios are available and some parameters can be tweaked via
command-line options. See the next sections for details.

## Description

Upon command invocation, local copies of the required services (bitcoind,
electrs, proxy) are started in docker, then each scenario sets up and executes
its specific operations, optionally tuned via the given command-line options,
and finally the services are stopped.

A brief description of each scenario follows.

### Send loop

This scenario uses two wallets, issues an asset with the first one, then sends
assets back and forth between the two wallets in loops.

The number of loops can be tweaked via command-line option.

This extends the transition history, allowing to see the impact on transfer
times and consignment sizes.

### Merge histories

This scenario uses six wallets. The first wallet is used to issue an asset to
two allocations. Those allocations are sent to two other wallets. Each of these
two wallets then sends assets back and forth between itself and another
(initially empty) wallet in loops. Once the loops are completed the resulting
allocations are sent from the two wallets holding them back to the first
wallet, which in turn sends the sum of the two allocations to another (empty)
wallet, thus merging the two transition histories, up to the two issuance
allocations. Finally, the resulting allocation (with merged histories) is sent
back to the first wallet.

The number of loops can be tweaked via command-line option.

This extends and then merges two transition histories, allowing to see the
impact on transfer times and consignment sizes.

### Merge UTXOs

This scenario uses one wallet per asset, with one asset issued per wallet, plus
a common (initially empty) receiver wallet and a merger wallet with just one
available UTXO. Assets are sent between each wallet and the common receiver in
loops. The resulting allocations are then all sent to the merger wallet, which
aggregates them all to a single UTXO, and finally sent to the common receiver
wallet.

The number of assets and loops can be tweaked via command-line options.

### Random wallets

This scenario uses four wallets by default. An asset is issued to the first
wallet, then it is sent to a randomly-chosen wallet each time.

The number of loops and wallets can be tweaked via command-line options.

## Usage

Build the CLI with:
```sh
cargo build
```

Get the CLI help message with:
```sh
cargo run -q -- -h
```

To run a test scenario with default options, add the test name. As an example:
```sh
cargo run -q -- send-loop
```

The test will print info messages about the steps as they are carried out.
Each transfer will print the sender -> receiver wallet fingerprints, followed
by the operation times (as they progress), the total time taken by the whole
transfer and the consignment size(s). If the `--verbose` command-line options
is set, some scenarios also show the state of relevant wallet allocations.

The steps for a transfer are:
- send: creation of the transfer and sending of the consignment
- refresh 1 (receiver): getting the consignment, validating and ACKing it
- refresh 2 (sender): getting the consignment ACK and broadcasting the transaction
- mining of a block
- refresh 3 (receiver): settling the transfer once it has been confirmed
- refresh 4 (sender): settling the transfer once it has been confirmed

Refer to the help message of each scenario for the list of supported options.
As an example:
```sh
cargo run -q -- send-loop -h
```

Notes:
- scenarios should work with default values, option tweaking is meant to
  explore variants but this is not guaranteed to work in all cases and
  execution may run into issues
- a startup check prevents to overwrite the generated report file by accident,
  name the file explicitly with the `--output` option or override the check
  with the `--force` option
- the wallet data directory is never cleaned up automatically
- if the command execution crashes, services are not stopped (you can stop them
  manually with `docker compose down`)

## Known issues

The release build is currently not working. See [this
issue](https://github.com/RGB-Tools/rgb-lib/issues/27)

## Report

Each test run produces a report file in CSV format, containing one line for
each transfer that has been carried out.

The default file name is `report.csv` but a custom path can be specified via
command-line option.

The generated file contains the following columns:
- fingerprint of the wallet acting as sender in the transfer
- fingerprint of the wallet acting as receiver in the transfer
- rgb-lib send time
- rgb-lib 1st refresh time
- rgb-lib 2nd refresh time
- rgb-lib 3rd refresh time
- rgb-lib 4th refresh time
- total time to complete the whole transfer
- consignment file size(s)
