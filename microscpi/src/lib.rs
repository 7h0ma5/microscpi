//! This crate provides a simple interface to create an async SCPI command
//! interpreter. It is especially suited for embedded devices The SCPI command
//! tree is created at compile time using a procedural macro.
//!
//! Notable features are:
//! * No dynamic memory allocation used and *no_std* support.
//! * Compile time creation of the SCPI command tree.
//! * Support for `async` commmand handler functions.
//!
//! # Usage
//!
//! To build an SCPI interface, create a struct that implements the
//! SCPI handler commands. The `#[interface]` attribute macro attached to the
//! `impl` of the interface will generate the command tree and implement the
//! functions required by the [Interface] trait. The [Interface::run] method
//! will parse the command string, execute the corresponding command handler
//! function and fill the output buffer with the response of the command
//! handler function.
//!
//! ## Minimal Example
//!
//! The following example demonstrates how to create a simple SCPI interface
//! with a single command that returns a value.
//!
//! ```
//! use microscpi::{self as scpi, Interface};
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
//!     let mut output = Vec::new();
//!     let mut interface = ExampleInterface { value: 42 };
//!
//!     interface.run(b"SYSTEM:VAL?\n", &mut output).await;
//!
//!     assert_eq!(output, b"42\n");
//! }
//! ```
//!
//! ## Using standard command handlers
//!
//! This crate provides a set of standard command handlers that can be used to
//! implement the SCPI standard commands. The `StandardCommands` trait provides
//! a set of default implementations for generic SCPI commands required by the
//! IEEE 488.2 standard. The `ErrorCommands` trait provides the default error
//! handling commands. These traits can be implemented for the interface struct
//! to provide a default implementations for these commands.
//!
//! The following example demonstrates how to use the `StandardCommands` and
//! `ErrorCommands` traits to add the SCPI standard commands.
//!
//! ```
//! use microscpi::{self as scpi, Interface};
//!
//! pub struct ExampleInterface {
//!     value: u64,
//!     errors: scpi::StaticErrorQueue<10>,
//! }
//!
//! impl scpi::ErrorCommands for ExampleInterface {
//!     fn error_queue(&mut self) -> &mut impl scpi::ErrorQueue {
//!         &mut self.errors
//!     }
//! }
//!
//! impl scpi::StandardCommands for ExampleInterface {}
//!
//! #[scpi::interface(StandardCommands, ErrorCommands)]
//! impl ExampleInterface {
//!     #[scpi(cmd = "SYSTem:VALue?")]
//!     async fn system_value(&mut self) -> Result<u64, scpi::Error> {
//!         Ok(self.value)
//!     }
//! }
//!
//! #[tokio::main]
//! pub async fn main() {
//!     let mut output = Vec::new();
//!     let mut interface = ExampleInterface { value: 42, errors: scpi::StaticErrorQueue::new() };
//!
//!     interface.run(b"UNKNOWN:COMMAND?\n", &mut output).await;
//!     interface.run(b"SYST:ERROR:NEXT?\n", &mut output).await;
//!
//!     assert_eq!(output, b"-113,\"Undefined header\"\n");
//! }
//! ```
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(async_fn_in_trait)]
#![allow(clippy::from_str_radix_10)]

#[cfg(feature = "std")]
extern crate std as core;

mod commands;
mod error;
mod error_queue;
mod interface;
#[doc(hidden)]
pub mod parser;
mod response;
#[doc(hidden)]
pub mod tree;
mod value;

pub use commands::{ErrorCommands, StandardCommands};
pub use error::Error;
pub use error_queue::{ErrorQueue, StaticErrorQueue};
pub use interface::{ErrorHandler, Interface, Adapter};
pub use microscpi_macros::interface;
pub use response::{Arbitrary, Characters, Response, Write};
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

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct ReadmeDoctests;
