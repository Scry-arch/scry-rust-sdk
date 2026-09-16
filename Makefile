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


# --- Target spec residency ----------------------------------------------------

# rustc looks for custom target specs in <sysroot>/lib/rustlib/<target>/target.json,
# so anything building against $(DIST) needs neither RUST_TARGET_PATH nor a path
# to the JSON. The Makefile's own rustc invocations still use RUST_TARGET_PATH,
# since they build the sysroot itself.
TARGET_SPEC := $(DIST)/lib/rustlib/$(TARGET)/target.json
$(TARGET_SPEC): targets/$(TARGET).json | sysroot-base
	cp $< $@

# --- cargo-scry wrapper --------------------------------------------------------

# `cargo scry <subcommand>` runs cargo for the Scry target with everything in
# $(DIST) injected, so projects need no configuration. The toolchain pin is
# baked in at build time.
CARGO_SCRY := $(DIST)/bin/cargo-scry$(EXE)
$(CARGO_SCRY): $(wildcard wrapper/cargo-scry/src/*.rs) wrapper/cargo-scry/Cargo.toml rust-toolchain.toml
	mkdir -p $(DIST)/bin
	SCRY_TOOLCHAIN=$(PIN) cargo +$(PIN) build --release \
	  --manifest-path wrapper/cargo-scry/Cargo.toml \
	  --target-dir $(BUILD)/cargo-scry > $(BUILD)/cargo-scry.log 2>&1 \
	  || { cat $(BUILD)/cargo-scry.log; exit 1; }
	cp $(BUILD)/cargo-scry/release/cargo-scry$(EXE) $@

.PHONY: cargo-scry
cargo-scry: $(CARGO_SCRY)

# --- Run tests -----------------------------------------------------------------

# Every directory under test/ holding an expected.txt is a run test: a plain
# cargo project that `make check-run` builds with `cargo scry`, runs on scryer,
# and whose returned operands (the line after "Returned Operands") must match
# expected.txt exactly. Add a test by adding such a directory; run one with
# `make check-run-<name>`. Output of each run is kept in $(BUILD)/test/.
# Needs $(DIST)/bin on PATH (for cargo-scry) and scryer installed, exactly as
# a user of the SDK would have them.
RUN_TESTS := $(patsubst test/%/expected.txt,%,$(wildcard test/*/expected.txt))

.PHONY: check-run
check-run: $(RUN_TESTS:%=check-run-%)

check-run-%: test/%/expected.txt build-all
	@mkdir -p $(BUILD)/test
	@cd test/$* && cargo scry run --quiet > "$(abspath $(BUILD))/test/$*.out" 2>&1 \
	  || { echo "check-run-$*: FAILED to build or run:"; cat "$(abspath $(BUILD))/test/$*.out"; exit 1; }
	@sed -n '/Returned Operands/{n;s/[[:space:]]*$$//;p;}' $(BUILD)/test/$*.out > $(BUILD)/test/$*.actual
	@sed 's/[[:space:]]*$$//' test/$*/expected.txt > $(BUILD)/test/$*.expected
	@git diff --no-index --quiet $(BUILD)/test/$*.expected $(BUILD)/test/$*.actual \
	  || { echo "check-run-$*: FAILED, expected:"; cat $(BUILD)/test/$*.expected; echo "actual:"; cat $(BUILD)/test/$*.actual; exit 1; }
	@echo "check-run-$*: OK"

.PHONY: check
check: check-run

.PHONY: build-all
build-all: $(BACKEND) $(WILD) $(TARGET_SPEC) $(CORE_RLIB) $(BUILTINS_RLIB) $(CARGO_SCRY)
