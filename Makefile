run:
	SKIP_WASM_BUILD= cargo run -- --dev --execution=native -lruntime=debug

toolchain:
	./scripts/init.sh

build-wasm:
	WASM_BUILD_TYPE=release cargo build

check:
	SKIP_WASM_BUILD= cargo check --tests --all

check-dummy:
	BUILD_DUMMY_WASM_BINARY= cargo check

build:
	SKIP_WASM_BUILD= cargo build

purge:
	target/debug/zeitgeist purge-chain --dev -y

restart: purge run