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
--try-runtime:
	RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
		cargo run \
		--bin=zeitgeist \
		--features=parachain,try-runtime \
		try-runtime \
		--chain=${TRYRUNTIME_CHAIN} \
		--runtime=${RUNTIME_PATH} \
		on-runtime-upgrade \
		--checks=all \
		live \
		--uri=${TRYRUNTIME_URL}

try-runtime-upgrade-battery-station:
	@$(MAKE) TRYRUNTIME_CHAIN="battery_station_staging" \
		TRYRUNTIME_URL="wss://bsr.zeitgeist.pm:443" \
		RUNTIME_PATH="./target/debug/wbuild/battery-station-runtime/battery_station_runtime.compact.compressed.wasm" \
		-- \
		--try-runtime

try-runtime-upgrade-zeitgeist:
	@$(MAKE) TRYRUNTIME_CHAIN="zeitgeist_staging" \
		TRYRUNTIME_URL="wss://zeitgeist-rpc.dwellir.com:443" \
		RUNTIME_PATH="./target/debug/wbuild/zeitgeist-runtime/zeitgeist_runtime.compact.compressed.wasm" \
		-- \
		--try-runtime

build:
	SKIP_WASM_BUILD= cargo build

build-production:
	SKIP_WASM_BUILD= cargo build --profile=production

purge:
	target/debug/zeitgeist purge-chain --dev -y

restart: purge run
