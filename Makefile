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
		--execution=Native \
		--chain=${TRYRUNTIME_CHAIN} \
		on-runtime-upgrade \
		live \
		--uri=${TRYRUNTIME_URL}

try-runtime-upgrade-battery-station:
	@$(MAKE) TRYRUNTIME_CHAIN="battery_station_staging" TRYRUNTIME_URL="wss://bsr.zeitgeist.pm:443" -- --try-runtime

try-runtime-upgrade-zeitgeist:
	@$(MAKE) TRYRUNTIME_CHAIN="zeitgeist_staging" TRYRUNTIME_URL="wss://zeitgeist-rpc.dwellir.com:443" -- --try-runtime

build:
	SKIP_WASM_BUILD= cargo build

purge:
	target/debug/zeitgeist purge-chain --dev -y

restart: purge run
