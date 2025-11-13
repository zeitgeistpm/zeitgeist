#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

PATCH_TARGETS=()
if [ -d "${PROJECT_ROOT}/node_modules" ]; then
    while IFS= read -r file; do
        PATCH_TARGETS+=("$file")
    done < <(find "${PROJECT_ROOT}/node_modules" -path '*@acala-network/chopsticks-core/dist/*/blockchain/inherent/parachain/validation-data.js' 2>/dev/null || true)
fi

if [ ${#PATCH_TARGETS[@]} -eq 0 ]; then
    echo "[patch-chopsticks] No chopsticks-core validation-data files found. Skipping."
    exit 0
fi

PATCH_SCRIPT="${SCRIPT_DIR}/patch-chopsticks-file.js"
if [ ! -x "${PATCH_SCRIPT}" ]; then
    chmod +x "${PATCH_SCRIPT}"
fi

for file in "${PATCH_TARGETS[@]}"; do
    echo "[patch-chopsticks] Patching ${file}"
    node "${PATCH_SCRIPT}" "${file}"
done
