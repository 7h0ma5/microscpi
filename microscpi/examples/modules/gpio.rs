use microscpi as scpi;

use crate::Device;

#[scpi::commands]
impl Device {
    #[scpi(cmd = "GPIO:STATe")]
    pub(crate) fn set_gpio_state(&mut self, line: usize, state: bool) -> Result<(), scpi::Error> {
        let gpio = self.gpio.get_mut(line).ok_or(scpi::Error::DataOutOfRange)?;
        *gpio = state;
        Ok(())
    }

    #[scpi(cmd = "GPIO:STATe?")]
    pub(crate) fn gpio_state(&mut self, line: usize) -> Result<bool, scpi::Error> {
        self.gpio
            .get(line)
            .copied()
            .ok_or(scpi::Error::DataOutOfRange)
    }
}
