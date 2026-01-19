default: build

test: build
	cargo test --all --tests

build:
	cargo rustc --manifest-path=bridge-oracle/Cargo.toml --crate-type=cdylib --target=wasm32v1-none --release
	cargo rustc --manifest-path=dao-utils/Cargo.toml --crate-type=cdylib --target=wasm32v1-none --release
	cargo rustc --manifest-path=treasury/Cargo.toml --crate-type=cdylib --target=wasm32v1-none --release

ifeq ($(OS),Windows_NT)
	if not exist target\wasm32v1-none\optimized mkdir target\wasm32v1-none\optimized
else
	mkdir -p target/wasm32v1-none/optimized
endif

	stellar contract optimize \
		--wasm target/wasm32v1-none/release/bridge_oracle.wasm \
		--wasm-out target/wasm32v1-none/optimized/bridge_oracle.wasm
	stellar contract optimize \
		--wasm target/wasm32v1-none/release/dao_utils.wasm \
		--wasm-out target/wasm32v1-none/optimized/dao_utils.wasm
	stellar contract optimize \
		--wasm target/wasm32v1-none/release/treasury.wasm \
		--wasm-out target/wasm32v1-none/optimized/treasury.wasm

ifeq ($(OS),Windows_NT)
	cd target/wasm32v1-none/optimized/ && for %%i in (*.wasm) do dir "%%i"
else
	cd target/wasm32v1-none/optimized/ && for i in *.wasm ; do ls -l "$$i"; done
endif

fmt:
	cargo fmt --all

clean:
	cargo clean