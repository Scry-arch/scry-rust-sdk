//! Shared code of `cargo-scry` and `scry-load`: board profiles, turning an ELF into the image a
//! board loads, and the board loader's serial protocol.

pub mod board;
pub mod image;
pub mod loader;
