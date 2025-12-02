#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later

# Exit on any error
set -euo pipefail

# Always run the commands from the "integration-tests" dir
cd $(dirname $0)/..

mkdir -p specs
# Start from the dev plain spec so zombienet can customize it. Avoid parsing
# and re-stringifying the whole JSON (which can switch large integers into
# exponent form) by doing targeted string replacements.
tmp_spec="$(mktemp)"
../target/release/zeitgeist build-spec --chain=dev --disable-default-bootnode > "${tmp_spec}"

out_path="$(pwd)/zeitgeist-parachain-2092.json"

node - "${tmp_spec}" "${out_path}" <<'NODE'
const fs = require("fs");
const path = require("path");
const [,, inPath, outPath] = process.argv;
if (!inPath || !outPath) {
  console.error("spec path arg missing");
  process.exit(1);
}

let text = fs.readFileSync(inPath, "utf8");
const replace = (pattern, replacement, label) => {
  const next = text.replace(pattern, replacement);
  if (next === text) {
    console.error(`WARN: pattern not replaced (${label})`);
  }
  text = next;
};

replace(/"name"\s*:\s*"[^"]*"/, '"name": "Zeitgeist Rococo Local"', "name");
replace(/"id"\s*:\s*"[^"]*"/, '"id": "zeitgeist_rococo_local"', "id");
replace(/"relay_chain"\s*:\s*"[^"]*"/, '"relay_chain": "rococo-local"', "relay_chain");
replace(/"parachain_id"\s*:\s*\d+/, '"parachain_id": 2092', "parachain_id");
replace(/"chainType"\s*:\s*"[^"]*"/, '"chainType": "Local"', "chainType");
replace(/"bootNodes"\s*:\s*\[[\s\S]*?\]/, '"bootNodes": []', "bootNodes");
replace(/"telemetryEndpoints"\s*:\s*(\[[\s\S]*?\]|null)/, '"telemetryEndpoints": null', "telemetryEndpoints");
replace(/"protocolId"\s*:\s*(null|"[^"]*")/, '"protocolId": "zeitgeist-rococo"', "protocolId");
replace(/"ss58Format"\s*:\s*\d+/, '"ss58Format": 73', "ss58Format");
replace(/"tokenDecimals"\s*:\s*[^,}\n]+/, '"tokenDecimals": 10', "tokenDecimals");
replace(/"tokenSymbol"\s*:\s*"[^"]*"/, '"tokenSymbol": "ZTG"', "tokenSymbol");
replace(/("parachainInfo"\s*:\s*\{\s*"parachainId"\s*:\s*)\d+/, "$1 2092", "parachainInfo.parachainId");

fs.writeFileSync(outPath, text);
console.log(`Wrote ${outPath}`);

// Keep the runtime wasm that Moonwall will upload in sync with the code baked
// into the genesis so the upgrade test compares against the correct artifact.
try {
  const spec = JSON.parse(text);
  const codeHex = spec?.genesis?.runtimeGenesis?.code || spec?.genesis?.runtime?.code;
  if (typeof codeHex === "string" && codeHex.startsWith("0x")) {
    const wasmPath = path.resolve(process.cwd(), "../target/release/wbuild/zeitgeist-runtime/zeitgeist_runtime.compact.compressed.wasm");
    fs.mkdirSync(path.dirname(wasmPath), { recursive: true });
    fs.writeFileSync(wasmPath, Buffer.from(codeHex.slice(2), "hex"));
    console.log(`Wrote runtime wasm to ${wasmPath}`);
  } else {
    console.error("WARN: runtime code not found in spec; skipping wasm write");
  }
} catch (err) {
  console.error("WARN: failed to parse spec JSON to write wasm", err?.message || err);
}
NODE
