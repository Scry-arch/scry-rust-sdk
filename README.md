# Scry Rust SDK

Build Rust programs for the [Scry](https://github.com/Scry-arch) architecture and run them on the
`scryer` simulator.

The SDK bundles everything rustc needs for the `scry32-unknown-none-elf` target: a Cranelift-based
codegen backend, a minimal `core` library, the `wild` linker with Scry support, and a `cargo scry`
subcommand that ties them together so that ordinary cargo projects need no configuration.

## Prerequisites

Install these first:

- **Git**
- **Rust**, installed through `rustup`. The SDK pins its own nightly toolchain and installs it for
  you (see below), so any existing default toolchain is fine.
- **GNU Make 4 or newer and a POSIX `sh`.** The Makefile is the SDK's build system on every
  platform.
  - Linux: the distribution's `make` is fine.
  - macOS: the system `make` is 3.81, which is too old. Install a current one with
    `brew install make` and put its `gnubin` directory on your `PATH`
    (`$(brew --prefix)/opt/make/libexec/gnubin`).
  - Windows: install [MSYS2](https://www.msys2.org/), then install make from an MSYS2 shell with
    `pacman -S make`. Either work from an MSYS2 shell, or add MSYS2's `usr\bin` directory (for
    example `C:\msys64\usr\bin`) to your `PATH` and run `make` from PowerShell. Do not run MSYS2's
    `make` from Git Bash: the two environments don't mix and the build fails at link time.
- **scryer**, the Scry simulator, which executes the programs you build. Install it with cargo:

  ```sh
  cargo install --locked --git https://github.com/Scry-arch/scryer.git scryer
  ```

  This puts `scryer` in `~/.cargo/bin`, which rustup already added to your `PATH`.

## A minimal working example

### 1. Build the SDK

Clone the repository and build it. The Makefile fetches the submodules itself, so a plain clone is
enough.

```sh
git clone https://github.com/Scry-arch/scry-rust-sdk.git
cd scry-rust-sdk
make build-all
```

This installs the pinned nightly through rustup and builds the codegen
backend, the linker, the `core` and `compiler_builtins` libraries, and the `cargo scry` wrapper, and
places everything under `dist/`. The first build compiles Cranelift and the linker from source, so
expect it to take a while; later runs only rebuild what changed.

### 2. Put the SDK on your `PATH`

Add the SDK's `dist/bin` directory to your `PATH`. Cargo finds the `cargo-scry` binary there, which
is what makes the `cargo scry` subcommand available.

```sh
# Linux, macOS, MSYS2
export PATH="$PWD/dist/bin:$PATH"
```

```powershell
# Windows PowerShell
$env:Path = "$PWD\dist\bin;" + $env:Path
```

These commands are only active for the life of the terminal. 
Use the regular means of permanently adding them to the PATH when needed.

### 3. Build and run with cargo

The repository contains a minimal program in `test/hello-scry`. It is an ordinary cargo project:
its `Cargo.toml` has nothing Scry-specific in it. The `cargo scry` wrapper supplies the target, the toolchain, the backend, the sysroot and the linker.
Therefore, any `cargo` build commands used must include `scry`, otherwise they will fail.

```rust
#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> u32 {
    add(40, 2)
}

#[inline(never)]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

A Scry program is `#![no_std]` and `#![no_main]`, and its entry point is an `extern "C"` function
named `_start`. Whatever `_start` returns is what the simulator reports when the program finishes.

Build and run it:

```sh
cd test/hello-scry
cargo scry run
```

`cargo scry run` compiles the program, links it, and hands the result to scryer. The interesting
part of the output is the value returned from `_start`:

```text
----------  Returned Operands  ----------
42u32,
```

It is followed by the simulator's execution metrics.

To only compile, use `cargo scry build`. Any other cargo subcommand and its options work the same
way, for example `cargo scry build --release` or `cargo scry clean`.

### 4. Run the ELF on scryer by hand

`cargo scry build` produces a normal 32-bit ELF executable. For the example above it is

```text
test/hello-scry/target/scry32-unknown-none-elf/debug/hello-scry
```

(`release` instead of `debug` for a release build.) To run it yourself, pass it to scryer together
with the target triple, which is the same one the compiler uses:

```sh
scryer --target scry32-unknown-none-elf target/scry32-unknown-none-elf/debug/hello-scry
```

This prints the same "Returned Operands" block as `cargo scry run`, which does exactly this for you.
Useful scryer options are `--timeout <N>` to stop a program that does not terminate and `--debug`
to print the machine state after every step; `scryer --help` lists the rest.

### Starting your own project

Create a project with `cargo new`, replace `src/main.rs` with a program shaped like the one above,
and use `cargo scry` instead of `cargo`. Nothing else is needed.

The SDK's `core` library is deliberately small. Integer arithmetic is available for types up to 32
bits, `for` loops work over `a..b` ranges, and `panic!` accepts a string literal only. There is no
`std`, no heap, no floating point and no formatting yet.

## Testing the SDK

`make check-run` builds every project under `test/` with `cargo scry`, runs it on scryer, and
compares the returned operands with the project's `expected.txt`. It needs `dist/bin` on your
`PATH` and scryer installed, exactly as above. To add a test, add a directory with a cargo project
and an `expected.txt`; `make check-run-<name>` runs a single one.
