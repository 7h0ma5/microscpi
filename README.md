# microSCPI

[![build](https://github.com/7h0ma5/microscpi/workflows/build/badge.svg)](https://github.com/7h0ma5/microscpi/actions)
[![Latest version](https://img.shields.io/crates/v/microscpi.svg)](https://crates.io/crates/microscpi)
[![Documentation](https://img.shields.io/docsrs/microscpi)](https://docs.rs/microscpi)
[![Codecov](https://img.shields.io/codecov/c/github/7h0ma5/microscpi)](https://codecov.io/github/7h0ma5/microscpi)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A lightweight, zero-allocation async SCPI command interpreter for Rust, optimized for embedded systems.

## 📋 Overview

microSCPI provides a simple yet powerful interface for creating an asynchronous SCPI (Standard Commands for Programmable Instruments) command interpreter. It's specifically designed for embedded devices where resource constraints are a primary concern. The library enables developers to easily implement SCPI communication protocols with minimal overhead.

## ✨ Key Features

- **Zero Heap Allocations**: Operates without dynamic memory allocation, perfect for memory-constrained embedded systems
- **Compile-Time Command Tree**: Efficient command processing with command definitions resolved at compile time
- **Async Command Handling**: Full support for asynchronous command handlers, enabling non-blocking operations
- **Type-Safe Responses**: Return values are automatically formatted according to their type
- **Minimal Dependencies**: Keeps your project's dependency tree lean and build times fast
- **Robust Error Handling**: Comprehensive error reporting and recovery mechanisms

## 🚀 Getting Started

### Installation

Add microSCPI to your project by including it in your `Cargo.toml`:

```toml
[dependencies]
microscpi = "0.5.0"
```

If you need async functionality, make sure to include an async runtime like `tokio`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### Basic Example

Here's a minimal example demonstrating how to use microSCPI to create an SCPI command interface:

```rust
use microscpi::Interface;

pub struct BasicInterface {}

impl microscpi::ErrorHandler for BasicInterface {
    fn handle_error(&mut self, error: microscpi::Error) {
        println!("Error: {error}");
    }
}

#[microscpi::interface]
impl BasicInterface {
    #[scpi(cmd = "MATH:MULTiply?")]
    async fn system_value(&mut self, a: f64, b: f64) -> Result<f64, microscpi::Error> {
        Ok(a * b)
    }
}

#[tokio::main]
pub async fn main() {
    let mut output = Vec::new();
    let mut interface = BasicInterface {};

    interface.run(b"MATH:MULT? 23, 42\n", &mut output).await;

    assert_eq!(output, b"966\n");
}
```

## 📖 Documentation

For comprehensive documentation, visit [docs.rs/microscpi](https://docs.rs/microscpi).

## 🔧 Project Structure

microSCPI is organized as a workspace with the following components:

- `microscpi`: The core library implementation
- `microscpi-macros`: Procedural macros for the `#[microscpi::interface]` and `#[scpi]` attributes

## 👥 Contributing

Contributions are welcome! Feel free to submit issues or pull requests on the [GitHub repository](https://github.com/7h0ma5/microscpi).

## 📄 License

This project is licensed under the [MIT License](LICENSE).
