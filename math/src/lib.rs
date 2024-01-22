#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(
    any(
        feature = "winter_compatibility",
        feature = "winter-math",
        feature = "miden-core"
    ),
    feature = "constant-time"
))]
compile_error!("Feature winter_compatibility and constant-time are mutually exclusive and cannot be enabled together");

pub mod cyclic_group;
pub mod elliptic_curve;
pub mod errors;
pub mod field;
pub mod helpers;
pub mod traits;
pub mod unsigned_integer;

pub mod gpu;

// These modules don't work in no-std mode
pub mod fft;
pub mod msm;
#[cfg(feature = "alloc")]
pub mod polynomial;
