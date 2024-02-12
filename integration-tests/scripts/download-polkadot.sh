# Copyright (C) Moondance Labs Ltd.
# Copyright 2022-2024 Forecasting Technologies LTD.
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

#!/bin/bash

# Exit on any error
set -e

# Check if the operating system is macOS
if [[ $(uname) == "Darwin" ]]; then
    echo "This script is not intended for MacOS, since the prebuild binaries are meant to be executed on Linux. But keep in mind you need to have 'polkadot', 'polkadot-execute-worker', 'polkadot-prepare-worker' in any case! So compile those yourself! Exiting..."
    exit 1
fi

# TODO: Use following line instead of the one above once Zeitgeist uses >=polkadot-v1.1.0
# Note: Version 1.4.0 of relaychain didn't allow the parachain to produce blocks
# branch=$(egrep -o '/polkadot.*#([^\"]*)' $(dirname $0)/../../Cargo.lock | head -1 | sed 's/.*release-//#')
# polkadot_release=$(echo $branch | sed 's/#.*//' | sed 's/\/polkadot-sdk?branch=polkadot-v//' | sed 's/-.*//')
polkadot_release="1.1.0"

# Always run the commands from the "integration-tests" dir
cd $(dirname $0)/..

if [[ -f tmp/polkadot ]]; then
	POLKADOT_VERSION=$(tmp/polkadot --version)
	if [[ $POLKADOT_VERSION == *$polkadot_release* ]]; then
		echo "Polkadot binary has correct version"
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