/* eslint-disable @typescript-eslint/no-var-requires */
/**
 * Monkey patch chopsticks so that every installed copy of
 * @acala-network/chopsticks-core injects TIMESTAMP_NOW into relay proofs.
 */

const fs = require('fs');
const path = require('path');

const patchedExecutors = new Set();
const FALLBACK_TIMESTAMP_KEY =
  '0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb';
const RELAY_BLOCK_TIME_MS = 6000;

const hexToU8a = (hex) => {
  if (!hex) {
    return new Uint8Array();
  }
  const normalized = hex.startsWith('0x') ? hex.slice(2) : hex;
  const padded = normalized.padEnd(Math.ceil(normalized.length / 2) * 2, '0');
  const bytes = new Uint8Array(padded.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    const chunk = padded.slice(i * 2, i * 2 + 2);
    bytes[i] = parseInt(chunk, 16);
  }
  return bytes;
};

const u8aToHex = (bytes) => {
  return (
    '0x' +
    Array.from(bytes)
      .map((byte) => byte.toString(16).padStart(2, '0'))
      .join('')
  );
};

const encodeMoment = (value) => {
  const bytes = new Uint8Array(8);
  let remainder = BigInt(value);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = Number(remainder & 0xffn);
    remainder >>= 8n;
  }
  return u8aToHex(bytes);
};

const decodeMoment = (hex) => {
  if (!hex) {
    return undefined;
  }
  const bytes = hexToU8a(hex);
  let result = 0n;
  const len = Math.min(bytes.length, 8);
  for (let i = 0; i < len; i++) {
    result |= BigInt(bytes[i]) << BigInt(i * 8);
  }
  return Number(result);
};

const patchExecutor = (moduleRoot) => {
  const proofPath = path.join(moduleRoot, 'dist', 'cjs', 'utils', 'proof.js');
  const wasmPath = path.join(moduleRoot, 'dist', 'cjs', 'wasm-executor', 'index.js');

  if (!fs.existsSync(proofPath) || !fs.existsSync(wasmPath)) {
    return;
  }

  if (patchedExecutors.has(wasmPath)) {
    return;
  }

  try {
    const proofModule = require(proofPath);
    const wasmExecutor = require(wasmPath);
    const timestampKey = proofModule?.WELL_KNOWN_KEYS?.TIMESTAMP_NOW || FALLBACK_TIMESTAMP_KEY;
    if (!timestampKey || !wasmExecutor?.decodeProof) {
      console.warn(
        `[chopsticks-relay-timestamp-hook] Missing timestamp key or decodeProof in ${moduleRoot}`
      );
      return;
    }

    let relayTimestamp = Date.now();
    const originalDecodeProof = wasmExecutor.decodeProof.bind(wasmExecutor);

    wasmExecutor.decodeProof = async (...args) => {
      const decoded = await originalDecodeProof(...args);
      const existing = decodeMoment(decoded[timestampKey]);
      if (typeof existing === 'number' && Number.isFinite(existing)) {
        relayTimestamp = existing + RELAY_BLOCK_TIME_MS;
      } else {
        relayTimestamp += RELAY_BLOCK_TIME_MS;
      }
      decoded[timestampKey] = encodeMoment(relayTimestamp);
      return decoded;
    };

    patchedExecutors.add(wasmPath);
    console.log(`[chopsticks-relay-timestamp-hook] Patched ${moduleRoot}`);
  } catch (error) {
    console.warn(`[chopsticks-relay-timestamp-hook] Failed to patch ${moduleRoot}:`, error);
  }
};

const projectRoot = path.resolve(__dirname, '..');

// Patch hoisted dependency (if present).
try {
  const resolvedProof = require.resolve('@acala-network/chopsticks-core/dist/cjs/utils/proof.js', {
    paths: [projectRoot],
  });
  const moduleRoot = path.resolve(resolvedProof, '..', '..', '..', '..');
  patchExecutor(moduleRoot);
} catch {
  // ignore – package might only exist in .pnpm
}

// Patch every copy living under .pnpm (covers nested dependencies such as @moonwall/cli).
const pnpmDir = path.join(projectRoot, 'node_modules', '.pnpm');
if (fs.existsSync(pnpmDir)) {
  for (const entry of fs.readdirSync(pnpmDir)) {
    if (!entry.startsWith('@acala-network+chopsticks-core@')) {
      continue;
    }
    const moduleRoot = path.join(
      pnpmDir,
      entry,
      'node_modules',
      '@acala-network',
      'chopsticks-core'
    );
    patchExecutor(moduleRoot);
  }
}
