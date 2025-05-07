use microscpi::{self as scpi, ErrorHandler, Interface, StandardCommands};

pub struct ExampleInterface {
    value: u64,
}

impl ErrorHandler for ExampleInterface {
    fn handle_error(&mut self, error: scpi::Error) {
        println!("Error: {error}");
    }
}

#[microscpi::interface(StandardCommands)]
impl ExampleInterface {
    #[scpi(cmd = "SYSTem:VALue?")]
    async fn system_value(&mut self) -> Result<u64, scpi::Error> {
        Ok(self.value)
    }
}

impl StandardCommands for ExampleInterface {}

#[tokio::main]
pub async fn main() {
    let mut output = Vec::new();
    let mut interface = ExampleInterface { value: 42 };

    interface.run(b"SYSTEM:VAL?\n", &mut output).await;

    assert_eq!(output, b"42\n");
}
