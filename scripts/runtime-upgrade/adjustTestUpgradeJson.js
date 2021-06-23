const fs = require("fs");

const raw = fs.readFileSync("./test-upgrade.json", { encoding: "utf-8" });
const json = JSON.parse(raw);
json.name = "Zeitgeist Battery Park Runtime Upgrade Test";
json.id = "battery_park_runtime_upgrade_test";
json.bootNodes = [];
json.protocolId = json.id;
json.genesis.raw.top["0x57f8dc2f5ab09467896f47300f0424385e0621c4869aa60c02be9adcc98a0d1d"] = "0x04d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
json.genesis.raw.top["0x5c0d1176a568c1f92944340dbfed9e9c530ebca703c85910e7164cb7d1c9e47b"] = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

fs.writeFileSync("./test-upgrade.json", JSON.stringify(json));
