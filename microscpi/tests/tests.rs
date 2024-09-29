use microscpi::{self as scpi, ErrorQueue, StaticErrorQueue, Interface};
use microscpi::commands::{ErrorCommands, StandardCommands};

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
    pub async fn idn(&mut self) -> Result<(), scpi::Error> {
        self.result = Some(TestResult::IdnOk);
        Ok(())
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

fn setup() -> (TestInterface, String) {
    let interface = TestInterface { errors: StaticErrorQueue::new(), result: None };
    (interface, String::new())
}

#[tokio::test]
async fn test_idn() {
    let (mut interface, mut output) = setup();
    interface.parse_and_execute(b"*IDN?\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::IdnOk));
}

#[tokio::test]
async fn test_rst() {
    let (mut interface, mut output) = setup();
    interface.parse_and_execute(b"*RST\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::ResetOk));
}

#[tokio::test]
async fn test_a_short() {
    let (mut interface, mut output) = setup();
    interface.parse_and_execute(b"TST:A\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_a_long() {
    let (mut interface, mut output) = setup();
    interface
        .parse_and_execute(b"SYSTEM:TEST:A\r\n", &mut output)
        .await;
    assert_eq!(interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_value_string() {
    let (mut interface, mut output) = setup();

    interface
        .parse_and_execute(b"VAL:STR?\n", &mut output)
        .await;

    assert_eq!(output, "\"Hello World\"\n");
}

#[tokio::test]
async fn test_terminators() {
    let (mut interface, mut output) = setup();

    assert_eq!(
        interface.parse_and_execute(b"*IDN?\n", &mut output).await,
        &[][..]
    );
    assert_eq!(
        interface
            .parse_and_execute(b"*IDN?\r\n", &mut output)
            .await,
        &[][..]
    );
    assert_eq!(
        interface
            .parse_and_execute(b"*IDN?\n\r", &mut output)
            .await,
        &[b'\r'][..]
    );
}

#[tokio::test]
async fn test_invalid_command() {
    let (mut interface, mut output) = setup();

    interface.parse_and_execute(b"*IDN\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);

    interface.parse_and_execute(b"FOO\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);

    interface
        .parse_and_execute(b"FOO:BAR\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);

    interface
        .parse_and_execute(b"SYST:FOO\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);
}

#[tokio::test]
async fn test_invalid_character() {
    let (mut interface, mut output) = setup();

    interface
        .parse_and_execute("*IDN!\n".as_bytes(), &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );
}

#[tokio::test]
async fn test_arguments() {
    let (mut interface, mut output) = setup();
    interface
        .parse_and_execute(b"MATH:OP:MULT? 123,456\n", &mut output)
        .await;
    assert_eq!(output, "56088\n");

    let (mut interface, mut output) = setup();
    interface
        .parse_and_execute(b"MATH:OP:MULT? #H7B,#Q710\n", &mut output)
        .await;
    assert_eq!(output, "56088\n");
}

#[tokio::test]
async fn test_float() {
    let (mut interface, mut output) = setup();
    interface
        .parse_and_execute(b"MATH:OP:MULTF? 23.42,42.23\n", &mut output)
        .await;
    assert_eq!(output, "989.0266\n");
}

#[tokio::test]
async fn test_invalid_arguments() {
    let (mut interface, mut output) = setup();

    interface
        .parse_and_execute(b"SYSTEM:TEST:A 123 456\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    interface
        .parse_and_execute(b"SYSTEM:TEST:A 123,,456\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    interface
        .parse_and_execute(b"SYSTEM:TEST:A ,123\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    interface
        .parse_and_execute(b"SYSTEM:TEST:A,123\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    assert_eq!(interface.errors.pop_error(), None);
}

#[tokio::test]
async fn test_next_error() {
    let (mut interface, mut output) = setup();

    interface.errors.push_error(scpi::Error::SystemError);

    interface
        .parse_and_execute(b"SYST:ERR:NEXT?\n", &mut output)
        .await;

    assert_eq!(output, "-310,\"System error\"\n");

    output.clear();

    interface
        .parse_and_execute(b"SYST:ERR:NEXT?\n", &mut output)
        .await;

    assert_eq!(output, "0,\"\"\n");
}