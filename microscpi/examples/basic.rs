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
