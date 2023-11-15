"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
exports.__esModule = true;
exports.run = void 0;
var api_1 = require("@polkadot/api");
var keyring_1 = require("@polkadot/keyring");
var util_crypto_1 = require("@polkadot/util-crypto");
// Addresses for Alice and Bob on the dev chain
var ALICE = '//Alice';
var BOB = '//Bob';
var run = function (nodeName, networkInfo, args) { return __awaiter(void 0, void 0, void 0, function () {
    var provider, api, keyring, alice, bob, aliceFreeBalanceBefore, bobFreeBalanceBefore, transfer, hash, aliceFreeBalanceAfter, bobFreeBalanceAfter, aliceLostAmount, bobGainedAmount;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0:
                provider = new api_1.WsProvider('ws://127.0.0.1:9944');
                return [4 /*yield*/, api_1.ApiPromise.create({ provider: provider })];
            case 1:
                api = _a.sent();
                // Wait for the crypto library to be ready
                return [4 /*yield*/, (0, util_crypto_1.cryptoWaitReady)()];
            case 2:
                // Wait for the crypto library to be ready
                _a.sent();
                keyring = new keyring_1.Keyring({ type: 'sr25519' });
                alice = keyring.addFromUri(ALICE);
                bob = keyring.addFromUri(BOB);
                return [4 /*yield*/, api.query.system.account(alice.address)];
            case 3:
                aliceFreeBalanceBefore = (_a.sent());
                return [4 /*yield*/, api.query.system.account(bob.address)];
            case 4:
                bobFreeBalanceBefore = (_a.sent());
                console.log("Alice has ".concat(aliceFreeBalanceBefore));
                console.log("Bob has ".concat(bobFreeBalanceBefore));
                transfer = api.tx.balances.transfer(bob.address, 500000);
                return [4 /*yield*/, transfer.signAndSend(alice)];
            case 5:
                hash = _a.sent();
                console.log("Transfer sent with hash ".concat(hash));
                return [4 /*yield*/, api.query.system.account(alice.address)];
            case 6:
                aliceFreeBalanceAfter = (_a.sent());
                return [4 /*yield*/, api.query.system.account(bob.address)];
            case 7:
                bobFreeBalanceAfter = (_a.sent());
                aliceLostAmount = aliceFreeBalanceBefore.data.free.sub(aliceFreeBalanceAfter.data.free);
                bobGainedAmount = bobFreeBalanceAfter.data.free.sub(bobFreeBalanceBefore.data.free);
                console.log("Alice lost ".concat(aliceLostAmount, " tokens"));
                console.log("Bob gained ".concat(bobGainedAmount, " tokens"));
                return [2 /*return*/, aliceLostAmount.eq(bobGainedAmount) ? 1 : 0];
        }
    });
}); };
exports.run = run;
