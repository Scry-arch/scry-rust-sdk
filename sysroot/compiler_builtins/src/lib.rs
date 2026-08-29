//! Stub `compiler_builtins` for Scry. rustc injects `extern crate
//! compiler_builtins` into every `#![no_std]` crate, so this crate must exist
//! in the sysroot even while empty. Phase 5 of the SDK guide fills it in:
//! memcpy/memmove/memset/memcmp and the __udivsi3-family division helpers.
#![feature(compiler_builtins, no_core)]
#![compiler_builtins]
#![no_std]
#![no_core]
