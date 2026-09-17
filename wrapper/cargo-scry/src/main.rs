//! `cargo scry <subcommand> ...`: runs cargo for the Scry target.
//!
//! The binary lives in `<sdk>/dist/bin` and finds everything else relative to itself: the
//! cranelift backend in `dist/lib`, the sysroot in `dist`, the linker in `dist/bin`. It injects
//! all of that through cargo's `--config` flags and pins the toolchain through `RUSTUP_TOOLCHAIN`,
//! so a project needs no `.cargo/config.toml`, no `rust-toolchain.toml` and no special profile
//! settings.

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;

/// The rustc target. Its spec lives in the sysroot, so rustc finds it by name.
const TARGET: &str = "scry32-unknown-none-elf";

/// The name scryer uses for the same target.
const SCRYER_TARGET: &str = "scry-unknown-none-elf32";

/// The nightly the SDK was built with. Baked in by the Makefile.
const TOOLCHAIN: &str = env!(
    "SCRY_TOOLCHAIN",
    "SCRY_TOOLCHAIN must be set to the SDK's toolchain pin; build via the SDK Makefile"
);

/// Subcommands that build for a target and so need `--target`.
const TARGET_SUBCOMMANDS: &[&str] = &[
    "build", "b", "check", "c", "clippy", "run", "r", "test", "t", "bench", "doc", "rustc",
    "rustdoc", "clean", "fix",
];

fn main() {
    let dist = sdk_dist_dir();

    // The SDK owns all compiler flags. A leaked RUSTFLAGS silently changes what gets built.
    for var in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"] {
        if env::var_os(var).is_some() {
            fail(&format!(
                "{var} is set in the environment; the Scry SDK owns all compiler flags, unset it"
            ));
        }
    }

    let backend = dist.join("lib").join(format!(
        "{}rustc_codegen_cranelift{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    ));
    let linker = dist
        .join("bin")
        .join(format!("wild{}", env::consts::EXE_SUFFIX));
    let core = dist
        .join("lib")
        .join("rustlib")
        .join(TARGET)
        .join("lib")
        .join("libcore.rlib");
    for (what, path) in [("backend", &backend), ("linker", &linker), ("sysroot", &core)] {
        if !path.exists() {
            fail(&format!(
                "{what} not found at {}; run `make build-all` in the SDK",
                path.display()
            ));
        }
    }

    let mut args: Vec<String> = env::args().skip(1).collect();
    // When invoked as `cargo scry ...`, cargo passes `scry` as the first argument.
    if args.first().is_some_and(|arg| arg == "scry") {
        args.remove(0);
    }
    let Some(subcommand) = args.first().cloned() else {
        eprintln!("usage: cargo scry <cargo subcommand> [args...]");
        eprintln!("       e.g. `cargo scry build`, `cargo scry run`");
        exit(2);
    };

    let mut cmd = Command::new("cargo");
    cmd.env("RUSTUP_TOOLCHAIN", TOOLCHAIN);
    cmd.arg(&subcommand);
    if TARGET_SUBCOMMANDS.contains(&subcommand.as_str()) {
        cmd.arg("--target").arg(TARGET);
    }

    // Cargo decides whether to rebuild from the text of the flags, not from the files they name,
    // so without this a project keeps its old binaries after the SDK's backend or sysroot is
    // rebuilt. Mixing a stamp of those files into the crate metadata changes the flags whenever
    // they change.
    let builtins = core.with_file_name("libcompiler_builtins.rlib");
    let stamp = sdk_stamp(&[&backend, &core, &builtins]);

    let rustflags = [
        "-Zunstable-options".to_owned(),
        format!("-Zcodegen-backend={}", toml_path(&backend)),
        "--sysroot".to_owned(),
        toml_path(&dist),
        format!("-Cmetadata=scry-sdk-{stamp:x}"),
    ];
    config(
        &mut cmd,
        &format!("target.{TARGET}.rustflags={}", toml_array(&rustflags)),
    );
    config(
        &mut cmd,
        &format!("target.{TARGET}.linker=\"{}\"", toml_path(&linker)),
    );
    match find_scryer(&dist) {
        Some(scryer) => {
            let runner = [
                toml_path(&scryer),
                "--target".to_owned(),
                SCRYER_TARGET.to_owned(),
            ];
            config(
                &mut cmd,
                &format!("target.{TARGET}.runner={}", toml_array(&runner)),
            );
        }
        None if matches!(subcommand.as_str(), "run" | "r" | "test" | "t" | "bench") => {
            fail("scryer not found in the SDK's dist/bin, on PATH or in ~/.cargo/bin");
        }
        None => {}
    }

    cmd.args(&args[1..]);

    let status = cmd.status().unwrap_or_else(|error| {
        fail(&format!(
            "failed to run cargo for toolchain {TOOLCHAIN}: {error}; run `make toolchain` in the SDK"
        ))
    });
    exit(status.code().unwrap_or(1));
}

/// The SDK's `dist` directory, found relative to this binary, which lives in `dist/bin`.
fn sdk_dist_dir() -> PathBuf {
    let exe = env::current_exe().unwrap_or_else(|error| fail(&format!("cannot locate cargo-scry: {error}")));
    exe.parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| fail("cargo-scry must be installed in the SDK's dist/bin directory"))
}

/// A value that changes whenever any of the supplied files is rebuilt: a hash of their sizes and
/// modification times.
fn sdk_stamp(files: &[&PathBuf]) -> u64 {
    // FNV-1a. Must be stable across runs, which `std`'s default hasher isn't guaranteed to be.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |value: u64| {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for file in files {
        let Ok(metadata) = std::fs::metadata(file) else {
            continue;
        };
        mix(metadata.len());
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |since_epoch| since_epoch.as_nanos() as u64);
        mix(modified);
    }
    hash
}

/// Finds scryer: shipped in the SDK, on PATH, or installed with `cargo install`.
fn find_scryer(dist: &Path) -> Option<PathBuf> {
    let name = format!("scryer{}", env::consts::EXE_SUFFIX);
    let mut candidates = vec![dist.join("bin").join(&name)];
    if let Some(path) = env::var_os("PATH") {
        candidates.extend(env::split_paths(&path).map(|dir| dir.join(&name)));
    }
    if let Some(home) = env::var_os("CARGO_HOME") {
        candidates.push(PathBuf::from(home).join("bin").join(&name));
    } else if let Some(home) = env::home_dir() {
        candidates.push(home.join(".cargo").join("bin").join(&name));
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn config(cmd: &mut Command, value: &str) {
    cmd.arg("--config").arg(value);
}

/// A path as a TOML string. Forward slashes work everywhere and avoid TOML escaping.
fn toml_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn toml_array(items: &[String]) -> String {
    let quoted = items
        .iter()
        .map(|item| format!("\"{item}\""))
        .collect::<Vec<_>>();
    format!("[{}]", quoted.join(","))
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    exit(1)
}
