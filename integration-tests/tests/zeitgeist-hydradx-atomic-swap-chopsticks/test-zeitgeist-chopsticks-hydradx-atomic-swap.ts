// Copyright (C) Moondance Labs Ltd.
// Copyright 2024 Forecasting Technologies LTD.
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

import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { KeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";
import { canExecuteAtomicSwap } from "tests/common-tests";
import { RuntimeVersion } from "@polkadot/types/interfaces";

const HYDRADX_PARA_ID = 2034;
describeSuite({
  id: "CZH",
  title: "Chopsticks Zeitgeist HydraDX Atomic Swap Tests",
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
        zeitgeistParaApi.consts.system.version as unknown as RuntimeVersion
      ).specName.toString();
      expect(paraZeitgeistNetwork, "Para API incorrect").to.contain(
        "zeitgeist"
      );

      const relayNetwork = (
        relayApi.consts.system.version as unknown as RuntimeVersion
      ).specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("polkadot");

      const paraHydraDXNetwork = (
        hydradxParaApi.consts.system.version as unknown as RuntimeVersion
      ).specName.toString();
      expect(paraHydraDXNetwork, "Para API incorrect").to.contain("hydradx");
    }, 120000);

    it({
      id: "T1",
      timeout: 60000,
      title: "Can execute atomic swap on HydraDX",
      test: async () => {
        await canExecuteAtomicSwap(
          context,
          log,
          "ZeitgeistPara",
          zeitgeistParaApi,
          hydradxParaApi,
          HYDRADX_PARA_ID
        );
      },
    });
  },
});
