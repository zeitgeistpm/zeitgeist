import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { AccountInfo } from '@polkadot/types/interfaces';

// Addresses for Alice and Bob on the dev chain
const ALICE = '//Alice';
const BOB = '//Bob';

export const run = async (nodeName: string, networkInfo: any, args: any) => {
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider });

    // Wait for the crypto library to be ready
    await cryptoWaitReady();

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri(ALICE);
    const bob = keyring.addFromUri(BOB);

    const aliceFreeBalanceBefore = await (api.query.system.account(alice.address)) as unknown as AccountInfo;
    const bobFreeBalanceBefore = await (api.query.system.account(bob.address)) as unknown as AccountInfo;

    console.log(`Alice has ${aliceFreeBalanceBefore.data.free}`);
    console.log(`Bob has ${bobFreeBalanceBefore.data.free}`);

    // Create a transfer transaction from Alice to Bob
    const transfer = api.tx.balances.transfer(bob.address, aliceFreeBalanceBefore.data.free);

    const hash = await transfer.signAndSend(alice);

    console.log(`Transfer sent with hash ${hash}`);

    const aliceFreeBalanceAfter = await (api.query.system.account(alice.address)) as unknown as AccountInfo;
    const bobFreeBalanceAfter = await (api.query.system.account(bob.address)) as unknown as AccountInfo;

    const aliceLostAmount = aliceFreeBalanceBefore.data.free.sub(aliceFreeBalanceAfter.data.free);
    const bobGainedAmount = bobFreeBalanceAfter.data.free.sub(bobFreeBalanceBefore.data.free);

    console.log(`Alice lost ${aliceLostAmount.toString()} tokens`);
    console.log(`Bob gained ${bobGainedAmount.toString()} tokens`);

    console.log(`Alice has ${aliceFreeBalanceAfter.data.free}`);
    console.log(`Bob has ${bobFreeBalanceAfter.data.free}`);

    return aliceLostAmount == bobGainedAmount ? 1 : 0;
}

run("", {}, {}).then((code) => process.exit(code));