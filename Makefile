run:
	SKIP_WASM_BUILD= cargo run -- --dev --execution=Native -lruntime=debug

toolchain:
	./scripts/init.sh

build-wasm:
	WASM_BUILD_TYPE=release cargo build

check:
	SKIP_WASM_BUILD= cargo check --tests --all

check-dummy:
	BUILD_DUMMY_WASM_BINARY= cargo check

.PHONY: try-runtime-upgrade
try-runtime-upgrade:
	cargo run --release --bin zeitgeist --features parachain --features=try-runtime try-runtime on-runtime-upgrade live --uri wss://bsr.zeitgeist.pm:443

build:
	SKIP_WASM_BUILD= cargo build

purge:
	target/debug/zeitgeist purge-chain --dev -y

restart: purge run