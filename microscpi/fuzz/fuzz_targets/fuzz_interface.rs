#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use microscpi::{
    self as scpi, ErrorCommands, ErrorQueue, Interface, StandardCommands, StaticErrorQueue,
};
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let runtime = RUNTIME.get_or_init(|| Runtime::new().unwrap());
    let mut interface = TestInterface {
        errors: StaticErrorQueue::new(),
        result: None,
    };
    let mut output = Vec::new();
    let _ = runtime.block_on(interface.run(data, &mut output));
});

#[derive(Debug, PartialEq)]
pub enum TestResult {
    ResetOk,
    IdnOk,
    TestA,
    TestAQ,
}

pub struct TestInterface {
    errors: StaticErrorQueue<10>,
    result: Option<TestResult>,
}

impl ErrorCommands for TestInterface {
    fn error_queue(&mut self) -> &mut impl ErrorQueue {
        &mut self.errors
    }
}

impl StandardCommands for TestInterface {}

#[scpi::interface(StandardCommands, ErrorCommands)]
impl TestInterface {
    #[scpi(cmd = "*RST")]
    pub async fn rst(&mut self) -> Result<(), scpi::Error> {
        self.result = Some(TestResult::ResetOk);
        Ok(())
    }

    #[scpi(cmd = "*IDN?")]
    pub async fn idn(&mut self) -> Result<&str, scpi::Error> {
        self.result = Some(TestResult::IdnOk);
        Ok("MICROSCPI,TEST,1,1.0")
    }

    #[scpi(cmd = "VALue:STRing?")]
    pub async fn value_str(&mut self) -> Result<&str, scpi::Error> {
        Ok("Hello World")
    }

    #[scpi(cmd = "[SYSTem]:TeST:A")]
    pub async fn system_test_a(&mut self) -> Result<(), scpi::Error> {
        self.result = Some(TestResult::TestA);
        Ok(())
    }

    #[scpi(cmd = "[SYSTem]:TeST:A?")]
    pub async fn system_test_aq(&mut self) -> Result<(), scpi::Error> {
        self.result = Some(TestResult::TestAQ);
        Ok(())
    }

    #[scpi(cmd = "MATH:OPeration:MULTiply?")]
    pub async fn math_multiply(&mut self, a: u64, b: u64) -> Result<u64, scpi::Error> {
        Ok(a * b)
    }

    #[scpi(cmd = "MATH:OPeration:MULTiplyFloat?")]
    pub async fn math_multiply_float(&mut self, a: f64, b: f64) -> Result<f64, scpi::Error> {
        Ok(a * b)
    }
}
