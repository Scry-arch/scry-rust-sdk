//! Panic support types.

/// The location a panic originated from, threaded implicitly by
/// `#[track_caller]`. Messages are never formatted on Scry, but the type must
/// exist for the calling convention of `#[track_caller]` functions.
#[lang = "panic_location"]
pub struct Location<'a> {
    _file: &'a str,
    _line: u32,
    _column: u32,
}
