[![build](https://github.com/7h0ma5/microscpi/workflows/build/badge.svg)](https://github.com/7h0ma5/microscpi/actions)
[![Latest version](https://img.shields.io/crates/v/microscpi.svg)](https://crates.io/crates/microscpi)
[![Documentation](https://img.shields.io/docsrs/microscpi)](https://docs.rs/microscpi)
[![Codecov](https://img.shields.io/codecov/c/github/7h0ma5/microscpi)](https://codecov.io/github/7h0ma5/microscpi)

# `microscpi` Crate

This crate provides a simple interface to create an async SCPI (Standard Commands for Programmable Instruments) command interpreter, particularly well-suited for embedded devices. It aims to offer a lightweight, efficient, and easily extensible solution for parsing and handling SCPI commands without requiring heap memory allocations.

## Features

- **No heap memory allocation required**: The crate is designed to function without the need for dynamic memory allocation, making it ideal for constrained embedded systems.
- **Compile-time SCPI command tree creation**: Commands are defined at compile-time, ensuring efficiency and reducing runtime overhead.
- **Support for async command handler functions**: Asynchronous command handling is fully supported, allowing non-blocking operations in embedded or other concurrent environments.

## Example

The following is a minimal example demonstrating how to use the `microscpi` crate to create an SCPI command interface that handles the `SYSTem:VALue?` command asynchronously.

```rust
use microscpi::{self as scpi, Interface, ErrorHandler};

pub struct ExampleInterface {
    value: u64
}

impl ErrorHandler for ExampleInterface {
    fn handle_error(&mut self, error: scpi::Error) {
        println!("Error: {error}");
    }
}

#[microscpi::interface]
impl ExampleInterface {
    #[scpi(cmd = "SYSTem:VALue?")]
    async fn system_value(&mut self) -> Result<u64, scpi::Error> {
        Ok(self.value)
    }
}

#[tokio::main]
pub async fn main() {
    let mut output = String::new();
    let mut interface = ExampleInterface { value: 42 };

    interface.parse_and_execute(b"SYSTEM:VAL?\n", &mut output).await;

    assert_eq!(output, "42\n");
}
```

## Crate Usage

To use this crate in your project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
microscpi = "0.1.0-alpha.3"
```

Make sure to include the async runtime such as `tokio` or another suitable runtime for executing async functions. 

Example for adding `tokio`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

## License

This project is licensed under the MIT License.