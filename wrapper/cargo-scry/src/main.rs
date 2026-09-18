//! `cargo scry <subcommand> ...`: runs cargo for the Scry target.
//!
//! The binary lives in `<sdk>/dist/bin` and finds everything else relative to itself: the
//! cranelift backend in `dist/lib`, the sysroot in `dist`, the linker in `dist/bin`. It injects
//! all of that through cargo's `--config` flags and pins the toolchain through `RUSTUP_TOOLCHAIN`,
//! so a project needs no `.cargo/config.toml`, no `rust-toolchain.toml` and no special profile
//! settings.
//!
//! Without a board, programs are linked for and run on the scryer simulator. With one, selected by
//! `--board <name>`, the `SCRY_BOARD` environment variable, or `board = "<name>"` under
//! `[package.metadata.scry]` in the project's `Cargo.toml`, the board's profile adds its linker
//! arguments and `cargo scry run` downloads the program to the board through `scry-load`.

use cargo_scry::board;
use cargo_scry::board::Board;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;

/// The target triple. rustc finds its spec by this name in the sysroot, and scryer uses the same
/// name for its `--target` option.
const TARGET: &str = "scry32-unknown-none-elf";

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

/// Subcommands that execute the program through the runner.
const RUN_SUBCOMMANDS: &[&str] = &["run", "r", "test", "t", "bench"];

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
    for (what, path) in [
        ("backend", &backend),
        ("linker", &linker),
        ("sysroot", &core),
    ] {
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
    let board_flag = take_board_flag(&mut args).unwrap_or_else(|message| fail(&message));
    let Some(subcommand) = args.first().cloned() else {
        eprintln!("usage: cargo scry <cargo subcommand> [--board <name>] [args...]");
        eprintln!(
            "       e.g. `cargo scry build`, `cargo scry run`, `cargo scry run --board scry5-nexys-a7`"
        );
        exit(2);
    };

    let board_profile = select_board(board_flag, &args, &dist.join("boards"))
        .unwrap_or_else(|message| fail(&message));

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

    let mut rustflags = vec![
        "-Zunstable-options".to_owned(),
        format!("-Zcodegen-backend={}", toml_path(&backend)),
        "--sysroot".to_owned(),
        toml_path(&dist),
        format!("-Cmetadata=scry-sdk-{stamp:x}"),
    ];
    if let Some((_, board)) = &board_profile {
        rustflags.extend(
            board
                .link
                .args
                .iter()
                .map(|arg| format!("-Clink-arg={arg}")),
        );
    }
    config(
        &mut cmd,
        &format!("target.{TARGET}.rustflags={}", toml_array(&rustflags)),
    );
    config(
        &mut cmd,
        &format!("target.{TARGET}.linker=\"{}\"", toml_path(&linker)),
    );

    let runs_program = RUN_SUBCOMMANDS.contains(&subcommand.as_str());
    match &board_profile {
        // A board: the runner downloads the program to it.
        Some((profile, _)) => {
            let loader = dist
                .join("bin")
                .join(format!("scry-load{}", env::consts::EXE_SUFFIX));
            if runs_program && !loader.exists() {
                fail(&format!(
                    "scry-load not found at {}; run `make build-all` in the SDK",
                    loader.display()
                ));
            }
            let runner = [
                toml_path(&loader),
                "--board-file".to_owned(),
                toml_path(profile),
            ];
            config(
                &mut cmd,
                &format!("target.{TARGET}.runner={}", toml_array(&runner)),
            );
        }
        // No board: the runner is the simulator.
        None => match find_scryer(&dist) {
            Some(scryer) => {
                let runner = [toml_path(&scryer), "--target".to_owned(), TARGET.to_owned()];
                config(
                    &mut cmd,
                    &format!("target.{TARGET}.runner={}", toml_array(&runner)),
                );
            }
            None if runs_program => {
                fail("scryer not found in the SDK's dist/bin, on PATH or in ~/.cargo/bin");
            }
            None => {}
        },
    }

    cmd.args(&args[1..]);

    let status = cmd.status().unwrap_or_else(|error| {
        fail(&format!(
            "failed to run cargo for toolchain {TOOLCHAIN}: {error}; run `make toolchain` in the SDK"
        ))
    });
    exit(status.code().unwrap_or(1));
}

/// Removes `--board <value>` or `--board=<value>` from the cargo arguments and returns the value.
/// Arguments after `--` belong to the program's runner and are left alone.
fn take_board_flag(args: &mut Vec<String>) -> Result<Option<String>, String> {
    let end = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    let mut board = None;
    let mut index = 0;
    let mut remaining = end;
    while index < remaining {
        if let Some(value) = args[index].strip_prefix("--board=") {
            board = Some(value.to_owned());
            args.remove(index);
            remaining -= 1;
        } else if args[index] == "--board" {
            if index + 1 >= remaining {
                return Err("--board needs a board name or a profile file".to_owned());
            }
            board = Some(args.remove(index + 1));
            args.remove(index);
            remaining -= 2;
        } else {
            index += 1;
        }
    }
    Ok(board)
}

/// Decides which board, if any, to build for: the `--board` flag, then the `SCRY_BOARD`
/// environment variable, then `[package.metadata.scry] board` in the project's manifest.
fn select_board(
    flag: Option<String>,
    args: &[String],
    boards_dir: &Path,
) -> Result<Option<(PathBuf, Board)>, String> {
    let from_env = env::var("SCRY_BOARD")
        .ok()
        .filter(|value| !value.is_empty());
    let (value, relative_to) = match flag.or(from_env) {
        Some(value) => (value, None),
        None => match board_from_manifest(args)? {
            Some((value, manifest_dir)) => (value, Some(manifest_dir)),
            None => return Ok(None),
        },
    };

    // A profile path written in a manifest is relative to that manifest.
    let value = match relative_to {
        Some(dir) if value.ends_with(".toml") && Path::new(&value).is_relative() => {
            dir.join(&value).display().to_string()
        }
        _ => value,
    };

    let profile = board::resolve(boards_dir, &value)?;
    let board = Board::load(&profile)?;
    Ok(Some((profile, board)))
}

/// Reads `board` from `[package.metadata.scry]` of the manifest cargo will use. Returns it with
/// the manifest's directory.
fn board_from_manifest(args: &[String]) -> Result<Option<(String, PathBuf)>, String> {
    let explicit = args.iter().enumerate().find_map(|(index, arg)| {
        arg.strip_prefix("--manifest-path=")
            .map(PathBuf::from)
            .or_else(|| {
                (arg == "--manifest-path").then(|| args.get(index + 1).map(PathBuf::from))?
            })
    });
    let manifest = match explicit {
        Some(path) => path,
        None => {
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            match cwd
                .ancestors()
                .map(|dir| dir.join("Cargo.toml"))
                .find(|path| path.is_file())
            {
                Some(path) => path,
                None => return Ok(None),
            }
        }
    };

    let Ok(text) = std::fs::read_to_string(&manifest) else {
        return Ok(None);
    };
    let table: toml::Table = text
        .parse()
        .map_err(|error| format!("cannot parse {}: {error}", manifest.display()))?;
    let board = table
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("scry"))
        .and_then(|scry| scry.get("board"));
    match board {
        None => Ok(None),
        Some(toml::Value::String(board)) => {
            let dir = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
            Ok(Some((board.clone(), dir)))
        }
        Some(_) => Err(format!(
            "`board` under [package.metadata.scry] in {} must be a string",
            manifest.display()
        )),
    }
}

/// The SDK's `dist` directory, found relative to this binary, which lives in `dist/bin`.
fn sdk_dist_dir() -> PathBuf {
    let exe = env::current_exe()
        .unwrap_or_else(|error| fail(&format!("cannot locate cargo-scry: {error}")));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn board_flag_is_removed_from_the_cargo_arguments() {
        let mut list = args(&["run", "--board", "scry5-nexys-a7", "--release"]);
        assert_eq!(
            take_board_flag(&mut list).unwrap().as_deref(),
            Some("scry5-nexys-a7")
        );
        assert_eq!(list, args(&["run", "--release"]));

        let mut list = args(&["build", "--board=boards/mine.toml"]);
        assert_eq!(
            take_board_flag(&mut list).unwrap().as_deref(),
            Some("boards/mine.toml")
        );
        assert_eq!(list, args(&["build"]));

        let mut list = args(&["build", "--release"]);
        assert_eq!(take_board_flag(&mut list).unwrap(), None);
        assert_eq!(list, args(&["build", "--release"]));
    }

    #[test]
    fn arguments_after_the_separator_belong_to_the_runner() {
        let mut list = args(&[
            "run",
            "--board",
            "scry5-nexys-a7",
            "--",
            "--board",
            "other",
            "--check",
        ]);
        assert_eq!(
            take_board_flag(&mut list).unwrap().as_deref(),
            Some("scry5-nexys-a7")
        );
        assert_eq!(list, args(&["run", "--", "--board", "other", "--check"]));
    }

    #[test]
    fn board_flag_without_a_value_is_an_error() {
        assert!(take_board_flag(&mut args(&["run", "--board"])).is_err());
        assert!(take_board_flag(&mut args(&["run", "--board", "--", "x"])).is_err());
    }
}
