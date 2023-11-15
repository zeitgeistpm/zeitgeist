// run this script by `./scripts/tests/zombienet.sh --test`

import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { AccountInfo } from '@polkadot/types/interfaces';

async function main() {
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider });

    // Wait for the crypto library to be ready
    await cryptoWaitReady();

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');

    const aliceBalance = await api.query.system.account(alice.address);

    // Cast the Codec type to AccountInfo type
    const aliceAccountInfo = aliceBalance as AccountInfo;

    // Create a transfer transaction from Alice to Bob
    const transfer = api.tx.balances.transfer(bob.address, aliceAccountInfo.data.free);

    const hash = await transfer.signAndSend(alice);

    console.log(`Transfer sent with hash ${hash}`);

    const bobBalance = await api.query.system.account(bob.address);
    const bobAccountInfo = bobBalance as AccountInfo;

    // Check if the balance transfer was successful
    if (bobAccountInfo.data.free.gt(aliceAccountInfo.data.free)) {
        console.log(`Transfer was successful. Transaction hash: ${hash}`);
        return 1;
    } else {
        console.log(`Transfer failed.`);
        return 0;
    }
}

main().catch(console.error);
