SHELL := /bin/sh
.DELETE_ON_ERROR:

# The SDK owns all compiler flags.
ifdef RUSTFLAGS
  $(error RUSTFLAGS is set in the environment; unset it)
endif

# Single source of truth for the nightly pin.
PIN  := $(shell sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)
DIST := dist
TARGET := scry32-unknown-none-elf
export RUST_TARGET_PATH := $(CURDIR)/targets
ifeq ($(OS),Windows_NT)
  EXE := .exe
endif

# We use cg_clif to pin which rust nightly version must be used.
rust-toolchain.toml: submodules/rustc_codegen_cranelift/rust-toolchain.toml
	cp $< $@

# Install the rust nightly version that must be used
.PHONY: toolchain
toolchain: rust-toolchain.toml
	rustup toolchain install $(PIN)
	
