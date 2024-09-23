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
//!
//! pub struct ExampleInterface {
//!     value: u64
//! }
//!
//! #[scpi::interface]
//! impl ExampleInterface {
//!     #[scpi(cmd = "SYSTem:VALue?")]
//!     pub async fn system_value(&mut self) -> Result<u64, scpi::Error> {
//!         Ok(self.value)
//!     }
//! }
//!
//! #[tokio::main]
//! pub async fn main() {
//!     let mut output = String::new();
//!     let interface = ExampleInterface { value: 42 };
//!     let mut interpreter = scpi::Interpreter::new(interface);
//!
//!     interpreter.parse_and_execute(b"SYSTEM:VAL?\n", &mut output).await;
//!
//!     assert_eq!(output, "42\n");
//! }
//! ```
#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]
#![allow(clippy::from_str_radix_10)]

mod context;
mod error;
mod interface;
mod interpreter;
mod parser;
pub mod tokens;
mod tree;
mod value;

const MAX_ERRORS: usize = 10;
const MAX_ARGS: usize = 10;
#[cfg(feature = "embedded-io-async")]
const OUTPUT_BUFFER_SIZE: usize = 100;

pub use context::Context;
pub use error::Error;
pub use interface::Interface;
pub use interpreter::Interpreter;
pub use microscpi_macros::interface;
pub use tree::{CommandId, Node};
pub use value::Value;
