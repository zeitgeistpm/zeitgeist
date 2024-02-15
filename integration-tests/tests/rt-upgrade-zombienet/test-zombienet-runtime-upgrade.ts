// Copyright (C) Moondance Labs Ltd.
// Copyright 2022-2024 Forecasting Technologies LTD.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
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
        relayApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("rococo");

      const paraNetwork = (
        paraApi.consts.system.version as RuntimeVersion
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

        const moonwallContext = MoonwallContext.getContext();
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

        await context.upgradeRuntime({ from: alice, logger: log });
        await context.waitBlock(2);
        const blockNumberAfter = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();
        log(`Before: #${blockNumberBefore}, After: #${blockNumberAfter}`);
        expect(
          blockNumberAfter,
          "Block number did not increase"
        ).to.be.greaterThan(blockNumberBefore);
      },
    });
  },
});
