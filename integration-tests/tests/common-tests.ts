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

import { expect, ChopsticksContext } from "@moonwall/cli";
import { generateKeyringPair } from "@moonwall/util";
import { ApiPromise, Keyring } from "@polkadot/api";
import { AccountInfo, AccountData } from "@polkadot/types/interfaces";
import WebSocket from "ws";
import { Debugger } from "debug";

const MAX_BALANCE_TRANSFER_TRIES = 5;

export async function canCreateBlocks(
  context: ChopsticksContext,
  providerName: string,
  paraApi: ApiPromise
) {
  const currentHeight = (
    await paraApi.rpc.chain.getBlock()
  ).block.header.number.toNumber();
  await context.createBlock({ providerName: providerName, count: 2 });
  const newHeight = (
    await paraApi.rpc.chain.getBlock()
  ).block.header.number.toNumber();
  expect(newHeight - currentHeight, "Block difference is not correct!").toBe(2);
}

export async function canSendBalanceTransfer(
  context: ChopsticksContext,
  providerName: string,
  paraApi: ApiPromise
) {
  const randomAccount = generateKeyringPair("sr25519");
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice", { name: "Alice default" });

  let tries = 0;
  const amount = BigInt("1000000000");
  const balanceBefore = (
    (await paraApi.query.system.account(
      randomAccount.address
    )) as unknown as AccountInfo
  ).data.free.toBigInt();

  /// It might happen that by accident we hit a session change
  /// A block in which a session change occurs cannot hold any tx
  /// Chopsticks does not have the notion of tx pool either, so we need to retry
  /// Therefore we just retry at most MAX_BALANCE_TRANSFER_TRIES
  while (tries < MAX_BALANCE_TRANSFER_TRIES) {
    const tx = await paraApi.tx.balances.transfer(
      randomAccount.address,
      amount
    );
    const txHash = tx.signAndSend(alice, { nonce: -1 });
    const result = await context.createBlock({
      providerName: providerName,
      count: 1,
    });

    const block = await paraApi.rpc.chain.getBlock(result.result);
    const includedTxHashes = block.block.extrinsics.map((x) =>
      x.hash.toString()
    );
    if (includedTxHashes.includes(txHash.toString())) {
      break;
    }
    tries++;
  }

  // without this, the xcm transfer `canSendXcmTransfer` test below has a timeout
  await context.createBlock({ providerName: providerName, count: 1 });

  const balanceAfter = (
    (await paraApi.query.system.account(
      randomAccount.address
    )) as unknown as AccountInfo
  ).data.free.toBigInt();
  expect(balanceAfter > balanceBefore, "Balance did not increase").toBeTruthy();
}

export async function canSendXcmTransfer(
  context: ChopsticksContext,
  log: Debugger,
  senderProviderName: string,
  senderParaApi: ApiPromise,
  receiverParaApi: ApiPromise,
  receiverParaId: number,
  tokensIndex: number
) {
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice", { name: "Alice default" });
  const bob = keyring.addFromUri("//Bob", { name: "Bob default" });

  const senderBalanceBefore = (
    (await senderParaApi.query.system.account(
      alice.address
    )) as unknown as AccountInfo
  ).data.free.toBigInt();
  const receiverBalanceBefore = (
    (await receiverParaApi.query.tokens.accounts(
      bob.address,
      tokensIndex
    )) as AccountData
  ).free.toBigInt();

  const ztg = { Ztg: null };
  const amount: bigint = BigInt("192913122185847181");
  const bobAccountId = senderParaApi
    .createType("AccountId32", bob.address)
    .toHex();
  const destination = {
    V3: {
      parents: 1,
      interior: {
        X2: [
          { Parachain: receiverParaId },
          { AccountId32: { id: bobAccountId, network: null } },
        ],
      },
    },
  };
  const destWeightLimit = { Unlimited: null };

  const xcmTransfer = senderParaApi.tx.xTokens.transfer(
    ztg,
    amount,
    destination,
    destWeightLimit
  );

  const { partialFee, weight } = await xcmTransfer.paymentInfo(alice.address);
  const transferFee: bigint = partialFee.toBigInt();

  await xcmTransfer.signAndSend(alice, { nonce: -1 });

  await context.createBlock({
    providerName: senderProviderName,
    count: 1,
    allowFailures: false,
  });

  const senderBalanceAfter = (
    (await senderParaApi.query.system.account(
      alice.address
    )) as unknown as AccountInfo
  ).data.free.toBigInt();
  expect(
    senderBalanceBefore - senderBalanceAfter,
    "Unexpected balance diff"
  ).toBe(amount + transferFee);

  // RpcError: 1: Block 0x... not found, if using this `await context.createBlock({ providerName: "ReceiverPara", count: 1 });`
  // Reported Bug here https://github.com/Moonsong-Labs/moonwall/issues/343

  // use a workaround for creating a block
  const newBlockPromise = new Promise((resolve, reject) => {
    // ws://127.0.0.1:8001 represents the receiver chain endpoint
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
  const receiverBalanceAfter: bigint = (
    (await receiverParaApi.query.tokens.accounts(
      bob.address,
      tokensIndex
    )) as AccountData
  ).free.toBigInt();
  expect(
    receiverBalanceAfter > receiverBalanceBefore,
    "Balance did not increase"
  ).toBeTruthy();
  const xcmFee: bigint = receiverBalanceBefore + amount - receiverBalanceAfter;
  console.log(
    `receiverBalanceBefore: ${receiverBalanceBefore}; amount: ${amount}; transferFee: ${transferFee}; receiverBalanceAfter: ${receiverBalanceAfter}; xcmFee: ${xcmFee}`
  );
  console.log(`xcmFee: ${xcmFee}`);
  // between 0.01 ZTG and 0.10 ZTG XCM fee
  const approxXcmFeeLow = 100000000;
  const approxXcmFeeHigh = 1000000000;
  expect(xcmFee).toBeGreaterThanOrEqual(approxXcmFeeLow);
  expect(xcmFee).toBeLessThanOrEqual(approxXcmFeeHigh);
  expect(
    receiverBalanceAfter - receiverBalanceBefore,
    "Unexpected xcm transfer balance diff"
  ).toBe(amount - xcmFee);
}

export async function canExecuteAtomicSwap(
  context: ChopsticksContext,
  log: Debugger,
  senderProviderName: string,
  senderParaApi: ApiPromise,
  senderParaId: number,
  hydradxParaApi: ApiPromise,
  hydradxParaId: number
) {
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice", { name: "Alice default" });
  const bob = keyring.addFromUri("//Bob", { name: "Bob default" });

  const senderBalanceBefore = (
    (await senderParaApi.query.system.account(
      alice.address
    )) as unknown as AccountInfo
  ).data.free.toBigInt();
  const tokensIndex = 0;
  const receiverBalanceBefore = (
    (await hydradxParaApi.query.tokens.accounts(
      bob.address,
      tokensIndex
    )) as AccountData
  ).free.toBigInt();

  const ztg = { Ztg: null };
  const amount: bigint = BigInt("192913122185847181");
  const bobAccountId = senderParaApi
    .createType("AccountId32", bob.address)
    .toHex();

  // TODO: fill in bobs AccountId32 address in beneficiary of DepositAsset for the polkadot js org reference below
  console.log("bobAccountId", bobAccountId);

  // TODO: register HDX token on Zeitgeist chain first in order to swap ZTG for HDX on HydraDX chain

  const dest = {
    parents: 1,
    interior: {
      X1: { Parachain: hydradxParaId },
    },
  };

  const destination = {
    V3: dest,
  };

  // taken from here https://github.com/galacticcouncil/HydraDX-node/blob/e3821e078bdb72a0416f8aebca21ba4a7a599f64/runtime/hydradx/src/xcm.rs#L312-L315
  const localHDX = {
    parents: 0,
    interior: {
      X1: { GeneralIndex: 0 },
    },
  };

  // TODO: mint HDX token on HydraDX for the swap executor to pay for the XCM execution
  const buyExecution = {
    BuyExecution: {
      fees: {
        id: {
          Concrete: localHDX,
        },
        fun: {
          // 100 HDX (12 decimals base)
          Fungible: 100_000_000_000_000n,
        },
      },
      weightLimit: {
        Unlimited: null,
      },
    },
  };

  const ztgOnHydraDX = {
    parents: 1,
    interior: {
      X2: [
        { Parachain: senderParaId },
        {
          GeneralKey: {
            length: 2,
            data: "0x0001000000000000000000000000000000000000000000000000000000000000",
          },
        },
      ],
    },
  };

  const exchangeAsset = {
    ExchangeAsset: {
      give: {
        Definite: [
          {
            id: {
              Concrete: ztgOnHydraDX,
            },
            fun: {
              // 100 ZTG (10 decimals base)
              Fungible: 1_000_000_000_000n,
            },
          },
        ],
      },
      want: {
        id: {
          Concrete: localHDX,
        },
        fun: {
          // 50 HDX (12 decimals base)
          Fungible: 50_000_000_000_000n,
        },
      },
      // Reference: https://github.com/paritytech/polkadot-sdk/blob/289f5bbf7a45dc0380904a435464b15ec711ed03/polkadot/xcm/src/v3/mod.rs#L722-L724
      // give as little ZTG as possible to receive at least 50 HDX
      maximal: false,
    },
  };

  const depositAsset = {
    DepositAsset: {
      assets: { Wild: { AllCounted: 2 } },
      beneficiary: {
        parents: 0,
        interior: {
          X1: { AccountId32: { id: bobAccountId, network: null } },
        },
      },
    },
  };

  // executed on HydraDX
  const hydradxXcm = {
    V3: [buyExecution, exchangeAsset, depositAsset],
  };

  const setFeesMode = {
    SetFeesMode: {
      jitWithdraw: true,
    },
  };

  const localZTG = {
    parents: 0,
    interior: {
      X1: {
        GeneralKey: {
          length: 2,
          data: "0x0001000000000000000000000000000000000000000000000000000000000000",
        },
      },
    },
  };

  const assets = [
    {
      id: {
        Concrete: localZTG,
      },
      fun: {
        // 100 ZTG (10 decimals base)
        Fungible: 1_000_000_000_000n,
      },
    },
  ];

  const transferReserveAsset = {
    TransferReserveAsset: {
      assets: assets,
      dest: dest,
      xcm: hydradxXcm,
    },
  };

  // Reference: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fzeitgeist-rpc.dwellir.com#/extrinsics/decode/0x7a0003010100c91f03082b0105040000010602000100000000000000000000000000000000000000000000000000000000000000070010a5d4e8010100c91f0c130000010500000b00407a10f35a000f000400010200b1200602000100000000000000000000000000000000000000000000000000000000000000070010a5d4e8040000010500000b00203d88792d000d0102080001010032324422424000000000f3230040020423040032003000f0302f30300f000323
  const xcmMessage = {
    V3: [setFeesMode, transferReserveAsset],
  };

  const xcmTransfer = senderParaApi.tx.xTokens.transfer(
    ztg,
    amount,
    destination,
    destWeightLimit
  );

  const { partialFee, weight } = await xcmTransfer.paymentInfo(alice.address);
  const transferFee: bigint = partialFee.toBigInt();

  await xcmTransfer.signAndSend(alice, { nonce: -1 });

  await context.createBlock({
    providerName: senderProviderName,
    count: 1,
    allowFailures: false,
  });

  const senderBalanceAfter = (
    (await senderParaApi.query.system.account(
      alice.address
    )) as unknown as AccountInfo
  ).data.free.toBigInt();
  expect(
    senderBalanceBefore - senderBalanceAfter,
    "Unexpected balance diff"
  ).toBe(amount + transferFee);

  // RpcError: 1: Block 0x... not found, if using this `await context.createBlock({ providerName: "ReceiverPara", count: 1 });`
  // Reported Bug here https://github.com/Moonsong-Labs/moonwall/issues/343

  // use a workaround for creating a block
  const newBlockPromise = new Promise((resolve, reject) => {
    // ws://127.0.0.1:8001 represents the receiver chain endpoint
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
}
