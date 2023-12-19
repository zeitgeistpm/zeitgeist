import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { KeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";

describeSuite({
    id: "CZXCM",
    title: "Chopsticks Zeitgeist XCM Tests",
    foundationMethods: "chopsticks",
    testCases: function ({ it, context, log }) {
        let relayApi: ApiPromise;
        let zeitgeistParaApi: ApiPromise;
        let hydradxParaApi: ApiPromise;
        let alice: KeyringPair;

        beforeAll(async () => {
            const keyring = new Keyring({ type: "sr25519" });
            alice = keyring.addFromUri("//Alice", { name: "Alice default" });
            relayApi = context.polkadotJs("PolkadotRelay");
            zeitgeistParaApi = context.polkadotJs("ZeitgeistPara");
            hydradxParaApi = context.polkadotJs("HydraDXPara");

            const relayNetwork = relayApi.consts.system.version.specName.toString();
            expect(relayNetwork, "Relay API incorrect").to.contain("polkadot");

            const paraZeitgeistNetwork = zeitgeistParaApi.consts.system.version.specName.toString();
            expect(paraZeitgeistNetwork, "Para API incorrect").to.contain("zeitgeist");

            const paraHydraDXNetwork = hydradxParaApi.consts.system.version.specName.toString();
            expect(paraHydraDXNetwork, "Para API incorrect").to.contain("hydradx");
        });

        it({
            id: "T1",
            timeout: 60000,
            title: "Can send ZTG to HydraDX",
            test: async () => {
                const keyring = new Keyring({ type: "sr25519" });
                const alice = keyring.addFromUri("//Alice", { name: "Alice default" });
                const bob = keyring.addFromUri("//Bob", { name: "Bob default" });

                const zeitgeistBalanceBefore = (await zeitgeistParaApi.query.system.account(alice.address)).data.free.toBigInt();
                const hydradxBalanceBefore = (await hydradxParaApi.query.tokens.accounts(bob.address, 12)).free.toBigInt();

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

                await context.createBlock({ providerName: "HydraDXPara", count: 1, logger: log });
                const hydradxBalanceAfter = (await hydradxParaApi.query.tokens.accounts(bob.address, 12)).free.toBigInt();
                expect(hydradxBalanceBefore < hydradxBalanceAfter).to.be.true;
            },
        });
    },
});