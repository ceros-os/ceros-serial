#![no_std]

// Externs for various crates we use
extern crate alloc;
extern crate core;
extern crate newlib_alloc;

// Modules that we define
#[cfg(feature = "v5")]
mod internal; // Internal functions that the user should not use
#[cfg(feature = "v5")]
pub mod serial; // Actual serial implementation

pub mod protocol; // Contains the basic protocol implementation