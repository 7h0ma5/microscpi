use microscpi::{self as scpi, Interface};

mod calibration;
mod gpio;

pub struct Device {
    errors: scpi::StaticErrorQueue<10>,
    gpio: [bool; 4],
    calibration: f64,
}

impl scpi::ErrorCommands for Device {
    fn error_queue(&mut self) -> &mut impl scpi::ErrorQueue {
        &mut self.errors
    }
}

impl scpi::StandardCommands for Device {}

#[scpi::interface(StandardCommands, ErrorCommands, modules(calibration, gpio))]
impl Device {
    #[scpi(cmd = "*IDN?")]
    fn identify(&mut self) -> Result<scpi::Characters<'_>, scpi::Error> {
        Ok(scpi::Characters("ACME,Device,1234,1.0"))
    }
}

#[tokio::main]
async fn main() {
    let mut device = Device {
        errors: scpi::StaticErrorQueue::new(),
        gpio: [false; 4],
        calibration: 1.0,
    };

    let mut output = heapless::Vec::<u8, 256>::new();
    device.run(b"*IDN?\n", &mut output).await;
    device.run(b"GPIO:STATe 2,1\n", &mut output).await;
    device.run(b"GPIO:STATe? 2\n", &mut output).await;
    device.run(b"CAL:GAIN 2.5\n", &mut output).await;
    device.run(b"CAL:GAIN?\n", &mut output).await;
    device.run(b"SYST:ERR:COUN?\n", &mut output).await;

    print!("{}", String::from_utf8_lossy(&output));
}
