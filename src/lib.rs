// Copyright (c) 2016 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>

//! A library for laser projection in Rust.

/// Re-exports from the Etherdream crate.
pub mod etherdream {
  extern crate etherdream;
  pub use self::etherdream::*;
}

/// Re-exports from the ILDA crate.
pub mod ilda {
  extern crate ilda;
  pub use self::ilda::*;
}

