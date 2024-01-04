// Copyright (C) Moondance Labs Ltd.

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
    (await paraApi.query.system.account(randomAccount.address)) as AccountInfo
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
    (await paraApi.query.system.account(randomAccount.address)) as AccountInfo
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
    (await senderParaApi.query.system.account(alice.address)) as AccountInfo
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

  await xcmTransfer.signAndSend(alice, { nonce: -1 });

  await context.createBlock({
    providerName: senderProviderName,
    count: 1,
    allowFailures: false,
  });

  const { partialFee, weight } = await xcmTransfer.paymentInfo(alice.address);
  const transferFee: bigint = partialFee.toBigInt();
  const senderBalanceAfter = (
    (await senderParaApi.query.system.account(alice.address)) as AccountInfo
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
  const xcmFee: bigint =
    receiverBalanceBefore + amount - transferFee - receiverBalanceAfter;
  // between 0.02 ZTG and 0.10 ZTG XCM fee
  const approxXcmFeeLow = 200000000;
  const approxXcmFeeHigh = 1000000000;
  expect(xcmFee).toBeGreaterThanOrEqual(approxXcmFeeLow);
  expect(xcmFee).toBeLessThanOrEqual(approxXcmFeeHigh);
  expect(
    receiverBalanceAfter - receiverBalanceBefore,
    "Unexpected xcm transfer balance diff"
  ).toBe(amount - transferFee - xcmFee);
}
