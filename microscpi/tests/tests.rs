use microscpi::{self as scpi, Context};

#[derive(Debug, PartialEq)]
pub enum TestResult {
    ResetOk,
    IdnOk,
    TestA,
    TestAQ,
}

pub struct TestInterface { result: Option<TestResult> }

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
    let interface = TestInterface {
        result: None,
    };
    (scpi::Context::new(interface), String::new())
}

#[tokio::test]
async fn test_idn() {
    let (mut context, mut output) = setup();
    context.process("*IDN?\r\n".as_bytes(), &mut output).await.unwrap();
    assert_eq!(context.interface.result, Some(TestResult::IdnOk));
}

#[tokio::test]
async fn test_rst() {
    let (mut context, mut output) = setup();
    context.process("*RST\r\n".as_bytes(), &mut output).await.unwrap();
    assert_eq!(context.interface.result, Some(TestResult::ResetOk));
}

#[tokio::test]
async fn test_a_short() {
    let (mut context, mut output) = setup();
    context.process("TST:A\r\n".as_bytes(), &mut output).await.unwrap();
    assert_eq!(context.interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_a_long() {
    let (mut context, mut output) = setup();
    context
        .process("SYSTEM:TEST:A\r\n".as_bytes(), &mut output)
        .await
        .unwrap();
    assert_eq!(context.interface.result, Some(TestResult::TestA));
}

#[tokio::test]
async fn test_value_string() {
    let (mut context, mut output) = setup();
    
    context.process("VAL:STR?\r\n".as_bytes(), &mut output).await.unwrap();

    assert_eq!(output, "\"Hello World\"\n");
}

#[tokio::test]
async fn test_terminators() {
    let (mut context, mut output) = setup();

    assert_eq!(
        context.process("*IDN?\r\n".as_bytes(), &mut output).await,
        Ok(&[] as &[u8])
    );
    assert_eq!(
        context.process("*IDN?\n".as_bytes(), &mut output).await,
        Ok(&[] as &[u8])
    );
    assert_eq!(
        context.process("*IDN?\r\n".as_bytes(), &mut output).await,
        Ok(&[] as &[u8])
    );
    assert_eq!(
        context.process("*IDN?\n\r".as_bytes(), &mut output).await,
        Ok(&[b'\r'] as &[u8])
    );
}

#[tokio::test]
async fn test_invalid_command() {
    let (mut context, mut output) = setup();

    assert_eq!(
        context.process("*IDN\n".as_bytes(), &mut output).await,
        Err(scpi::Error::InvalidCommand)
    );
    assert_eq!(
        context.process("FOO\n".as_bytes(), &mut output).await,
        Err(scpi::Error::InvalidCommand)
    );
    assert_eq!(
        context.process("FOO:BAR\n".as_bytes(), &mut output).await,
        Err(scpi::Error::InvalidCommand)
    );
    assert_eq!(
        context.process("SYST:FOO\n".as_bytes(), &mut output).await,
        Err(scpi::Error::InvalidCommand)
    );
}

#[tokio::test]
async fn test_arguments() {
    let (mut context, mut output) = setup();
    context.process("MATH:OP:MULT? 123,456\n".as_bytes(), &mut output).await.unwrap();
    assert_eq!(output, "56088\n");
}

#[tokio::test]
async fn test_invalid_arguments() {
    let (mut context, mut output) = setup();

    assert_eq!(
        context.process("SYSTEM:TEST:A 123 456\r\n".as_bytes(), &mut output).await,
        Err(scpi::Error::ParseError)
    );

    assert_eq!(
        context.process("SYSTEM:TEST:A 123,,456\r\n".as_bytes(), &mut output).await,
        Err(scpi::Error::ParseError)
    );

    assert_eq!(
        context.process("SYSTEM:TEST:A ,123\r\n".as_bytes(), &mut output).await,
        Err(scpi::Error::ParseError)
    );

    assert_eq!(
        context.process("SYSTEM:TEST:A,123\r\n".as_bytes(), &mut output).await,
        Err(scpi::Error::ParseError)
    );
}
