#!/usr/bin/env bash

# Copyright (C) Moondance Labs Ltd.
# Copyright 2022-2025 Forecasting Technologies LTD.
#
# This file is part of Zeitgeist.
#
# Zeitgeist is free software: you can redistribute it and/or modify it
# under the terms of the GNU General Public License as published by the
# Free Software Foundation, either version 3 of the License, or (at
# your option) any later version.
#
# Zeitgeist is distributed in the hope that it will be useful, but
# WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
# General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

# Exit on any error
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INTEGRATION_DIR="${SCRIPT_DIR}/.."
REPO_ROOT="${INTEGRATION_DIR}/.."
CARGO_LOCK="${REPO_ROOT}/Cargo.lock"
TMP_DIR="${INTEGRATION_DIR}/tmp"
GITHUB_REPO="https://github.com/paritytech/polkadot-sdk"

# Always run the commands from the "integration-tests" dir
cd "${INTEGRATION_DIR}"

IS_MACOS=0
if [[ "$(uname)" == "Darwin" ]]; then
	IS_MACOS=1
fi

mkdir -p "${TMP_DIR}"

POLKADOT_SOURCE_LINE=""
POLKADOT_SOURCE_COMMIT=""

get_polkadot_source_line() {
	if [[ -n "${POLKADOT_SOURCE_LINE}" || ! -f "${CARGO_LOCK}" ]]; then
		printf "%s" "${POLKADOT_SOURCE_LINE}"
		return
	fi

	POLKADOT_SOURCE_LINE=$(grep -m1 'git+https://github.com/paritytech/polkadot-sdk' "${CARGO_LOCK}" || true)
	printf "%s" "${POLKADOT_SOURCE_LINE}"
}

determine_release() {
	local release="${POLKADOT_RELEASE_OVERRIDE:-}"
	if [[ -n "${release}" ]]; then
		printf "%s" "${release}"
		return
	fi

	local lock_line
	lock_line=$(get_polkadot_source_line)
	if [[ -n "${lock_line}" ]]; then
		if [[ "${lock_line}" =~ tag=([^&#\"]+) ]]; then
			printf "%s" "${BASH_REMATCH[1]}"
			if [[ "${lock_line}" =~ \#([0-9a-f]+)\" ]]; then
				POLKADOT_SOURCE_COMMIT="${BASH_REMATCH[1]}"
			fi
			return
		elif [[ "${lock_line}" =~ branch=([^&#\"]+) ]]; then
			printf "%s" "${BASH_REMATCH[1]}"
			if [[ "${lock_line}" =~ \#([0-9a-f]+)\" ]]; then
				POLKADOT_SOURCE_COMMIT="${BASH_REMATCH[1]}"
			fi
			return
		elif [[ "${lock_line}" =~ \#([0-9a-f]+)\" ]]; then
			# Fallback to commit-only references.
			POLKADOT_SOURCE_COMMIT="${BASH_REMATCH[1]}"
		fi
	fi

	printf "latest"
}

build_from_source_macos() {
	local release="$1"
	local commit="$2"
	local repo_dir="${TMP_DIR}/polkadot-sdk-src"

	if [[ "${release}" == "latest" ]]; then
		echo "Unable to determine Polkadot release for macOS builds. Set POLKADOT_RELEASE_OVERRIDE to a concrete tag or branch."
		exit 1
	fi

	if [[ ! -d "${repo_dir}/.git" ]]; then
		echo "Cloning Polkadot SDK into ${repo_dir}"
		git clone "${GITHUB_REPO}" "${repo_dir}"
	fi

	(
		cd "${repo_dir}"
		# Fetch only from origin to avoid hitting private remotes like "benchmarks".
		git fetch origin --tags --prune
		local target="${commit:-${release}}"
		echo "Checking out ${target}"
		git checkout "${target}"
		echo "Building Polkadot binaries from source (this may take a while)..."
		cargo build --locked --release -p polkadot
	)

	for bin in polkadot polkadot-execute-worker polkadot-prepare-worker; do
		local source_binary="${repo_dir}/target/release/${bin}"
		if [[ ! -x "${source_binary}" ]]; then
			echo "Failed to build ${bin} at ${source_binary}"
			exit 1
		fi
		cp "${source_binary}" "tmp/${bin}"
		chmod +x "tmp/${bin}"
	done
}

delete_if_not_binary() {
	local file_path="$1"
	if [[ -f "${file_path}" ]] && ! file "${file_path}" | grep -q 'executable'; then
		rm -f "${file_path}"
	fi
}

run_moonwall_download() {
	local name="$1"
	local version="$2"
	local log_file
	log_file=$(mktemp)

	if pnpm moonwall download "${name}" "${version}" tmp 2>&1 | tee "${log_file}"; then
		rm -f "${log_file}"
		chmod +x "tmp/${name}"
		return 0
	fi

	LAST_ERROR_LOG=$(cat "${log_file}")
	rm -f "${log_file}"
	return 1
}

download_from_github() {
	local name="$1"
	local version="$2"
	local url="${GITHUB_REPO}/releases/download/${version}/${name}"
	local temp_file="${TMP_DIR}/${name}.download"

	echo "Downloading ${name} ${version} from ${url}"
	if curl --fail --location --progress-bar -o "${temp_file}" "${url}"; then
		mv "${temp_file}" "tmp/${name}"
		chmod +x "tmp/${name}"
		return 0
	fi

	rm -f "${temp_file}"
	return 1
}

download_binary() {
	local name="$1"
	local version="$2"
	if [[ "${version}" != "latest" ]]; then
		if download_from_github "${name}" "${version}"; then
			return
		fi

		echo "Failed to download ${name} ${version} from ${GITHUB_REPO}."
		exit 1
	fi

	if run_moonwall_download "${name}" "${version}"; then
		return
	fi

	echo "Failed to download ${name}:"
	echo "${LAST_ERROR_LOG}"
	exit 1
}

POLKADOT_RELEASE="$(determine_release)"
echo "Using Polkadot release \"${POLKADOT_RELEASE}\""

delete_if_not_binary "tmp/polkadot"
delete_if_not_binary "tmp/polkadot-execute-worker"
delete_if_not_binary "tmp/polkadot-prepare-worker"

if [[ -x tmp/polkadot && -x tmp/polkadot-execute-worker && -x tmp/polkadot-prepare-worker ]]; then
	POLKADOT_VERSION=$(tmp/polkadot --version || true)
	if [[ "${POLKADOT_RELEASE}" == "latest" || "${POLKADOT_VERSION}" == *"${POLKADOT_RELEASE}"* ]]; then
		echo "Polkadot binaries already match the requested release."
		exit 0
	else
		echo "Updating polkadot binary from \"${POLKADOT_VERSION}\" to \"${POLKADOT_RELEASE}\"..."
	fi
else
	echo "Polkadot binary not found, downloading..."
fi

if [[ "${IS_MACOS}" -eq 1 ]]; then
	build_from_source_macos "${POLKADOT_RELEASE}" "${POLKADOT_SOURCE_COMMIT}"
else
	download_binary "polkadot" "${POLKADOT_RELEASE}"
	download_binary "polkadot-execute-worker" "${POLKADOT_RELEASE}"
	download_binary "polkadot-prepare-worker" "${POLKADOT_RELEASE}"
fi

echo "Polkadot binaries downloaded to ${TMP_DIR}"
