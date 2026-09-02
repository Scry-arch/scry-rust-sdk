//! Scry `compiler_builtins`: rustc injects `extern crate compiler_builtins`
//! into every `#![no_std]` crate, and cg_clif emits calls to the mem*
//! functions for aggregate copies/compares above its inlining threshold.
#![feature(compiler_builtins)]
#![compiler_builtins]
#![no_std]
#![allow(internal_features)]

mod mem;
