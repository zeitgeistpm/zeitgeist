// Copyright 2022-2025 Forecasting Technologies LTD.
// Copyright (C) Moondance Labs Ltd.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

import {
  MoonwallContext,
  beforeAll,
  describeSuite,
  expect,
} from "@moonwall/cli";
import { KeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";
import { u8aConcat } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import fs from "node:fs";
import { RuntimeVersion } from "@polkadot/types/interfaces";

describeSuite({
  id: "R01",
  title: "Zombie Zeitgeist Upgrade Test",
  foundationMethods: "zombie",
  testCases: function ({ it, context, log }) {
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;
    let alice: KeyringPair;

    beforeAll(async () => {
      const keyring = new Keyring({ type: "sr25519" });
      alice = keyring.addFromUri("//Alice", { name: "Alice default" });
      paraApi = context.polkadotJs("parachain");
      relayApi = context.polkadotJs("Relay");

      const relayNetwork = (
        relayApi.consts.system.version as unknown as RuntimeVersion
      ).specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("rococo");

      const paraNetwork = (
        paraApi.consts.system.version as unknown as RuntimeVersion
      ).specName.toString();
      expect(paraNetwork, "Para API incorrect").to.contain("zeitgeist");

      const currentBlock = (
        await paraApi.rpc.chain.getBlock()
      ).block.header.number.toNumber();
      expect(currentBlock, "Parachain not producing blocks").to.be.greaterThan(
        0
      );
    }, 120000);

    it({
      id: "T01",
      title: "Blocks are being produced on parachain",
      test: async function () {
        const blockNum = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();
        expect(blockNum).to.be.greaterThan(0);
      },
    });

    it({
      id: "T02",
      title: "Chain can be upgraded",
      timeout: 600000,
      test: async function () {
        const blockNumberBefore = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();
        const currentCode = await paraApi.rpc.state.getStorage(":code");
        const codeString = currentCode.toString();

        const moonwallContext = await MoonwallContext.getContext();
        const specVersion = (
          paraApi.consts.system.version as unknown as RuntimeVersion
        ).specVersion.toNumber();
        log(
          `Parachain specVersion=${specVersion}, block=${blockNumberBefore}, rtUpgradePath=${moonwallContext.rtUpgradePath}`
        );
        log(
          "Moonwall Context providers: " +
            moonwallContext.providers.map((p) => p.name).join(", ")
        );
        const wasm = fs.readFileSync(moonwallContext.rtUpgradePath);
        const rtHex = `0x${wasm.toString("hex")}`;

        if (rtHex === codeString) {
          log("Runtime already upgraded, skipping test");
          return;
        } else {
          log("Runtime not upgraded, proceeding with test");
          log(
            "Current runtime hash: " +
              rtHex.slice(0, 10) +
              "..." +
              rtHex.slice(-10)
          );
          log(
            "New runtime hash: " +
              codeString.slice(0, 10) +
              "..." +
              codeString.slice(-10)
          );
        }

        const txStatus = async (tx: any, label: string) =>
          new Promise<void>((resolve, reject) => {
            let unsubscribe: (() => void) | undefined;
            tx.signAndSend(alice, (result: any) => {
              if (result.dispatchError) {
                // Dispatch errors won't throw, so surface them explicitly.
                const errText = result.dispatchError.toString();
                log(`${label} dispatchError=${errText}`);
                reject(new Error(`${label} failed: ${errText}`));
                unsubscribe?.();
                return;
              }
              log(
                `${label} status=${result.status?.type ?? "unknown"}, events=${result.events
                  ?.map((ev: any) => `${ev.event.section}.${ev.event.method}`)
                  .join(",")}`
              );
              if (result.status?.isInBlock || result.status?.isFinalized) {
                unsubscribe?.();
                resolve();
              }
            })
              .then((unsub: () => void) => {
                unsubscribe = unsub;
              })
              .catch(reject);
          });

        const findCall = (callName: string) => {
          for (const [section, calls] of Object.entries(paraApi.tx)) {
            const typedCalls = calls as Record<string, any>;
            if (typedCalls?.[callName]) {
              return { call: typedCalls[callName], section };
            }
          }
          return undefined;
        };

        const upgradeCallLocation = {
          authorize: undefined as string | undefined,
          enact: undefined as string | undefined,
        };

        const authorizeUpgradeResult = findCall("authorizeUpgrade");
        if (authorizeUpgradeResult) {
          upgradeCallLocation.authorize = authorizeUpgradeResult.section;
        }

        // On this SDK the enact call lives in frame-system as applyAuthorizedUpgrade.
        const applyAuthorizedUpgradeResult = findCall("applyAuthorizedUpgrade");
        if (applyAuthorizedUpgradeResult) {
          upgradeCallLocation.enact = applyAuthorizedUpgradeResult.section;
        }

        const authorizeUpgrade = authorizeUpgradeResult?.call;
        const applyAuthorizedUpgrade = applyAuthorizedUpgradeResult?.call;
        const upgradeAvailable = authorizeUpgrade && applyAuthorizedUpgrade;

        const upgradeSections = Object.keys(paraApi.tx).filter((section) =>
          /parachain|upgrade|system/i.test(section)
        );
        log(`tx sections matching /parachain|upgrade|system/: ${upgradeSections.join(",")}`);

        if (upgradeAvailable) {
          // Zeitgeist runtime blocks `setCode`, so use the authorized upgrade flow.
          const wasmHash = blake2AsHex(wasm);
          log("Authorizing runtime upgrade via system.authorizeUpgrade");
          const authorizeTx =
            authorizeUpgrade.meta.args.length === 1
              ? authorizeUpgrade(wasmHash)
              : authorizeUpgrade(wasmHash, true);
          log(
            `authorizeUpgrade located in section=${upgradeCallLocation.authorize}, args=${authorizeUpgrade.meta.args.length}`
          );
          await txStatus(
            paraApi.tx.sudo.sudo(authorizeTx),
            "authorizeUpgrade"
          );

          log("Waiting for validation function approval");
          await context.waitBlock(2);

          log("Enacting authorized upgrade");
          log(
            `applyAuthorizedUpgrade located in section=${upgradeCallLocation.enact}, args=${applyAuthorizedUpgrade.meta.args.length}`
          );
          await txStatus(
            paraApi.tx.sudo.sudo(applyAuthorizedUpgrade(rtHex)),
            "applyAuthorizedUpgrade"
          );
        } else {
          throw new Error(
            "Runtime upgrade calls missing in metadata; expected system.authorizeUpgrade/applyAuthorizedUpgrade"
          );
        }

        await context.waitBlock(2);
        const blockNumberAfter = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();
        const codeAfter = (await paraApi.rpc.state.getStorage(":code"))?.toString();
        log(
          `Before: #${blockNumberBefore}, After: #${blockNumberAfter}, code changed=${
            codeAfter !== codeString
          }`
        );
        log(
          `Code (before): ${codeString.slice(0, 10)}...${codeString.slice(-10)}, code (after): ${
            codeAfter ? codeAfter.slice(0, 10) + "..." + codeAfter.slice(-10) : "undefined"
          }`
        );
        expect(
          blockNumberAfter,
          "Block number did not increase"
        ).to.be.greaterThan(blockNumberBefore);
        expect(codeAfter, "Runtime code should match upgraded wasm").to.equal(rtHex);
      },
    });

    it({
      id: "T03",
      title: "Relay timestamp (from relay proof) is present and increases across blocks",
      timeout: 120000,
      test: async function () {
        const runtimePrefix = xxhashAsU8a("runtime", 128);
        const tsPrefix = xxhashAsU8a("RelayTimestampNow", 128);
        const relayTsKey = u8aConcat(runtimePrefix, tsPrefix);

        const readRelayTs = async (): Promise<number> => {
          const raw = await paraApi.rpc.state.getStorage(relayTsKey);
          expect(raw, "RelayTimestampNow storage should exist").to.not.be.null;
          const rawHex = raw?.toHex();
          expect(rawHex, "RelayTimestampNow should decode to hex").to.not.be.undefined;
          log(`RelayTimestampNow raw=${rawHex}`);
          return paraApi.createType("u64", rawHex ?? 0).toNumber();
        };

        let tsRelay1 = 0;
        let retries = 0;
        while (tsRelay1 === 0 && retries < 5) {
          log(`Attempt ${retries + 1}: reading RelayTimestampNow`);
          tsRelay1 = await readRelayTs();
          if (tsRelay1 === 0) {
            await context.waitBlock(1);
          }
          retries++;
        }

        const tsPara1 = (await paraApi.query.timestamp.now()).toNumber();
        const block1 = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();

        expect(tsRelay1, "Initial relay timestamp should be non-zero").to.be.greaterThan(0);
        expect(
          tsPara1,
          "Parachain timestamp should mirror relay timestamp"
        ).to.equal(tsRelay1);

        await context.waitBlock(2);

        const tsRelay2 = await readRelayTs();
        const tsPara2 = (await paraApi.query.timestamp.now()).toNumber();
        const block2 = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();

        expect(block2, "Block height should advance").to.be.greaterThan(block1);
        expect(
          tsRelay2,
          "Relay timestamp should increase with new relay proofs"
        ).to.be.greaterThan(tsRelay1);
        expect(tsPara2, "Parachain timestamp should mirror relay timestamp").to.equal(tsRelay2);
      },
    });
  },
});
