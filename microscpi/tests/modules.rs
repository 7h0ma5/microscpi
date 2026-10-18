use microscpi::{self as scpi, Interface};

pub struct TestInterface {
    value: u32,
}

impl scpi::ErrorHandler for TestInterface {
    fn handle_error(&mut self, error: scpi::Error) {
        panic!("Unexpected error: {error}");
    }
}

mod first {
    use super::*;

    #[scpi::commands]
    impl TestInterface {
        #[scpi(cmd = "FIRSt:VALue")]
        pub(crate) fn set_value(&mut self, value: u32) -> Result<(), scpi::Error> {
            self.value = value;
            Ok(())
        }
    }
}

mod nested {
    pub mod second {
        use crate::*;

        #[scpi::commands]
        impl TestInterface {
            #[scpi(cmd = "SECond:VALue?")]
            pub(crate) async fn value(&mut self) -> Result<u32, scpi::Error> {
                Ok(self.value)
            }
        }
    }
}

#[scpi::interface(modules(first, nested::second))]
impl TestInterface {
    #[scpi(cmd = "MAIN?")]
    fn main(&mut self) -> Result<u32, scpi::Error> {
        Ok(1)
    }
}

#[tokio::test]
async fn test_modules() {
    let mut interface = TestInterface { value: 0 };
    let mut output = heapless::Vec::<u8, 64>::new();

    interface.run(b"MAIN?\n", &mut output).await;
    interface.run(b"FIRS:VAL 42\n", &mut output).await;
    interface.run(b"SEC:VAL?\n", &mut output).await;

    assert_eq!(output, b"1\n42\n");
}

mod empty {
    use microscpi::{self as scpi, Interface};

    pub struct EmptyInterface;

    impl scpi::ErrorHandler for EmptyInterface {
        fn handle_error(&mut self, _error: scpi::Error) {}
    }

    #[scpi::interface]
    impl EmptyInterface {}

    #[tokio::test]
    async fn test_empty_interface() {
        let mut interface = EmptyInterface;
        let mut output = heapless::Vec::<u8, 64>::new();

        interface.run(b"MAIN?\n", &mut output).await;

        assert!(output.is_empty());
    }
}
