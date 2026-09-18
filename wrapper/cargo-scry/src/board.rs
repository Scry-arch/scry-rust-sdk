//! Board profiles.
//!
//! A profile is a TOML file that states facts about one piece of hardware: how programs for it
//! must be linked, which memory they may occupy, and how its loader is reached. The SDK ships
//! profiles in `boards/`; a project can also point at a profile file of its own.

use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    /// Human-readable description, used in messages.
    pub name: String,

    #[serde(default)]
    pub link: Link,

    pub memory: Memory,

    #[serde(default)]
    pub loader: Loader,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// Extra arguments for the linker, e.g. `--image-base=0x0`.
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    /// The range that a program's code and data, including zero-initialised data, must fit in.
    pub program: Region,
}

/// A half-open address range, `start..end`.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub start: u64,
    pub end: u64,
}

impl Region {
    pub fn contains_range(&self, start: u64, end: u64) -> bool {
        self.start <= start && end <= self.end
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Loader {
    /// Serial speed of the board's loader.
    #[serde(default = "default_baud")]
    pub baud: u32,

    /// The largest payload, in bytes, that the loader accepts in one download.
    #[serde(default)]
    pub max_payload: Option<u64>,
}

impl Default for Loader {
    fn default() -> Self {
        Loader {
            baud: default_baud(),
            max_payload: None,
        }
    }
}

fn default_baud() -> u32 {
    115_200
}

impl Board {
    pub fn load(path: &Path) -> Result<Board, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read board profile {}: {error}", path.display()))?;
        Board::parse(&text)
            .map_err(|error| format!("invalid board profile {}: {error}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Board, String> {
        let board: Board = toml::from_str(text).map_err(|error| error.to_string())?;
        let program = board.memory.program;
        if program.start >= program.end {
            return Err(format!(
                "memory.program is empty: start {:#x} is not below end {:#x}",
                program.start, program.end
            ));
        }
        Ok(board)
    }
}

/// Finds the profile for `--board <value>`: a path if the value looks like one, otherwise the
/// name of a profile shipped in `boards_dir`.
pub fn resolve(boards_dir: &Path, value: &str) -> Result<PathBuf, String> {
    let looks_like_path = value.ends_with(".toml") || value.contains('/') || value.contains('\\');
    if looks_like_path {
        let path = PathBuf::from(value);
        return if path.is_file() {
            Ok(path)
        } else {
            Err(format!("board profile {value} does not exist"))
        };
    }

    let path = boards_dir.join(format!("{value}.toml"));
    if path.is_file() {
        return Ok(path);
    }

    let mut known = std::fs::read_dir(boards_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()? == "toml").then(|| path.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect::<Vec<_>>();
    known.sort();
    Err(format!(
        "unknown board `{value}`; boards shipped with the SDK: {}",
        if known.is_empty() {
            "none".to_owned()
        } else {
            known.join(", ")
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = r#"
        name = "Test board"

        [link]
        args = ["--image-base=0x0", "-n"]

        [memory]
        program = { start = 0x0000_0000, end = 0x0001_0000 }

        [loader]
        max-payload = 0xFFFF
    "#;

    #[test]
    fn parses_a_profile() {
        let board = Board::parse(PROFILE).unwrap();
        assert_eq!(board.name, "Test board");
        assert_eq!(board.link.args, ["--image-base=0x0", "-n"]);
        assert_eq!(board.memory.program.start, 0);
        assert_eq!(board.memory.program.end, 0x1_0000);
        assert_eq!(board.loader.baud, 115_200);
        assert_eq!(board.loader.max_payload, Some(0xFFFF));
    }

    #[test]
    fn link_and_loader_sections_are_optional() {
        let board = Board::parse(
            "name = \"Minimal\"\n[memory]\nprogram = { start = 0x100, end = 0x200 }\n",
        )
        .unwrap();
        assert!(board.link.args.is_empty());
        assert_eq!(board.loader.max_payload, None);
    }

    #[test]
    fn rejects_unknown_keys_and_empty_regions() {
        let typo = PROFILE.replace("max-payload", "max-payloads");
        assert!(Board::parse(&typo).is_err());

        let empty = "name = \"x\"\n[memory]\nprogram = { start = 0x200, end = 0x200 }\n";
        let error = Board::parse(empty).unwrap_err();
        assert!(error.contains("memory.program is empty"), "{error}");
    }

    #[test]
    fn region_containment() {
        let region = Region {
            start: 0x100,
            end: 0x200,
        };
        assert!(region.contains_range(0x100, 0x200));
        assert!(region.contains_range(0x180, 0x190));
        assert!(!region.contains_range(0xff, 0x110));
        assert!(!region.contains_range(0x1f0, 0x201));
    }
}
