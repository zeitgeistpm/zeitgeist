# Integration tests

## Description

Moonwall - Test framework for testing chain networks.

## Installation

### NPM Installation

> Package manager `pnpm` is required for the integration tests. You can install it with `npm install -g pnpm` or otherwise following [their instructions](https://pnpm.io/installation).

```
pnpm -g i @moonwall/cli
```

From here you can import the items you need from moonwall packages in your code:
```
import { describeSuite , beforeAll, expect, ALITH_ADDRESS } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
```

## Functions

- Init: Generates a new config file.
- Run: Runs a network.
- Test: Executes tests, and runs a network if neccesary.
- Download: Gets node binaries for polkadot, moonbeam from GH.

> :information_source: Use `--help` for more information about arguments for each command

### Usage Examples (non-exhaustive)

You should have installed `python` for using `sqlite3` and then used `pnpm rebuild && pnpm rebuild sqlite3`.

Run `pnpm install` in the `integration-tests` folder before running any of the following commands.

Useful for integration testing:

- `./integration-tests/scripts/deploy-zombienet.sh` - Deploy a local relay-parachain network for zombienet.
- `pnpm chopsticks xcm -r polkadot -p ./configs/hydradx.yml -p ./configs/zeitgeist.yml` - Deploy a local relay-parachain fork network via chopsticks to test XCM.
- `./integration-tests/scripts/deploy-zombienet.sh --test` - Run ZNDSL zombienet tests on a local relay-parachain network.
- `pnpm exec moonwall test zombienet_zeitgeist_upgrade` - Test Zeitgeist runtime upgrade on zombienet for the local network.
- `pnpm exec moonwall test chopsticks_zeitgeist_upgrade` - Test Zeitgeist runtime upgrade on chopsticks for the live network.
- `pnpm exec moonwall test chopsticks_battery_station_upgrade` - Test Battery Station runtime upgrade on chopsticks for the test network.

Possible moonwall commands:

- `moonwall` : If you have globally installed moonwall, here is the most minimal entrypoint

- `pnpm moonwall` : This can be used if locally installed, and will launch the main menu.. However, there were many bugs experienced when using this cli.

- `pnpx @moonwall/cli run <ENV_NAME>` : To download and run the latest moonwall binary from npm.js repository, and run a network specified in your config file.

- `pnpm exec moonwall test <ENV_NAME>` : To run the locally compiled version of the binary, to start network and run tests against it.

- `pnpm moonwall download <ARTIFACT NAME> <VERSION> <PATH>` : To run the locally compiled version of the binary, to download an artifact directly from github.


The combinations are endless, for more information you can see the pnpm docs [here](https://pnpm.io/cli/run).