use microscpi::{
    self as scpi, ErrorCommands, ErrorQueue, Interface, StandardCommands, StaticErrorQueue,
};

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

fn setup() -> (TestInterface, String) {
    let interface = TestInterface {
        errors: StaticErrorQueue::new(),
        result: None,
    };
    (interface, String::new())
}

#[tokio::test]
async fn test_idn() {
    let (mut interface, mut output) = setup();
    interface.run(b"*IDN?\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::IdnOk));
}

#[tokio::test]
async fn test_rst() {
    let (mut interface, mut output) = setup();
    interface.run(b"*RST\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::ResetOk));
}

#[tokio::test]
async fn test_a_short() {
    let (mut interface, mut output) = setup();
    interface.run(b"TST:A\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_a_long() {
    let (mut interface, mut output) = setup();
    interface.run(b"SYSTEM:TEST:A\r\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_system_test_aq() {
    let (mut interface, mut output) = setup();
    interface.run(b"SYST:TEST:A?\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::TestAQ));
}

#[tokio::test]
async fn test_value_string() {
    let (mut interface, mut output) = setup();

    interface.run(b"VAL:STR?\n", &mut output).await;

    assert_eq!(output, "\"Hello World\"\n");
}

#[tokio::test]
async fn test_terminators() {
    let (mut interface, mut output) = setup();

    assert_eq!(interface.run(b"*IDN?\n", &mut output).await, &[][..]);
    assert_eq!(interface.run(b"*IDN?\r\n", &mut output).await, &[][..]);
    assert_eq!(interface.run(b"*IDN?\n\r", &mut output).await, &[b'\r'][..]);
}

#[tokio::test]
async fn test_invalid_command() {
    let (mut interface, mut output) = setup();

    interface.run(b"*IDN\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);

    interface.run(b"FOO\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);

    interface.run(b"FOO:BAR\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);

    interface.run(b"SYST:FOO\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::UndefinedHeader)
    );
    assert_eq!(interface.errors.pop_error(), None);
}

#[tokio::test]
async fn test_invalid_character() {
    let (mut interface, mut output) = setup();

    interface.run("*IDN!\n".as_bytes(), &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );
}
#[tokio::test]
async fn test_math_multiply() {
    let (mut interface, mut output) = setup();
    interface.run(b"MATH:OP:MULT? 7,6\n", &mut output).await;
    assert_eq!(output, "42\n");
}

#[tokio::test]
async fn test_math_multiply_float() {
    let (mut interface, mut output) = setup();
    interface
        .run(b"MATH:OP:MULTF? 23.42,42.23\n", &mut output)
        .await;
    assert_eq!(output, "989.0266\n");
}

#[tokio::test]
async fn test_math_multiply_hexadecimal() {
    let (mut interface, mut output) = setup();
    interface
        .run(b"MATH:OP:MULT? #H7B,#Q710\n", &mut output)
        .await;
    assert_eq!(output, "56088\n");
}

#[tokio::test]
async fn test_invalid_arguments() {
    let (mut interface, mut output) = setup();

    interface.run(b"SYSTEM:TEST:A 123 456\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    interface
        .run(b"SYSTEM:TEST:A 123,,456\n", &mut output)
        .await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    interface.run(b"SYSTEM:TEST:A ,123\n", &mut output).await;
    assert_eq!(
        interface.errors.pop_error(),
        Some(scpi::Error::InvalidCharacter)
    );

    interface.run(b"SYSTEM:TEST:A,123\n", &mut output).await;
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

    interface.run(b"SYST:ERR:NEXT?\n", &mut output).await;

    assert_eq!(output, "-310,\"System error\"\n");

    output.clear();

    interface.run(b"SYST:ERR:NEXT?\n", &mut output).await;

    assert_eq!(output, "0,\"\"\n");
}

#[tokio::test]
async fn test_value_string_with_whitespace() {
    let (mut interface, mut output) = setup();
    interface.run(b"  VAL:STR?  \n", &mut output).await;
    assert_eq!(output, "\"Hello World\"\n");
}

#[tokio::test]
async fn test_multiple_commands() {
    let (mut interface, mut output) = setup();
    interface.run(b"*RST\n*IDN?\n", &mut output).await;
    assert_eq!(interface.result, Some(TestResult::IdnOk));
    assert_eq!(output, "\"MICROSCPI,TEST,1,1.0\"\n");
}
