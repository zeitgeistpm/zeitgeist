.PHONY: $(MAKECMDGOALS)

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

# Pseudo private target is invoked by public targets for different chains
--execute-try-runtime:
	RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=info \
	cargo build --release --features=parachain,try-runtime,force-debug
	try-runtime \
	--runtime=${RUNTIME_PATH} \
	on-runtime-upgrade \
	--checks=all \
	live \
	--uri=${TRYRUNTIME_URL}

try-runtime-upgrade-battery-station:
	@$(MAKE) TRYRUNTIME_URL="wss://bsr.zeitgeist.pm:443" \
	RUNTIME_PATH="./target/release/wbuild/battery-station-runtime/battery_station_runtime.compact.compressed.wasm" \
	-- \
	--execute-try-runtime

try-runtime-upgrade-zeitgeist:
	@$(MAKE) TRYRUNTIME_URL="wss://zeitgeist.api.onfinality.io:443/public-ws" \
	RUNTIME_PATH="./target/release/wbuild/zeitgeist-runtime/zeitgeist_runtime.compact.compressed.wasm" \
	-- \
	--execute-try-runtime

build:
	SKIP_WASM_BUILD= cargo build

build-production:
	SKIP_WASM_BUILD= cargo build --profile=production

purge:
	target/debug/zeitgeist purge-chain --dev -y

restart: purge run
