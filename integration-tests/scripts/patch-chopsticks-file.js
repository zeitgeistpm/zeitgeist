#!/usr/bin/env node

/**
 * Injects a fallback TIMESTAMP_NOW entry into chopsticks' validation data
 * helper so that relay proofs contain the relay timestamp key required by
 * Zeitgeist's consensus hook.
 */

const fs = require('fs');
const path = require('path');

const [, , targetPath] = process.argv;

if (!targetPath) {
  console.error('Usage: patch-chopsticks-file.js <path-to-validation-data.js>');
  process.exit(1);
}

const absolutePath = path.resolve(targetPath);

if (!fs.existsSync(absolutePath)) {
  console.error(`Patch target does not exist: ${absolutePath}`);
  process.exit(1);
}

let source = fs.readFileSync(absolutePath, 'utf8');

if (source.includes('relayTimestampStep')) {
  console.log(`Already patched: ${absolutePath}`);
  process.exit(0);
}

const slotRegex = /(const slotIncrease =[\s\S]*?\.toNumber\(\);\n)/;
if (!slotRegex.test(source)) {
  console.error(`Unable to locate slotIncrease declaration in ${absolutePath}`);
  process.exit(1);
}

source = source.replace(
  slotRegex,
  (match) => `${match}        const relayTimestampStep = slotIncrease * 6000;\n`
);

const usesPrefixedKeys = source.includes('_proof.WELL_KNOWN_KEYS');
const keyExpr = `${usesPrefixedKeys ? '_proof.' : ''}WELL_KNOWN_KEYS`;

const usesPrefixedUtils = source.includes('(0, _util.hexToU8a)');
const hexCall = usesPrefixedUtils ? '(0, _util.hexToU8a)' : 'hexToU8a';
const toHexCall = usesPrefixedUtils ? '(0, _util.u8aToHex)' : 'u8aToHex';

const elseBlock =
  "            } else {\n" +
  "                newEntries.push([\n" +
  "                    key,\n" +
  "                    decoded[key]\n" +
  "                ]);\n" +
  "            }";

if (!source.includes(elseBlock)) {
  console.error(`Unable to locate relay key fallback block in ${absolutePath}`);
  process.exit(1);
}

const replacement =
  `            } else if (key === ${keyExpr}.TIMESTAMP_NOW) {\n` +
  `                const relayTimestamp = decoded[key] ? meta.registry.createType('Moment', ${hexCall}(decoded[key])).toNumber() : Date.now();\n` +
  "                const newTimestamp = meta.registry.createType('Moment', relayTimestamp + relayTimestampStep);\n" +
  "                newEntries.push([\n" +
  "                    key,\n" +
  `                    ${toHexCall}(newTimestamp.toU8a())\n` +
  "                ]);\n" +
  "            } else {\n" +
  "                newEntries.push([\n" +
  "                    key,\n" +
  "                    decoded[key]\n" +
  "                ]);\n" +
  "            }";

const updated = source.replace(elseBlock, replacement);

if (updated === source) {
  console.error(`Failed to patch ${absolutePath}`);
  process.exit(1);
}

fs.writeFileSync(absolutePath, updated);
console.log(`Patched ${absolutePath}`);
