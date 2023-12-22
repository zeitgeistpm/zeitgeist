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
import { RuntimeVersion } from "@polkadot/types/interfaces";

const ZEITGEIST_TOKENS_INDEX = 12;
const BASILISK_PARA_ID = 2090;
describeSuite({
  id: "CAN",
  title: "Chopsticks Battery Station Post-Upgrade Tests",
  foundationMethods: "chopsticks",
  testCases: function ({ it, context, log }) {
    let batteryStationParaApi: ApiPromise;
    let relayApi: ApiPromise;
    let basiliskParaApi: ApiPromise;
    let alice: KeyringPair;

    beforeAll(async () => {
      const keyring = new Keyring({ type: "sr25519" });
      alice = keyring.addFromUri("//Alice", { name: "Alice default" });
      batteryStationParaApi = context.polkadotJs("BatteryStationPara");
      relayApi = context.polkadotJs("RococoRelay");
      basiliskParaApi = context.polkadotJs("BasiliskPara");

      const paraZeitgeistNetwork = (
        batteryStationParaApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(paraZeitgeistNetwork, "Para API incorrect").to.contain(
        "zeitgeist"
      );

      const relayNetwork = (
        relayApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("rococo");

      const paraBasiliskNetwork = (
        basiliskParaApi.consts.system.version as RuntimeVersion
      ).specName.toString();
      expect(paraBasiliskNetwork, "Para API incorrect").to.contain("basilisk");

      const rtBefore = (
        batteryStationParaApi.consts.system.version as RuntimeVersion
      ).specVersion.toNumber();
      log(`About to upgrade to runtime at:`);
      log(MoonwallContext.getContext().rtUpgradePath);

      await context.upgradeRuntime();

      const rtafter = (
        batteryStationParaApi.consts.system.version as RuntimeVersion
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
        await canCreateBlocks(
          context,
          "BatteryStationPara",
          batteryStationParaApi
        );
      },
    });

    it({
      id: "T2",
      timeout: 60000,
      title: "Can send balance transfers",
      test: async () => {
        await canSendBalanceTransfer(
          context,
          "BatteryStationPara",
          batteryStationParaApi
        );
      },
    });

    /*
    Currently not working, bug tracked here https://github.com/galacticcouncil/HydraDX-node/issues/725

    it({
      id: "T3",
      timeout: 60000,
      title: "Can send ZBS to Basilisk",
      test: async () => {
        await canSendXcmTransfer(
          context,
          log,
          "BatteryStationPara",
          batteryStationParaApi,
          basiliskParaApi,
          BASILISK_PARA_ID,
          ZEITGEIST_TOKENS_INDEX
        );
      },
    });
    */
  },
});
