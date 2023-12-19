import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { AccountInfo } from '@polkadot/types/interfaces';

// Addresses for Alice and Bob on the dev chain
const ALICE = '//Alice';
const BOB = '//Bob';

export const run = async (nodeName: string, networkInfo: any, args: any) => {
    const provider = new WsProvider('ws://127.0.0.1:9966');
    const api = await ApiPromise.create({ provider });

    // Wait for the crypto library to be ready
    await cryptoWaitReady();

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri(ALICE);
    const bob = keyring.addFromUri(BOB);

    const aliceFreeBalanceBefore = (await api.query.system.account(alice.address)) as unknown as AccountInfo;
    const bobFreeBalanceBefore = (await api.query.system.account(bob.address)) as unknown as AccountInfo;

    console.log(`Alice has ${aliceFreeBalanceBefore.data.free} before transfer.`);
    console.log(`Bob has ${bobFreeBalanceBefore.data.free} before transfer.`);

    const transfer_amount = "42000000000000000";

    // Create a transfer transaction from Alice to Bob
    const transfer = api.tx.balances.transfer(bob.address, transfer_amount);

    // Get weight info
    const { partialFee, weight } = await transfer.paymentInfo(alice.address);

    console.log(`Transaction weight: ${weight}`);
    console.log(`Transaction fee: ${partialFee.toString()}`);

    // Wait for the transaction to be finalized
    await new Promise((resolve, reject) => {
        transfer.signAndSend(alice, ({ status }) => {
            if (status.isInBlock || status.isFinalized) {
                resolve(status);
            }
        }).catch(reject);
    });

    console.log(`Transfer sent`);

    const aliceFreeBalanceAfter = (await api.query.system.account(alice.address)) as unknown as AccountInfo;
    const bobFreeBalanceAfter = (await api.query.system.account(bob.address)) as unknown as AccountInfo;

    const aliceLostAmount = aliceFreeBalanceBefore.data.free.sub(aliceFreeBalanceAfter.data.free);
    const bobGainedAmount = bobFreeBalanceAfter.data.free.sub(bobFreeBalanceBefore.data.free);

    console.log(`Alice lost ${aliceLostAmount.toString()} tokens.`);
    console.log(`Bob gained ${bobGainedAmount.toString()} tokens.`);

    console.log(`Alice has ${aliceFreeBalanceAfter.data.free} after transfer.`);
    console.log(`Bob has ${bobFreeBalanceAfter.data.free} after transfer.`);

    const testPassed = transfer_amount == bobGainedAmount.toString();
    console.log(`Test passed: ${testPassed}`);
    await api.disconnect();

    return testPassed ? 1 : 0;
}