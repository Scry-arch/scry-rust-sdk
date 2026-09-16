SHELL := /bin/sh
.DELETE_ON_ERROR:

# The SDK owns all compiler flags.
ifdef RUSTFLAGS
	$(error RUSTFLAGS is set in the environment; unset it)
endif

# Single source of truth for the nightly pin.
PIN = $(or $(shell sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml 2>/dev/null),$(error rust-toolchain.toml missing or unreadable.))
DIST := dist
TARGET := scry32-unknown-none-elf
export RUST_TARGET_PATH := $(CURDIR)/targets
ifeq ($(OS),Windows_NT)
  EXE := .exe
endif
BUILD := build
CGCLIF := submodules/rustc_codegen_cranelift

# --- Submodules --------------------------------------------------------------

# Update the submodules and identify freshness through cg_clif's rust-toolchain.toml.
$(CGCLIF)/rust-toolchain.toml:
	git submodule update --init --recursive

# Bump every submodule to its remote's current default branch. Uses
# reset --hard because the forks rebase/force-push; refuses if a submodule
# has local changes unless FORCE_SUBMODULES=1 is set (hack in the standalone
# clones, not under submodules/).
.PHONY: update-submodules
update-submodules:
	git submodule sync --quiet
	git submodule update --init --recursive
	@git config --file .gitmodules --get-regexp '\.path$$' | cut -d' ' -f2 | \
	while read -r sub; do \
	  if [ -z "$(FORCE_SUBMODULES)" ] && [ -n "$$(git -C "$$sub" status --porcelain)" ]; then \
	    echo "error: $$sub has local changes; commit or discard them, or rerun with FORCE_SUBMODULES=1" >&2; \
	    exit 1; \
	  fi; \
	  echo "== $$sub"; \
	  git -C "$$sub" fetch origin || exit 1; \
	  branch=$$(git -C "$$sub" symbolic-ref -q --short refs/remotes/origin/HEAD | sed 's|^origin/||'); \
	  [ -n "$$branch" ] || branch=main; \
	  git -C "$$sub" reset --hard "origin/$$branch" || exit 1; \
	done

# -----------------------------------------------------------------------------

# We use cg_clif to pin which rust nightly version must be used.
rust-toolchain.toml: $(CGCLIF)/rust-toolchain.toml
	cp $< $@

# Install the rust nightly version that must be used
.PHONY: toolchain
toolchain: rust-toolchain.toml
	rustup toolchain install $(PIN)

# Check if cg_clif has changed and put the commit hash in a file
$(BUILD)/cgclif.rev: FORCE
	@mkdir -p $(BUILD)
	@rev=$$(git -C $(CGCLIF) rev-parse HEAD); \
	 [ "$$rev" = "$$(cat $@ 2>/dev/null || true)" ] || echo "$$rev" > $@
.PHONY: FORCE
FORCE:

# Path to the cranelift build library that rustc uses for compiling to Scry
ifeq ($(OS),Windows_NT)
  BACKEND_DLL := rustc_codegen_cranelift.dll
else ifeq ($(shell uname -s),Darwin)
  BACKEND_DLL := librustc_codegen_cranelift.dylib
else
  BACKEND_DLL := librustc_codegen_cranelift.so
endif
BACKEND := $(DIST)/lib/$(BACKEND_DLL)

$(BACKEND): $(BUILD)/cgclif.rev rust-toolchain.toml
	mkdir -p $(DIST)/lib
	cd $(CGCLIF) && ./y.sh build > ../../$(BUILD)/backend.log 2>&1 \
	  || { cat ../../$(BUILD)/backend.log; exit 1; }
	! grep -q 'was not used in the crate graph' $(BUILD)/backend.log
	cp $(CGCLIF)/dist/lib/$(BACKEND_DLL) $@
	

# --- wild linker -------------------------------------------------------------

# The Scry fork of the wild linker ships as a host binary in $(DIST)/bin.
WILD_SRC := submodules/scry-wild
WILD := $(DIST)/bin/wild$(EXE)

# wild's default `fork` feature is Unix-only; its own CI builds Windows and
# macOS with --no-default-features, so do the same here.
ifeq ($(shell uname -s),Linux)
  WILD_FEATURES :=
else
  WILD_FEATURES := --no-default-features
endif

# Submodule-rev stamp, same trick as cgclif.rev: only touched when HEAD moves.
$(BUILD)/wild.rev: FORCE
	@mkdir -p $(BUILD)
	@rev=$$(git -C $(WILD_SRC) rev-parse HEAD); \
	 [ "$$rev" = "$$(cat $@ 2>/dev/null || true)" ] || echo "$$rev" > $@

# Built with the pinned nightly so no separate stable toolchain is needed.
# --locked honours the fork's Cargo.lock (which pins the Scry object fork).
# The target dir lives under $(BUILD) so the submodule checkout stays clean.
$(WILD): $(BUILD)/wild.rev rust-toolchain.toml
	mkdir -p $(DIST)/bin
	cargo +$(PIN) build --locked --release $(WILD_FEATURES) \
	  --manifest-path $(WILD_SRC)/Cargo.toml --package wild-linker --bin wild \
	  --target-dir $(BUILD)/wild > $(BUILD)/wild.log 2>&1 \
	  || { cat $(BUILD)/wild.log; exit 1; }
	cp $(BUILD)/wild/release/wild$(EXE) $@

.PHONY: wild
wild: $(WILD)

# Build the sysroot folder for the Scry target
SYSROOT_LIB := $(DIST)/lib/rustlib/$(TARGET)/lib

# use a file to check that the sysroot folder has been created
$(DIST)/.sysroot.stamp:
	mkdir -p $(SYSROOT_LIB)
	touch $@

.PHONY: sysroot-base
sysroot-base: $(DIST)/.sysroot.stamp

# Build the core library
CORE_RLIB := $(DIST)/lib/rustlib/$(TARGET)/lib/libcore.rlib
$(CORE_RLIB): $(wildcard sysroot/core/src/*.rs) $(BACKEND) targets/$(TARGET).json | sysroot-base
	rustc +$(PIN) -Zunstable-options -Zcodegen-backend=$(abspath $(BACKEND)) \
	  --target $(TARGET) --edition 2024 --crate-name core --crate-type rlib \
	  -Ccodegen-units=1 -o $@ sysroot/core/src/lib.rs

# Build the compiler_builtins library. rustc injects `extern crate
# compiler_builtins` into every #![no_std] crate, and cg_clif calls its mem*
# functions for aggregate copies above its inlining threshold. Needs
# --sysroot: the crate is #![no_std] against scry-core. Overflow checks off:
# these are the innermost loops of every copy and their indices cannot
# overflow (i < n <= usize::MAX).
BUILTINS_RLIB := $(DIST)/lib/rustlib/$(TARGET)/lib/libcompiler_builtins.rlib
$(BUILTINS_RLIB): $(wildcard sysroot/compiler_builtins/src/*.rs) $(CORE_RLIB) targets/$(TARGET).json
	rustc +$(PIN) -Zunstable-options -Zcodegen-backend=$(abspath $(BACKEND)) \
	  --target $(TARGET) --sysroot $(DIST) --edition 2024 \
	  --crate-name compiler_builtins --crate-type rlib \
	  -Ccodegen-units=1 -Coverflow-checks=no -o $@ sysroot/compiler_builtins/src/lib.rs

.PHONY: build-all
build-all: $(BACKEND) $(WILD) $(CORE_RLIB) $(BUILTINS_RLIB)