<a href="https://zeitgeist.pm">
  <img src="./GH-banner.jpg">
</a>

# Zeitgeist: An Evolving Blockchain for Prediction Markets and Futarchy

![Rust](https://github.com/zeitgeistpm/zeitgeist/workflows/Rust/badge.svg)

<a href="https://t.me/zeitgeist_official">
  <img src="https://img.shields.io/badge/telegram-https%3A%2F%2Ft.me%2Fzeitgeist__official-blue" />
</a>

⠀Zeitgeist is a decentralized network for creating, betting on, and resolving
prediction markets. The platform's native currency, the ZTG,
is used to sway the direction of the network, and as a means of last-call dispute
resolution. Additionally, Zeitgeist is a protocol for efficient trading of prediction
market shares and will one day become the backbone of the decentralized finance ecosystem
by allowing for traders to create complex financial contracts on virtually _anything_.

## Content


- [Modules](#Modules)
- [Launching](#Launching)
  - [From source code](#From-source-code)
  - [Using a service file in Ubuntu](#Using-a-service-file-in-Ubuntu)
  - [Using Docker in Ubuntu](#Using-Docker-in-Ubuntu)
  - [Using Docker in other OS](#Using-Docker-in-other-OS)
- [Updating](#Updating)
  - [A service file in Ubuntu](#A-service-file-in-Ubuntu)
  - [Docker in Ubuntu](#Docker-in-Ubuntu)


## Modules


- [authorized](./zrml/authorized) - Offers authorized resolution of disputes.
- [court](./zrml/court) - An implementation of a court mechanism used to resolve
  disputes in a decentralized fashion.
- [liquidity-mining](./zrml/liquidity-mining) - This pallet implements the
  time-based incentivization with Zeitgeist tokens for continuously providing
  liquidity to swap pools.
- [market-commons](./zrml/market-commons) - Contains common operations on markets
  that are used by multiple pallets.
- [orderbook-v1](./zrml/orderbook-v1) - A naive orderbook implementation that's
  only part of Zeitgeist's PoC. Will be replaced by a v2 orderbook that uses 0x-style
  hybrid on-chain and off-chain trading.
- [prediction-markets](./zrml/prediction-markets) - The core implementation of the
  prediction market logic for creating and resolving markets.
- [simple-disputes](./zrml-simple-disputes) - Simple disputes selects the last dispute
  after a predetermined amount of disputes as the canonical outcome.
- [swaps](./zrml/swaps) - An implementation of liquidity pools that allows any user
  to provide liquidity to the pool or swap assets in and out of the pool. The market
  maker that is traded against is either a Constant Function Market Maker (CFMM) or
  a Rikiddo Market Maker.
- [primitives](./zrml/primitives) - Contains custom and common types, traits and constants.
- [rikiddo](./zrml/rikiddo) - The module contains a completely modular implementation
  of our novel market scoring rule [Rikiddo][rikiddo]. It also offer a pallet,
  that other pallets can use to utilize the Rikiddo market scoring rule. Rikiddo can
  be used by the automated market maker to determine swap prices.

## Launching

### From source code

⠀Zeitgeist node comes in two flavors, one for standalone self-contained execution
and another for Kusama/Polkadot parachain integration.

⠀To build the standalone version, simply point to the top directory of this project and type:

```bash
cargo build --release
```

⠀To build the parachain version, execute the following conmmand:

```
cargo build --features parachain --release
```

⠀Optimized binaries (`--release`) are usually used for production (faster and smaller), 
but this behavior is optional and up to you.

⠀Our current beta test network [Battery Station][zg-beta] runs as a parachain.
To connect your Zeitgeist parachain node, follow the tutorial at our [documentation site][bs-docs].

⠀Alternatively you can run a non-parachain node, which is usually only necessary for
testing purposes, by executing the following command:

```
cargo run --release --bin zeitgeist -- <node-options-and-flags>
```

⠀A common value for `<node-options-and-flags>` is `--dev --tmp`, which runs a 
local temporary development node.

### Using a service file in Ubuntu

⠀Update packages
```sh
sudo apt update && sudo apt upgrade -y
```

⠀Install dependencies
```sh
sudo apt install wget jq build-essential pkg-config libssl-dev -y
```

⠀Create a user
```sh
sudo useradd -M zeitgeist; \
sudo usermod zeitgeist -s /sbin/nologin
```

⠀Create a folder for the node
```sh
sudo mkdir -p /services/zeitgeist/bin /services/zeitgeist/battery_station
```

⠀Download appropriate files
```sh
cd; \
zeitgeist_version=`wget -qO- https://api.github.com/repos/zeitgeistpm/zeitgeist/releases/latest | jq -r ".tag_name"`; \
wget -qO /services/zeitgeist/bin/zeitgeist "https://github.com/zeitgeistpm/zeitgeist/releases/download/${zeitgeist_version}/zeitgeist_parachain"; \
wget -qO /services/zeitgeist/battery_station/battery-station-relay.json "https://raw.githubusercontent.com/zeitgeistpm/polkadot/battery-station-relay/node/service/res/battery-station-relay.json"; \
chmod +x /services/zeitgeist/bin/zeitgeist; \
cp /services/zeitgeist/bin/zeitgeist /usr/bin/
```

⠀Assign the folder owner
```sh
sudo chown -R zeitgeist:zeitgeist /services/zeitgeist
```

⠀Come up with a name for a node, run a command and enter a name, thereby adding it to the system as a variable
```sh
printf "\033[1;32mEnter the value:\033[0m "; \
read -r value; \
echo "export zeitgeist_moniker=\"${value}\"" >> $HOME/.bash_profile
```

⠀Open used ports
```sh
sudo ufw status

# If "Status: active"
sudo ufw allow 30333 9933 9944 30334 9934 9945

# If "Status: inactive" or an error
sudo iptables -I INPUT -p tcp --match multiport --dports 30333,9933,9944,30334,9934,9945 -j ACCEPT
sudo apt-get -y install iptables-persistent
sudo netfilter-persistent save
```

⠀Create a service file
```sh
sudo tee <<EOF >/dev/null /etc/systemd/system/zeitgeistd.service
[Unit]
Description=Zeitgeist node
After=network.target
Requires=network.target

[Service]
User=zeitgeist
Group=zeitgeist
Restart=on-failure
RestartSec=3
LimitNOFILE=65535
ExecStart=/services/zeitgeist/bin/zeitgeist \\
    --base-path /services/zeitgeist/battery_station \\
    --chain battery_station \\
    --name "$zeitgeist_moniker" \\
    --parachain-id 2050 \\
    --port 30333 \\
    --rpc-port 9933 \\
    --ws-port 9944 \\
    --rpc-external \\
    --ws-external \\
    --rpc-cors all \\
    --pruning archive \\
    -- \\
    --port 30334 \\
    --rpc-port 9934 \\
    --ws-port 9945

[Install]
WantedBy=multi-user.target
EOF
```

⠀Run the service file
```sh
sudo systemctl daemon-reload; \
sudo systemctl enable zeitgeistd; \
sudo systemctl restart zeitgeistd
```

⠀Add a command to view the log of a node in the system as a variable
```sh
echo "alias zeitgeist_log=\"sudo journalctl -f -n 100 -u zeitgeistd\"" >> $HOME/.bash_profile
```

### :exclamation::exclamation::exclamation:

⠀Save the file in a safe place (the command displays the path)
```sh
echo /services/zeitgeist/battery_station/chains/battery_station_mainnet/network/secret_ed25519
```

### :exclamation::exclamation::exclamation:

### Using Docker in Ubuntu

⠀Install Docker
```sh
sudo apt update && \
sudo apt upgrade -y; \
sudo apt install curl apt-transport-https ca-certificates gnupg lsb-release -y; \
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg; \
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu `lsb_release -cs` stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null; \
sudo apt update; \
sudo apt install docker-ce docker-ce-cli containerd.io -y; \
docker_version=`apt-cache madison docker-ce | grep -oPm1 "(?<=docker-ce \| )([^_]+)(?= \| https)"`; \
sudo apt install docker-ce="$docker_version" docker-ce-cli="$docker_version" containerd.io -y
```

⠀Create a folder for node
```sh
mkdir $HOME/zeitgeist
```

⠀Download appropriate files
```sh
zeitgeist_version=`wget -qO- https://api.github.com/repos/zeitgeistpm/zeitgeist/releases/latest | jq -r ".tag_name"`; \
wget -qO $HOME/zeitgeist/battery-station-relay.json "https://github.com/zeitgeistpm/zeitgeist/releases/download/${zeitgeist_version}/battery-station-relay.json"
```

⠀Come up with a name for a node, run a command and enter a name, thereby adding it to the system as a variable
```sh
printf "\033[1;32mEnter the value:\033[0m "; \
read -r value; \
echo "export zeitgeist_moniker=\"${value}\"" >> $HOME/.bash_profile
```

⠀Open used ports
```sh
sudo ufw status

# If "Status: active"
sudo ufw allow 30333 9933 9944 30334 9934 9945

# If "Status: inactive" or an error
sudo iptables -I INPUT -p tcp --match multiport --dports 30333,9933,9944,30334,9934,9945 -j ACCEPT
sudo apt-get -y install iptables-persistent
sudo netfilter-persistent save
```

⠀Add a command to view the log of a node in the system as a variable
```sh
echo "alias zeitgeist_log=\"docker logs zeitgeist_node -fn 100\"" >> $HOME/.bash_profile
```

⠀Run container
```sh
docker run -dit \
    --name zeitgeist_node \
    --restart always \
    -p 30333:30333 \
    -p 9933:9933 \
    -p 9944:9944 \
    zeitgeistpm/zeitgeist-node-parachain \
    --base-path /zeitgeist/data \
    --chain battery_station \
    --name "$zeitgeist_moniker" \
    --pruning archive
```

### :exclamation::exclamation::exclamation:

⠀Save the file in a safe place (the command displays the path)
```sh
echo /services/zeitgeist/battery_station/chains/battery_station_mainnet/network/secret_ed25519
```

### :exclamation::exclamation::exclamation:

### Using Docker in other OS

⠀We publish the latest standalone and parachain version to the [Docker Hub][zg-docker-hub], 
from where it can be pulled and ran locally to connect to the network with relatively
low effort and high compatibility. In order to fetch the latest docker image,
ensure you have Docker installed locally, then type (or paste) the following
commands in your terminal.

⠀For parachain Zeitgeist node:
```sh
docker pull zeitgeistpm/zeitgeist-node-parachain
```

⠀For standalone, non-parachain Zeitgeist node:
```sh
docker pull zeitgeistpm/zeitgeist-node
```

⠀Our current beta test network [Battery Station][zg-beta] runs as a parachain.
To connect your Zeitgeist parachain node, follow the tutorial at our [documentation site][bs-docs].

⠀Alternatively you can run a non-parachain node, which is usually only necessary for
testing purposes, by executing the following command:

```sh
docker run zeitgeistpm/zeitgeist-node -- <node-options-and-flags>
```

## Updating

### A service file in Ubuntu

⠀Stop a service file
```sh
systemctl stop zeitgeistd
```

⠀Download the latest version of binary
```sh
cd; \
zeitgeist_version=`wget -qO- https://api.github.com/repos/zeitgeistpm/zeitgeist/releases/latest | jq -r ".tag_name"`; \
wget -qO /services/zeitgeist/bin/zeitgeist "https://github.com/zeitgeistpm/zeitgeist/releases/download/${zeitgeist_version}/zeitgeist_parachain"; \
wget -qO /services/zeitgeist/battery_station/battery-station-relay.json "https://raw.githubusercontent.com/zeitgeistpm/polkadot/battery-station-relay/node/service/res/battery-station-relay.json"; \
chmod +x /services/zeitgeist/bin/zeitgeist; \
cp /services/zeitgeist/bin/zeitgeist /usr/bin/
```

⠀Give permission to execute it
```sh
sudo chown -R zeitgeist:zeitgeist /services/zeitgeist
```

⠀Restart the service file
```sh
systemctl restart zeitgeistd
```

⠀Check the logs
```sh
zeitgeist_log
```

### Docker in Ubuntu

⠀Update an image
```sh
docker pull zeitgeistpm/zeitgeist-node-parachain:latest
```

⠀Stop the node
```sh
docker stop zeitgeist_node
```

⠀Delete the node
```sh
docker rm zeitgeist_node
```

⠀Create a new container
```sh
docker run -dit \
    --name zeitgeist_node \
    --restart always \
    -p 30333:30333 \
    -p 9933:9933 \
    -p 9944:9944 \
    -v $HOME/zeitgeist/secret_ed25519:/zeitgeist/data/secret_ed25519 \
    zeitgeistpm/zeitgeist-node-parachain:latest \
    --base-path /zeitgeist/data \
    --node-key-file /zeitgeist/data/secret_ed25519 \
    --chain battery_station \
    --name "$zeitgeist_moniker" \
    --pruning archive
```


[bs-docs]: https://docs.zeitgeist.pm/battery-station
[ls-lmsr]: https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/OPRS10.pdf
[rikiddo]: https://blog.zeitgeist.pm/introducing-zeitgeists-rikiddo-scoring-rule/
[zg-beta]: https://blog.zeitgeist.pm/zeitgeist-beta/
[zg-docker-hub]: https://hub.docker.com/r/zeitgeistpm/zeitgeist-node
