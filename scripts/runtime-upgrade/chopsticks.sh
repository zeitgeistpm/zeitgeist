#!/usr/bin/env bash

set -euxo pipefail
usage() {
    cat <<EOF
Invoke chopsticks to fork the network at latest best block.

usage: chopsticks.sh [options]

Options:
    -h, --help                                   Shows this dialogue
    --bs                                         Fork battery_station network. 
    --override_wasm=<PATH_TO_RUNTIME_WASM_FILE>  Use wasm file from the path and apply it on forked network.
EOF
}

endpoit="wss://zeitgeist-rpc.dwellir.com:443"
override_wasm_from_path=""

for arg in "$@"; do
    case $arg in
    "--help" | "-h")
        usage
        exit 0
        ;;
    "--bs")
    endpoint="wss://bsr.zeitgeist.pm:443"
        ;;
    --override_wasm=*)
        override_wasm_from_path=--wasm-override="${arg#*=}"
        ;;
    *)
        echo "Unknown option '$arg'"
        usage
        exit 1
        ;;
    esac
done

# NOTE: For now we are not using yarn.
# currentver="$(yarn --version | head -n1 | cut -d"." -f1)"
# requiredver=2

# if [ $currentver -lt $requiredver ]; then 
#         echo "Atleast require 2.0 version of Yarn."
#         exit 1
# fi

npx @acala-network/chopsticks@0.2.3 dev --endpoint=$endpoit --db=./temp_db.sqlite --port=8080 $override_wasm_from_path


