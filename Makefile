default: build

test: build
	cargo test --all --tests

build:
	cargo rustc --manifest-path=bridge-oracle/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	cargo rustc --manifest-path=dao-utils/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	cargo rustc --manifest-path=treasury/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release

ifeq ($(OS),Windows_NT)
	if not exist target\wasm32-unknown-unknown\optimized mkdir target\wasm32-unknown-unknown\optimized
else
	mkdir -p target/wasm32-unknown-unknown/optimized
endif

	stellar contract optimize \
		--wasm target/wasm32-unknown-unknown/release/bridge_oracle.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/bridge_oracle.wasm
	stellar contract optimize \
		--wasm target/wasm32-unknown-unknown/release/dao_utils.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/dao_utils.wasm
	stellar contract optimize \
		--wasm target/wasm32-unknown-unknown/release/treasury.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/treasury.wasm

ifeq ($(OS),Windows_NT)
	cd target/wasm32-unknown-unknown/optimized/ && for %%i in (*.wasm) do dir "%%i"
else
	cd target/wasm32-unknown-unknown/optimized/ && for i in *.wasm ; do ls -l "$$i"; done
endif

fmt:
	cargo fmt --all

clean:
	cargo clean