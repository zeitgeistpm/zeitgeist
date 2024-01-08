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
import {
  canCreateBlocks,
  canSendBalanceTransfer,
  canSendXcmTransfer,
} from "tests/common-tests";
import { RuntimeVersion } from "@polkadot/types/interfaces";

const ZEITGEIST_TOKENS_INDEX = 12;
const HYDRADX_PARA_ID = 2034;
describeSuite({
  id: "CAN",
  title: "Chopsticks Zeitgeist Post-Upgrade Tests",
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

      const paraZeitgeistNetwork = (
        zeitgeistParaApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(paraZeitgeistNetwork, "Para API incorrect").to.contain(
        "zeitgeist"
      );

      const relayNetwork = (
        relayApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("polkadot");

      const paraHydraDXNetwork = (
        hydradxParaApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(paraHydraDXNetwork, "Para API incorrect").to.contain("hydradx");

      const rtBefore = (
        zeitgeistParaApi.consts.system.version as RuntimeVersion
      ).specVersion.toNumber();
      log(`About to upgrade to runtime at:`);
      log(MoonwallContext.getContext().rtUpgradePath);

      await context.upgradeRuntime();

      const rtafter = (
        zeitgeistParaApi.consts.system.version as RuntimeVersion
      ).specVersion.toNumber();
      log(
        `RT upgrade has increased specVersion from ${rtBefore} to ${rtafter}`
      );
    }, 60000);

    it({
      id: "T1",
      timeout: 60000,
      title: "Can create new blocks",
      test: async () => {
        await canCreateBlocks(context, "ZeitgeistPara", zeitgeistParaApi);
      },
    });

    it({
      id: "T2",
      timeout: 60000,
      title: "Can send balance transfers",
      test: async () => {
        await canSendBalanceTransfer(
          context,
          "ZeitgeistPara",
          zeitgeistParaApi
        );
      },
    });

    it({
      id: "T3",
      timeout: 60000,
      title: "Can send ZTG to HydraDX",
      test: async () => {
        await canSendXcmTransfer(
          context,
          log,
          "ZeitgeistPara",
          zeitgeistParaApi,
          hydradxParaApi,
          HYDRADX_PARA_ID,
          ZEITGEIST_TOKENS_INDEX
        );
      },
    });
  },
});
