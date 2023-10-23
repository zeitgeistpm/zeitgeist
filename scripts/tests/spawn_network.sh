#!/usr/bin/env bash

if [ ! -d "./scripts/tests" ]; then
    echo "Please execute this script from the root of the Zeitgeist project folder"
    exit 1
fi;

cargo build --features parachain

export ADDITIONAL_ZOMBIECONFIG="${ADDITIONAL_ZOMBIECONFIG:-}"
export ZOMBIENET_CONFIG_FILE="${ZOMBIENET_CONFIG_FILE:-"./integration-tests/zombienet/produce-blocks.toml"}"
# Define destination path
ZOMBIENET_BINARY="./tmp/zombienet"

# Check if the file already exists
if [[ -f "${ZOMBIENET_BINARY}" ]]; then
    echo "zombienet already exists in /tmp. Executing it."
    $ZOMBIENET_BINARY spawn --provider native $ZOMBIENET_CONFIG_FILE $ADDITIONAL_ZOMBIECONFIG
    exit 0
fi

function download_zombienet {
    # Get the appropriate download link based on the OS
    case "$(uname -s)" in
        Linux)
            ARCHITECTURE="$(uname -m)"
            if [[ "${ARCHITECTURE}" == "x86_64" ]]; then
                FILE_NAME="zombienet-linux-x64"
            elif [[ "${ARCHITECTURE}" == "aarch64" ]]; then
                FILE_NAME="zombienet-linux-arm64"
            else
                echo "Unsupported architecture."
                exit 1
            fi
            ;;
        Darwin)
            FILE_NAME="zombienet-macos"
            ;;
        *)
            echo "Unsupported operating system."
            exit 1
            ;;
    esac

    # Fetch the latest release download URL from GitHub
    DOWNLOAD_URL=$(curl -s https://api.github.com/repos/paritytech/zombienet/releases/latest | \
                jq -r ".assets[] | select(.name == \"${FILE_NAME}\") | .browser_download_url")

    if [[ -z "${DOWNLOAD_URL}" ]]; then
        echo "Failed to retrieve download URL."
        exit 1
    fi

    mkdir -p ./tmp/

    # Download the file
    echo "Downloading ${FILE_NAME} from ${DOWNLOAD_URL}"
    curl -L "${DOWNLOAD_URL}" -o "${ZOMBIENET_BINARY}"

    # Provide feedback on the download status
    if [[ $? -eq 0 ]]; then
        echo "Download successful!"
    else
        echo "Download failed!"
        exit 1
    fi
}

if [ ! -f ${ZOMBIENET_BINARY} ]; then
    download_zombienet $ZOMBIENET_BINARY
fi

# Make the file executable
chmod +x "${ZOMBIENET_BINARY}"

$ZOMBIENET_BINARY spawn --provider native $ZOMBIENET_CONFIG_FILE $ADDITIONAL_ZOMBIECONFIG