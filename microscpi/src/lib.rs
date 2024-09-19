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
//!
//!     let mut context = scpi::Context::new(interface);
//!     context.process_buffer(b"SYSTEM:VAL?\n", &mut output).await;
//!
//!     assert_eq!(output, "42\n");
//! }
//! ```
#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]
#![allow(clippy::from_str_radix_10)]

mod context;
mod error;
mod parser;
pub mod tokens;
mod tree;
mod value;

pub use context::Context;
pub use error::Error;
pub use microscpi_macros::interface;
pub use tree::{CommandId, ScpiTreeNode};
pub use value::Value;

#[doc(hidden)]
pub trait Interface<'i> {
    fn root_node() -> &'static ScpiTreeNode;
    async fn run_command(
        &'i mut self, command_id: CommandId, args: &[Value<'i>],
    ) -> Result<Value<'i>, Error>;
}
