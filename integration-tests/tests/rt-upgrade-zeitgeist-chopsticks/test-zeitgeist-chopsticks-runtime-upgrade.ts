// Copyright (C) Moondance Labs Ltd.

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

      const paraZeitgeistNetwork =
        zeitgeistParaApi.consts.system.version.specName.toString();
      expect(paraZeitgeistNetwork, "Para API incorrect").to.contain(
        "zeitgeist"
      );

      const relayNetwork = relayApi.consts.system.version.specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("polkadot");

      const paraHydraDXNetwork =
        hydradxParaApi.consts.system.version.specName.toString();
      expect(paraHydraDXNetwork, "Para API incorrect").to.contain("hydradx");

      const rtBefore =
        zeitgeistParaApi.consts.system.version.specVersion.toNumber();
      log(`About to upgrade to runtime at:`);
      log(MoonwallContext.getContext().rtUpgradePath);

      await context.upgradeRuntime(context);

      const rtafter =
        zeitgeistParaApi.consts.system.version.specVersion.toNumber();
      log(
        `RT upgrade has increased specVersion from ${rtBefore} to ${rtafter}`
      );
    });

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
          log,
          zeitgeistParaApi,
          hydradxParaApi,
          HYDRADX_PARA_ID,
          ZEITGEIST_TOKENS_INDEX
        );
      },
    });
  },
});
