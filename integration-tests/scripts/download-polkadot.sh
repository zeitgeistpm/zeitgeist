#!/bin/bash

# Exit on any error
set -e

# Check if the operating system is macOS
if [[ $(uname) == "Darwin" ]]; then
    echo "This script is not intended for MacOS, since the prebuild binaries are meant to be executed on Linux. But keep in mind you need to have 'polkadot', 'polkadot-execute-worker', 'polkadot-prepare-worker' in any case! So compile those yourself! Exiting..."
    exit 1
fi

# Version 1.4.0 of relaychain didn't allow the parachain to produce blocks
# that's why we use 1.1.0
polkadot_release="1.1.0"

# Always run the commands from the "test" dir
cd $(dirname $0)/..

if [[ -f tmp/polkadot ]]; then
	POLKADOT_VERSION=$(tmp/polkadot --version)
	if [[ $POLKADOT_VERSION == *$polkadot_release* ]]; then
		exit 0
	else
		echo "Updating polkadot binary..."

		pnpm moonwall download polkadot $polkadot_release tmp
		chmod +x tmp/polkadot

		pnpm moonwall download polkadot-execute-worker $polkadot_release tmp
		chmod +x tmp/polkadot-execute-worker

		pnpm moonwall download polkadot-prepare-worker $polkadot_release tmp
		chmod +x tmp/polkadot-prepare-worker

	fi
else
	echo "Polkadot binary not found, downloading..."
	pnpm moonwall download polkadot $polkadot_release tmp
	chmod +x tmp/polkadot

	pnpm moonwall download polkadot-execute-worker $polkadot_release tmp
	chmod +x tmp/polkadot-execute-worker

	pnpm moonwall download polkadot-prepare-worker $polkadot_release tmp
	chmod +x tmp/polkadot-prepare-worker
fi