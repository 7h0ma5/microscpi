use microscpi::{self as scpi, Context};

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

fn setup() -> (Context<TestInterface>, String) {
    let interface = TestInterface { result: None };
    (scpi::Context::new(interface), String::new())
}

#[tokio::test]
async fn test_idn() {
    let (mut context, mut output) = setup();
    context.process_buffer(b"*IDN?\n", &mut output).await;
    assert_eq!(context.interface.result, Some(TestResult::IdnOk));
}

#[tokio::test]
async fn test_rst() {
    let (mut context, mut output) = setup();
    context.process_buffer(b"*RST\n", &mut output).await;
    assert_eq!(context.interface.result, Some(TestResult::ResetOk));
}

#[tokio::test]
async fn test_a_short() {
    let (mut context, mut output) = setup();
    context.process_buffer(b"TST:A\n", &mut output).await;
    assert_eq!(context.interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_a_long() {
    let (mut context, mut output) = setup();
    context
        .process_buffer(b"SYSTEM:TEST:A\r\n", &mut output)
        .await;
    assert_eq!(context.interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_value_string() {
    let (mut context, mut output) = setup();

    context.process_buffer(b"VAL:STR?\n", &mut output).await;

    assert_eq!(output, "\"Hello World\"\n");
}

#[tokio::test]
async fn test_terminators() {
    let (mut context, mut output) = setup();

    assert_eq!(
        context.process_buffer(b"*IDN?\r\n", &mut output).await,
        None
    );
    assert_eq!(context.process_buffer(b"*IDN?\n", &mut output).await, None);
    assert_eq!(
        context.process_buffer(b"*IDN?\r\n", &mut output).await,
        None
    );
    assert_eq!(
        context.process_buffer(b"*IDN?\n\r", &mut output).await,
        Some(&[b'\r'] as &[u8])
    );
}

#[tokio::test]
async fn test_invalid_command() {
    let (mut context, mut output) = setup();

    context.process_buffer(b"*IDN\n", &mut output).await;
    assert_eq!(context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(context.pop_error(), None);

    context.process_buffer(b"FOO\n", &mut output).await;
    assert_eq!(context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(context.pop_error(), None);

    context.process_buffer(b"FOO:BAR\n", &mut output).await;
    assert_eq!(context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(context.pop_error(), None);

    context.process_buffer(b"SYST:FOO\n", &mut output).await;
    assert_eq!(context.pop_error(), Some(scpi::Error::UndefinedHeader));
    assert_eq!(context.pop_error(), None);
}

#[tokio::test]
async fn test_invalid_character() {
    let (mut context, mut output) = setup();

    context
        .process_buffer("*IDN!\n".as_bytes(), &mut output)
        .await;
    assert_eq!(context.pop_error(), Some(scpi::Error::InvalidCharacter));
}

#[tokio::test]
async fn test_arguments() {
    let (mut context, mut output) = setup();
    context
        .process_buffer(b"MATH:OP:MULT? 123,456\n", &mut output)
        .await;
    assert_eq!(output, "56088\n");
}

#[tokio::test]
async fn test_invalid_arguments() {
    let (mut context, mut output) = setup();

    context
        .process_buffer(b"SYSTEM:TEST:A 123 456\n", &mut output)
        .await;
    assert_eq!(context.pop_error(), Some(scpi::Error::InvalidSeparator));

    context
        .process_buffer(b"SYSTEM:TEST:A 123,,456\n", &mut output)
        .await;
    assert_eq!(context.pop_error(), Some(scpi::Error::InvalidSeparator));

    context
        .process_buffer(b"SYSTEM:TEST:A ,123\n", &mut output)
        .await;
    assert_eq!(context.pop_error(), Some(scpi::Error::InvalidSeparator));

    context
        .process_buffer(b"SYSTEM:TEST:A,123\n", &mut output)
        .await;
    assert_eq!(context.pop_error(), Some(scpi::Error::InvalidSeparator));

    assert_eq!(context.pop_error(), None);
}
