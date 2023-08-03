//! The core crate for Drone, an Embedded Operating System.
//!
//! # Documentation
//!
//! - [Drone Book](https://book.drone-os.com/)
//! - [API documentation](https://api.drone-os.com/drone-core/0.14/)
//!
//! # Usage
//!
//! Add the crate to your `Cargo.toml` dependencies:
//!
//! ```toml
//! [dependencies]
//! drone-core = { version = "0.14.2" }
//! ```
//!
//! Add or extend `std` feature as follows:
//!
//! ```toml
//! [features]
//! std = ["drone-core/std"]
//! ```

#![feature(alloc_layout_extra)]
#![feature(allocator_api)]
#![feature(core_intrinsics)]
#![feature(exhaustive_patterns)]
#![feature(generator_trait)]
#![feature(generators)]
#![feature(lang_items)]
#![feature(marker_trait_attr)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(never_type_fallback)]
#![feature(slice_internals)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(
    incomplete_features,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_doctest_main,
    clippy::precedence,
    clippy::type_repetition_in_bounds,
    clippy::use_self,
    clippy::used_underscore_binding
)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod bitfield;
pub mod fib;
pub mod heap;
pub mod inventory;
pub mod io;
#[cfg(feature = "log")]
pub mod log;
pub mod mem;
pub mod periph;
pub mod proc_loop;
pub mod reg;
pub mod sync;
pub mod thr;
pub mod token;

#[cfg(feature = "lang-items")]
mod lang_items;

/// Defines dynamic memory structures.
///
/// See [the module level documentation](heap) for details.
#[doc(inline)]
pub use drone_core_macros::heap;

/// Defines a new generic peripheral.
///
/// See [the module level documentation](periph) for details.
#[doc(inline)]
pub use drone_core_macros::periph;

/// Defines a memory-mapped register.
///
/// See [the module level documentation](reg) for details.
#[doc(inline)]
pub use drone_core_macros::reg;

#[doc(hidden)]
pub use drone_core_macros::config_override;
