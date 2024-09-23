use microscpi::Interpreter;
use microscpi as scpi;

#[derive(Debug, PartialEq)]
pub enum TestResult {
    ResetOk,
    IdnOk,
    TestA,
    TestAQ,
}

pub struct TestInterface {
    result: Option<TestResult>,
}

#[scpi::interface]
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
}

fn setup() -> (Interpreter<TestInterface>, String) {
    let interface = Interpreter::new(TestInterface { result: None });
    (interface, String::new())
}

#[tokio::test]
async fn test_idn() {
    let (mut interpreter, mut output) = setup();
    interpreter.parse_and_execute(b"*IDN?\n", &mut output).await;
    assert_eq!(interpreter.interface.result, Some(TestResult::IdnOk));
}

#[tokio::test]
async fn test_rst() {
    let (mut interpreter, mut output) = setup();
    interpreter.parse_and_execute(b"*RST\n", &mut output).await;
    assert_eq!(interpreter.interface.result, Some(TestResult::ResetOk));
}

#[tokio::test]
async fn test_a_short() {
    let (mut interpreter, mut output) = setup();
    interpreter.parse_and_execute(b"TST:A\n", &mut output).await;
    assert_eq!(interpreter.interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_a_long() {
    let (mut interpreter, mut output) = setup();
    interpreter
        .parse_and_execute(b"SYSTEM:TEST:A\r\n", &mut output)
        .await;
    assert_eq!(interpreter.interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_value_string() {
    let (mut interpreter, mut output) = setup();

    interpreter.parse_and_execute(b"VAL:STR?\n", &mut output).await;

    assert_eq!(output, "\"Hello World\"\n");
}

#[tokio::test]
async fn test_terminators() {
    let (mut interpreter, mut output) = setup();

    assert_eq!(
        interpreter.parse_and_execute(b"*IDN?\r\n", &mut output).await,
        None
    );
    assert_eq!(interpreter.parse_and_execute(b"*IDN?\n", &mut output).await, None);
    assert_eq!(
        interpreter.parse_and_execute(b"*IDN?\r\n", &mut output).await,
        None
    );
    assert_eq!(
        interpreter.parse_and_execute(b"*IDN?\n\r", &mut output).await,
        Some(&[b'\r'] as &[u8])
    );
}

#[tokio::test]
async fn test_invalid_command() {
    let (mut interpreter, mut output) = setup();

    interpreter.parse_and_execute(b"*IDN\n", &mut output).await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(interpreter.context.pop_error(), None);

    interpreter.parse_and_execute(b"FOO\n", &mut output).await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(interpreter.context.pop_error(), None);

    interpreter.parse_and_execute(b"FOO:BAR\n", &mut output).await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(interpreter.context.pop_error(), None);

    interpreter.parse_and_execute(b"SYST:FOO\n", &mut output).await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(interpreter.context.pop_error(), None);
}

#[tokio::test]
async fn test_invalid_character() {
    let (mut interpreter, mut output) = setup();

    interpreter
        .parse_and_execute("*IDN!\n".as_bytes(), &mut output)
        .await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::InvalidCharacter));
}

#[tokio::test]
async fn test_arguments() {
    let (mut interpreter, mut output) = setup();
    interpreter
        .parse_and_execute(b"MATH:OP:MULT? 123,456\n", &mut output)
        .await;
    assert_eq!(output, "56088\n");
}

#[tokio::test]
async fn test_invalid_arguments() {
    let (mut interpreter, mut output) = setup();

    interpreter
        .parse_and_execute(b"SYSTEM:TEST:A 123 456\n", &mut output)
        .await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::InvalidSeparator));

    interpreter
        .parse_and_execute(b"SYSTEM:TEST:A 123,,456\n", &mut output)
        .await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::InvalidSeparator));

    interpreter
        .parse_and_execute(b"SYSTEM:TEST:A ,123\n", &mut output)
        .await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::InvalidSeparator));

    interpreter
        .parse_and_execute(b"SYSTEM:TEST:A,123\n", &mut output)
        .await;
    assert_eq!(interpreter.context.pop_error(), Some(scpi::Error::InvalidSeparator));

    assert_eq!(interpreter.context.pop_error(), None);
}