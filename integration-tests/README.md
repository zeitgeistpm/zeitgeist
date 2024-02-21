# Integration tests

## Description

Integation testing for ZeitgeistPM using
[Moonwall](https://github.com/Moonsong-Labs/moonwall).

Consider the documentation of the
[Moonwall repository](https://github.com/Moonsong-Labs/moonwall) for more
information.

## Installation

### NPM Installation

> Package manager `pnpm` is required for the integration tests. You can install
> it with `npm install -g pnpm` or otherwise following
> [their instructions](https://pnpm.io/installation).

#### Go to the `integration-tests` directory:

```bash
cd integration-tests
```

The following commands all assume you are in the `integration-tests` directory.

Run `pnpm install` in the `integration-tests` folder before running any of the
following commands.

```
pnpm install
```

You should have installed `python` for using `sqlite3` and then used
`pnpm rebuild && pnpm rebuild sqlite3`.

### Running the test environments

#### Deploy a local, running relay-parachain network for zombienet:

This is useful for testing the parachain client. It starts producing blocks of
the relay and parachain from genesis.

```bash
./scripts/download-polkadot.sh
./scripts/deploy-zombienet.sh
```

It is expected to see the following output multiple times before the network is launched:

```text
Error fetching metrics from: http://127.0.0.1:<port>/metrics
```

##### Run ZNDSL zombienet tests on a local relay-parachain network:

Using the additional `--test` flag, you can run the ZNDSL tests on the network.

```bash
./scripts/download-polkadot.sh
./scripts/deploy-zombienet.sh --test
```

#### Deploy a local, running relay-parachain fork network via chopsticks (e. g. to test XCM):

This is useful for testing XCM or any other runtime interaction that needs to be
tested on the state of the production network.

```bash
pnpm chopsticks xcm -r polkadot -p ./configs/hydradx.yml -p ./configs/zeitgeist.yml
```

The expected output looks like this:

```text
Unable to map [u8; 32] to a lookup index
[16:36:13.005] INFO (xcm/24440): HydraDX RPC listening on port 8000
Unable to map [u8; 32] to a lookup index
[16:36:14.895] INFO (xcm/24440): Zeitgeist RPC listening on port 8001
[16:36:14.964] INFO (xcm/24440): Connected parachains [2034,2092]
[16:36:14.964] INFO (24440): Loading config file https://raw.githubusercontent.com/AcalaNetwork/chopsticks/master/configs/polkadot.yml
Unable to map [u8; 32] to a lookup index
[16:36:16.944] INFO (xcm/24440): Polkadot RPC listening on port 8002
[16:36:17.112] INFO (xcm/24440): Connected relaychain 'Polkadot' with parachain 'HydraDX'
[16:36:17.240] INFO (xcm/24440): Connected relaychain 'Polkadot' with parachain 'Zeitgeist'
```

#### Test the upgrade to the WASM from `./target/release/wbuild/zeitgeist-runtime` on zombienet:

```bash
pnpm exec moonwall test zombienet_zeitgeist_upgrade
```

#### Test the upgrade to the WASM from `./target/release/wbuild/zeitgeist-runtime` on the live main-net fork using chopsticks:

```bash
pnpm exec moonwall test chopsticks_zeitgeist_upgrade
```

#### Test the upgrade to the WASM from `./target/release/wbuild/battery-station-runtime` on the live test-net fork using chopsticks:

```bash
pnpm exec moonwall test chopsticks_battery_station_upgrade
```
