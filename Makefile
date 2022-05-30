# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

#=======================================================================================================================
# Default Paths
#=======================================================================================================================

export PREFIX ?= $(HOME)
export PKG_CONFIG_PATH ?= $(shell find $(PREFIX)/lib/ -name '*pkgconfig*' -type d | xargs | sed -e 's/\s/:/g')

#=======================================================================================================================
# Build Parameters
#=======================================================================================================================

DRIVER ?= $(shell [ ! -z "`lspci | grep -E "ConnectX-[4,5]"`" ] && echo mlx5 || echo mlx4)
CARGO_FEATURES += --features=$(DRIVER)

#=======================================================================================================================
# Toolchain Configuration
#=======================================================================================================================

# Rust Toolchain
export BUILD ?= --release
export CARGO ?= $(HOME)/.cargo/bin/cargo

#=======================================================================================================================

all: all-libs all-tests

all-libs: check-fmt
	$(CARGO) build $(BUILD) $(CARGO_FEATURES)

all-tests: check-fmt
	$(CARGO) build --tests $(BUILD) $(CARGO_FEATURES)

check-fmt: check-fmt-rust

check-fmt-rust:
	$(CARGO) fmt -- --check

test:
	$(CARGO) test --lib $(BUILD) $(CARGO_FEATURES)

clean:
	$(CARGO) clean
