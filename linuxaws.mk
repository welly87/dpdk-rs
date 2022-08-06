# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

#=======================================================================================================================
# Default Paths
#=======================================================================================================================

export PREFIX ?= $(HOME)
# export PKG_CONFIG_PATH ?= $(shell find $(PREFIX)/lib/ -name '*pkgconfig*' -type d | xargs | sed -e 's/\s/:/g')
export PKG_CONFIG_PATH=/usr/local/lib64/pkgconfig
#=======================================================================================================================
# Tools
#=======================================================================================================================

export CARGO ?= $(HOME)/.cargo/bin/cargo
export RM ?= rm -rf

#=======================================================================================================================
# Switches
#=======================================================================================================================

# Set build mode.
ifneq ($(DEBUG),yes)
export BUILD = release
else
export BUILD = dev
endif

# Set build flags.
export FLAGS += --profile $(BUILD)

# Set driver version.
# export DRIVER ?= $(shell [ ! -z "`lspci | grep -E "ConnectX-[4,5]"`" ] && echo mlx5 || echo mlx4)
# export FLAGS += --features=$(DRIVER)

export DRIVER = ena
export FLAGS += --features=$(DRIVER)