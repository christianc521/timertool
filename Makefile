# ──────────────────────────────────────────────
# TimeTool v2 — Build Commands
# ──────────────────────────────────────────────
# 
# Your .cargo/config.toml is set up for ESP hardware by default.
# These commands handle the target/feature switching for you.
#

# Build and flash to ESP32-S3 hardware (default workflow, uses .cargo/config.toml)
.PHONY: flash
flash:
	cargo run --release

# Build for hardware without flashing
.PHONY: build
build:
	cargo build --release

# ──────────────────────────────────────────────
# Desktop simulator
# ──────────────────────────────────────────────
# Requires SDL2: sudo apt install libsdl2-dev

# Detect host triple (e.g. x86_64-unknown-linux-gnu, aarch64-apple-darwin)
HOST_TARGET := $(shell rustc -vV | grep '^host:' | cut -d' ' -f2)

.PHONY: clean-sim
clean-sim:
	cargo clean --target $(HOST_TARGET)

.PHONY: sim
sim:
	RUST_BACKTRACE=1 cargo +stable run \
		--bin simulator \
		--features simulator \
		--no-default-features \
		--target $(HOST_TARGET) \
		--config 'build.target="$(HOST_TARGET)"' \
		--config 'unstable.build-std=[]'

.PHONY: sim-release
sim-release:
	RUST_BACKTRACE=1 cargo +stable run --release \
		--bin simulator \
		--features simulator \
		--no-default-features \
		--target $(HOST_TARGET) \
		--config 'build.target="$(HOST_TARGET)"' \
		--config 'unstable.build-std=[]'
