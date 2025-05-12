// This is a test module to verify that the SCPI command documentation export works.
// It uses the export attribute to generate documentation during build time.
// Note: The export attribute syntax is export = "filename.json"

use microscpi;

use microscpi::ErrorHandler;
// Import the macros
use microscpi_macros::interface;

// Define a simple interface for testing
struct TestInterface {}

impl ErrorHandler for TestInterface {
    fn handle_error(&mut self, _error: microscpi::Error) {
        unreachable!()
    }
}

#[interface(export = "target/scpi_commands.json")]
impl TestInterface {
    /// Returns the device identifier.
    ///
    /// ```yaml
    /// unit: string
    /// example: "ACME,Widget3000,1234,v1.02"
    /// ```
    #[scpi(cmd = "*IDN?")]
    fn identify(&mut self) -> Result<String, microscpi::Error> {
        Ok("TEST,DEVICE,1234,1.0".to_string())
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

// This isn't a real test since we can't easily verify the exported content
// without adding dependencies. Instead, it serves as a compile-time check
// and also generates the documentation file during the build process.
#[test]
fn verify_export_compiles() {
    // This test doesn't actually verify anything, but ensures the code compiles
    let _interface = TestInterface {};
}
