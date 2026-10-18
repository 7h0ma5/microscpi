use microscpi as scpi;

use crate::Device;

#[scpi::commands]
impl Device {
    #[scpi(cmd = "CALibration:GAIN")]
    pub(crate) async fn set_calibration_gain(&mut self, gain: f64) -> Result<(), scpi::Error> {
        self.calibration = gain;
        Ok(())
    }

    #[scpi(cmd = "CALibration:GAIN?")]
    pub(crate) async fn calibration_gain(&mut self) -> Result<f64, scpi::Error> {
        Ok(self.calibration)
    }
}
