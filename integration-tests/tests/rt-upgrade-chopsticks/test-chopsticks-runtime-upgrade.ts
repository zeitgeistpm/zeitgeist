// Copyright (C) Moondance Labs Ltd.

import { MoonwallContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import { generateKeyringPair } from "@moonwall/util";
import { KeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";

const MAX_BALANCE_TRANSFER_TRIES = 5;
const ZEITGEIST_TOKENS_INDEX = 12;
describeSuite({
    id: "CAN",
    title: "Chopsticks Zeitgeist or Battery Station Post-Upgrade Tests",
    foundationMethods: "chopsticks",
    testCases: function ({ it, context, log }) {
        let zeitgeistParaApi: ApiPromise;
        let relayApi: ApiPromise;
        let hydradxParaApi: ApiPromise;
        let alice: KeyringPair;

        beforeAll(async () => {
            const keyring = new Keyring({ type: "sr25519" });
            alice = keyring.addFromUri("//Alice", { name: "Alice default" });
            zeitgeistParaApi = context.polkadotJs("ZeitgeistPara");
            relayApi = context.polkadotJs("PolkadotRelay");
            hydradxParaApi = context.polkadotJs("HydraDXPara");

            const paraZeitgeistNetwork = zeitgeistParaApi.consts.system.version.specName.toString();
            expect(paraZeitgeistNetwork, "Para API incorrect").to.contain("zeitgeist");

            const relayNetwork = relayApi.consts.system.version.specName.toString();
            expect(relayNetwork, "Relay API incorrect").to.contain("polkadot");

            const paraHydraDXNetwork = hydradxParaApi.consts.system.version.specName.toString();
            expect(paraHydraDXNetwork, "Para API incorrect").to.contain("hydradx");

            const rtBefore = zeitgeistParaApi.consts.system.version.specVersion.toNumber();
            log(`About to upgrade to runtime at:`);
            log(MoonwallContext.getContext().rtUpgradePath);

            await context.upgradeRuntime(context);

            const rtafter = zeitgeistParaApi.consts.system.version.specVersion.toNumber();
            log(`RT upgrade has increased specVersion from ${rtBefore} to ${rtafter}`);
        });

        it({
            id: "T1",
            timeout: 60000,
            title: "Can create new blocks",
            test: async () => {
                const currentHeight = (await zeitgeistParaApi.rpc.chain.getBlock()).block.header.number.toNumber();
                await context.createBlock({ providerName: 'ZeitgeistPara', count: 2 });
                const newHeight = (await zeitgeistParaApi.rpc.chain.getBlock()).block.header.number.toNumber();
                expect(newHeight - currentHeight).to.be.equal(2);
            },
        });

        it({
            id: "T2",
            timeout: 60000,
            title: "Can send balance transfers",
            test: async () => {
                const randomAccount = generateKeyringPair("sr25519");
                const keyring = new Keyring({ type: "sr25519" });
                const alice = keyring.addFromUri("//Alice", { name: "Alice default" });

                let tries = 0;
                const balanceBefore = (await zeitgeistParaApi.query.system.account(randomAccount.address)).data.free.toBigInt();

                /// It might happen that by accident we hit a session change
                /// A block in which a session change occurs cannot hold any tx
                /// Chopsticks does not have the notion of tx pool either, so we need to retry
                /// Therefore we just retry at most MAX_BALANCE_TRANSFER_TRIES
                while (tries < MAX_BALANCE_TRANSFER_TRIES) {
                    const txHash = await zeitgeistParaApi.tx.balances
                        .transfer(randomAccount.address, 1_000_000_000)
                        .signAndSend(alice);
                    const result = await context.createBlock({ providerName: 'ZeitgeistPara', count: 1 });

                    const block = await zeitgeistParaApi.rpc.chain.getBlock(result.result);
                    const includedTxHashes = block.block.extrinsics.map((x) => x.hash.toString());
                    if (includedTxHashes.includes(txHash.toString())) {
                        break;
                    }
                    tries++;
                }

                const balanceAfter = (await zeitgeistParaApi.query.system.account(randomAccount.address)).data.free.toBigInt();
                expect(balanceBefore < balanceAfter).to.be.true;
            },
        });

        it({
            id: "T3",
            timeout: 60000,
            title: "Can send ZTG to HydraDX",
            test: async () => {
                const keyring = new Keyring({ type: "sr25519" });
                const alice = keyring.addFromUri("//Alice", { name: "Alice default" });
                const bob = keyring.addFromUri("//Bob", { name: "Bob default" });

                const zeitgeistBalanceBefore = (await zeitgeistParaApi.query.system.account(alice.address)).data.free.toBigInt();
                const hydradxBalanceBefore = (await hydradxParaApi.query.tokens.accounts(bob.address, ZEITGEIST_TOKENS_INDEX)).free.toBigInt();

                const ztg = { 'Ztg': null };
                const amount = "192913122185847181";
                const bobAccountId = zeitgeistParaApi.createType("AccountId32", bob.address).toHex();
                const destination = {
                    V3: {
                        parents: 1,
                        interior: { X2: [{ Parachain: 2034 }, { AccountId32: { id: bobAccountId, network: null } }] },
                    }
                };
                const destWeightLimit = { Unlimited: null };
                // Create a promise that resolves when the transaction is finalized
                const finalizedPromise = new Promise((resolve, reject) => {
                    zeitgeistParaApi.tx.xTokens.transfer(ztg, amount, destination, destWeightLimit).signAndSend(alice, ({ status }) => {
                        if (status.isFinalized) {
                            log(`Transaction finalized at blockHash ${status.asFinalized}`);
                            resolve();
                        } else if (status.isError) {
                            reject(new Error(`Transaction failed with status ${status}`));
                        }
                    });
                });

                // Wait for the transaction to be finalized
                await finalizedPromise;

                const zeitgeistBalanceAfter = (await zeitgeistParaApi.query.system.account(alice.address)).data.free.toBigInt();
                expect(zeitgeistBalanceBefore > zeitgeistBalanceAfter).to.be.true;

                // RpcError: 1: Block 0x... not found, if using this `await context.createBlock({ providerName: "HydraDXPara", count: 1 });`
                // Reported Bug here https://github.com/Moonsong-Labs/moonwall/issues/343

                // use a workaround for creating a block
                // TODO create block somehow manually using chopsticks

                const hydradxBalanceAfter = (await hydradxParaApi.query.tokens.accounts(bob.address, ZEITGEIST_TOKENS_INDEX)).free.toBigInt();
                // expect(hydradxBalanceBefore < hydradxBalanceAfter).to.be.true;
            },
        });
    },
});