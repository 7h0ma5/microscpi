//! This crate provides a simple interface to create an async SCPI command
//! interpreter. It is especially suited for embedded devices.
//!
//! Notable features are:
//! * No heap memory allocation required.
//! * Compile time creation of the SCPI command tree.
//! * Support for async commmand handler functions.
//!
//! # Minimal Example
//! ```
//! use microscpi as scpi;
//! use microscpi::Interface;
//!
//! pub struct ExampleInterface {
//!     value: u64
//! }
//! 
//! impl scpi::ErrorHandler for ExampleInterface {
//!     fn handle_error(&mut self, error: scpi::Error) {
//!         println!("Error: {error}");
//!     }
//! }
//!
//! #[scpi::interface]
//! impl ExampleInterface {
//!     #[scpi(cmd = "SYSTem:VALue?")]
//!     async fn system_value(&mut self) -> Result<u64, scpi::Error> {
//!         Ok(self.value)
//!     }
//! }
//!
//! #[tokio::main]
//! pub async fn main() {
//!     let mut output = String::new();
//!     let mut interface = ExampleInterface { value: 42 };
//!
//!     interface.parse_and_execute(b"SYSTEM:VAL?\n", &mut output).await;
//!
//!     assert_eq!(output, "42\n");
//! }
//! ```
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(async_fn_in_trait)]
#![allow(clippy::from_str_radix_10)]

#[cfg(feature = "std")]
extern crate std as core;

pub mod commands;
mod error;
mod error_queue;
mod interface;
mod parser;
mod response;
#[doc(hidden)]
mod tree;
mod value;

pub use error::Error;
pub use error_queue::{ErrorQueue, StaticErrorQueue};
pub use interface::{ErrorHandler, Interface};
pub use microscpi_macros::interface;
pub use response::{Mnemonic, Response};
#[doc(hidden)]
pub use tree::Node;
pub use value::Value;

/// Reference identifier of a command or query
///
/// Due to current limitations with async function pointers, the references to
/// the command handler functions are stored as integers.
#[doc(hidden)]
pub type CommandId = usize;

pub type Result<T> = core::result::Result<T, Error>;

/// The version of the SCPI standard this crate implements.
pub const SCPI_STD_VERSION: &str = "1999.0";

/// The maximum number of arguments that can be passed to a command.
pub const MAX_ARGS: usize = 10;

#[cfg(feature = "embedded-io-async")]
/// The size of the output buffer used for the embedded io handler.
pub const OUTPUT_BUFFER_SIZE: usize = 100;

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct ReadmeDoctests;