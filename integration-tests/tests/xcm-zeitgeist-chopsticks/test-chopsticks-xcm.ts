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
  beforeAll,
  describeSuite,
  expect,
} from "@moonwall/cli";
import { KeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";
import { RuntimeVersion } from "@polkadot/types/interfaces";
import { AccountInfo, AccountData } from "@polkadot/types/interfaces";
import WebSocket from "ws";

const ZEITGEIST_PARA_ID = 2092;
describeSuite({
  id: "XCM01",
  title: "Chopsticks Zeitgeist XCM Tests",
  foundationMethods: "zombie",
  testCases: function ({ it, context, log }) {
    let zeitgeistParaApi: ApiPromise;
    let relayApi: ApiPromise;
    let assetHubParaApi: ApiPromise;
    let alice: KeyringPair;

    beforeAll(async () => {
      const keyring = new Keyring({ type: "sr25519" });
      alice = keyring.addFromUri("//Alice", { name: "Alice default" });
      zeitgeistParaApi = context.polkadotJs("ZeitgeistPara");
      relayApi = context.polkadotJs("PolkadotRelay");
      assetHubParaApi = context.polkadotJs("AssetHubPara");

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
        assetHubParaApi.consts.system.version as unknown as RuntimeVersion
      ).specName.toString();
      expect(paraHydraDXNetwork, "Para API incorrect").to.contain("hydradx");
    }, 120000);

    it({
      id: "T01",
      title: "Send USDC from AssetHub to Zeitgeist",
      test: async function () {
        const assetHubBalanceBefore = (
          (await assetHubParaApi.query.system.account(
            alice.address
          )) as unknown as AccountInfo
        ).data.free.toBigInt();
        const usdcForeignAsset = { ForeignAsset: 4 };
        const zeitgeistUSDCBalanceBefore = (
          (await zeitgeistParaApi.query.tokens.accounts(
            alice.address,
            usdcForeignAsset
          )) as AccountData
        ).free.toBigInt();

        const amount: bigint = BigInt("100000");
        const aliceAccountId = assetHubParaApi
          .createType("AccountId32", alice.address)
          .toHex();
        const destination = {
          V3: {
            parents: 1,
            interior: {
              X1: [{ Parachain: ZEITGEIST_PARA_ID }],
            },
          },
        };
        const beneficiary = {
          V3: {
            parents: 1,
            interior: {
              X1: [{ AccountId32: aliceAccountId }],
            },
          },
        };
        const assets = {
          V3: [
            {
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X2: [{ PalletInstance: 50 }, { GeneralIndex: 1337 }],
                  },
                },
              },
              fun: { Fungible: amount },
            },
          ],
        };
        const feeAssetItem = 0;

        const xcmTransfer =
          assetHubParaApi.tx.polkadotXcm.reserveTransferAssets(
            destination,
            beneficiary,
            assets,
            feeAssetItem
          );

        const { partialFee, weight } = await xcmTransfer.paymentInfo(
          alice.address
        );
        const transferFee: bigint = partialFee.toBigInt();

        await xcmTransfer.signAndSend(alice, { nonce: -1 });

        // RpcError: 1: Block 0x... not found, if using this `await context.createBlock({ providerName: "ReceiverPara", count: 1 });`
        // Reported Bug here https://github.com/Moonsong-Labs/moonwall/issues/343

        // use a workaround for creating a block
        const newBlockPromise = new Promise((resolve, reject) => {
          // ws://127.0.0.1:8001 represents the AssetHub parachain
          const ws = new WebSocket("ws://127.0.0.1:8001");

          ws.on("open", function open() {
            const message = {
              jsonrpc: "2.0",
              id: 1,
              method: "dev_newBlock",
              params: [{ count: 1 }],
            };

            ws.send(JSON.stringify(message));
          });

          ws.on("message", async function message(data) {
            const dataObj = JSON.parse(data.toString());
            log("Received message:", dataObj);
            resolve(dataObj.result);
          });

          ws.on("error", function error(error) {
            log("Error:", error.toString());
            reject(error);
          });
        });

        await newBlockPromise;

        const assetHubBalanceAfter = (
          (await assetHubParaApi.query.system.account(
            alice.address
          )) as unknown as AccountInfo
        ).data.free.toBigInt();
        expect(
          assetHubBalanceBefore - assetHubBalanceAfter,
          "Unexpected balance diff"
        ).toBe(amount + transferFee);

        await context.createBlock({
          providerName: "ZeitgeistPara",
          count: 1,
          allowFailures: false,
        });
        const zeitgeistUSDCBalanceAfter: bigint = (
          (await zeitgeistParaApi.query.tokens.accounts(
            alice.address,
            usdcForeignAsset
          )) as AccountData
        ).free.toBigInt();
      },
    });
  },
});
