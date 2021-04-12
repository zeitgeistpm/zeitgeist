#!/usr/bin/env bash

CENTRIFUGE_IMAGE="centrifugeio/rococo:chachacha-v1"
ZEITGEST_IMAGE="zeitgeistpm/zeitgeist-node-parachain"

CHACHACHA_VALIDATOR_0="chachacha-validator-0"
CHACHACHA_VALIDATOR_1="chachacha-validator-1"
ZEITGEST_PARACHAIN_0="zeitgeist-parachain-0"

sudo apt update
sudo apt install -y docker.io
sudo docker stop $CHACHACHA_VALIDATOR_0 $CHACHACHA_VALIDATOR_1 $ZEITGEST_PARACHAIN_0
sudo docker rm $CHACHACHA_VALIDATOR_0 $CHACHACHA_VALIDATOR_1 $ZEITGEST_PARACHAIN_0

sudo docker run \
    -d \
    --name $CHACHACHA_VALIDATOR_0 \
    --restart always \
    $CENTRIFUGE_IMAGE \
    /usr/local/bin/polkadot \
    --bootnodes /ip4/34.89.248.129/tcp/30333/p2p/12D3KooWD8CAZBgpeZiSVVbaj8mijR6mfgUsHNAmCKwsRoRnFod4 \
    --bootnodes /ip4/35.242.217.240/tcp/30333/p2p/12D3KooWBthdCz4JshkMb4GxJXVwrHPv9GpWAgfh2hAdkyXQDKyN \
    --chain rococo-chachacha \
    --name $CHACHACHA_VALIDATOR_0 \
    --validator

sudo docker run \
    -d \
    --name $CHACHACHA_VALIDATOR_1 \
    --restart always \
    $CENTRIFUGE_IMAGE \
    /usr/local/bin/polkadot \
    --bootnodes /ip4/34.89.248.129/tcp/30333/p2p/12D3KooWD8CAZBgpeZiSVVbaj8mijR6mfgUsHNAmCKwsRoRnFod4 \
    --bootnodes /ip4/35.242.217.240/tcp/30333/p2p/12D3KooWBthdCz4JshkMb4GxJXVwrHPv9GpWAgfh2hAdkyXQDKyN \
    --chain rococo-chachacha \
    --name $CHACHACHA_VALIDATOR_1 \
    --validator


sudo docker run --rm $ZEITGEST_IMAGE export-genesis-state --parachain-id 9123 > zeitgeist-genesis-state
sudo docker run --rm $ZEITGEST_IMAGE export-genesis-wasm > zeitgeist-genesis-wasm
curl -o rococo-chachacha.json https://storage.googleapis.com/centrifuge-artifact-releases/rococo-chachacha.json

sudo docker run \
    -d \
    --name $ZEITGEST_PARACHAIN_0 \
    --restart always \
    $ZEITGEST_IMAGE \
    --collator \
    --parachain-id 9123 \
    -- \
    --chain rococo-chachacha.json \
    --execution wasm
