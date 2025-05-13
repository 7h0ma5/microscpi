use microscpi::{self, ErrorHandler};

pub struct TestInterface {}

impl ErrorHandler for TestInterface {
    fn handle_error(&mut self, _error: microscpi::Error) {
        unreachable!()
    }
}

#[microscpi::interface]
impl TestInterface {
    /// Returns the device identifier.
    ///
    /// ```yaml
    /// unit: string
    /// example: "ACME,Widget3000,1234,v1.02"
    /// ```
    #[scpi(cmd = "*IDN?")]
    fn identify(&mut self) -> Result<&str, microscpi::Error> {
        Ok("TEST,DEVICE,1234,1.0")
    }

    /// Sets a measurement parameter.
    ///
    /// ```yaml
    /// args:
    ///   - name: value
    ///     type: float
    ///     unit: volts
    ///     range: [0.0, 10.0]
    /// ```
    #[scpi(cmd = "MEASure:VOLTage")]
    fn set_voltage(&mut self, value: f32) -> Result<(), microscpi::Error> {
        // Check if the value is within the valid range
        if !(0.0..=10.0).contains(&value) {
            return Err(microscpi::Error::DataOutOfRange);
        }
        Ok(())
    }

    /// Queries the current measurement value.
    ///
    /// This command returns the most recent measurement value.
    #[scpi(cmd = "MEASure:VOLTage?")]
    fn get_voltage(&mut self) -> Result<f32, microscpi::Error> {
        Ok(5.0) // Return a fixed value for testing
    }
}
