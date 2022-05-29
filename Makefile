.PHONY: run
run:
	SKIP_WASM_BUILD= cargo run -- --dev --execution=Native -lruntime=debug

.PHONY: toolchain
toolchain:
	./scripts/init.sh

.PHONY: build-wasm
build-wasm:
	WASM_BUILD_TYPE=release cargo build

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --tests --all

.PHONY: check-dummy
check-dummy:
	BUILD_DUMMY_WASM_BINARY= cargo check

.PHONY: build 
build:
	SKIP_WASM_BUILD= cargo build

.PHONY: purge
purge:
	target/debug/zeitgeist purge-chain --dev -y

.PHONY: restart
restart: purge run
