//! This module containts implementations of SCPI standard commands.
use crate::{Characters, Error, ErrorHandler, ErrorQueue, SCPI_STD_VERSION};

#[cfg(feature = "registers")]
use crate::registers::{EventStatus, StatusByte, StatusRegisters};

/// Error Commands
///
/// The [ErrorCommands] trait implements the standard SCPI commands used for
/// error management. The only requirement to implement this trait is to provide
/// an [ErrorQueue] via the [ErrorCommands::error_queue()] method. This crate
/// contains an implementation of an error queue based on a statically allocated
/// data structure: [crate::StaticErrorQueue].
//
/// # Implemented commands
///
/// * `SYSTem:ERRor:[NEXT]?`
/// * `SYSTem:ERRor:[COUNt]?`
pub trait ErrorCommands {
    fn error_queue(&mut self) -> &mut impl ErrorQueue;

    fn system_error_count(&mut self) -> Result<usize, Error> {
        Ok(self.error_queue().error_count())
    }

    fn system_error_next(&mut self) -> Result<(i16, &'static str), Error> {
        if let Some(error) = self.error_queue().pop_error() {
            Ok((error.number(), error.into()))
        } else {
            Ok((0, ""))
        }
    }
}

impl<I> ErrorHandler for I
where
    I: ErrorCommands,
{
    fn handle_error(&mut self, error: Error) {
        self.error_queue().push_error(error);
    }
}

/// Standard Commands
///
/// # Implemented commands
///
/// * `SYSTem:VERSion?`
pub trait StandardCommands {
    fn system_version(&mut self) -> Result<Characters, Error> {
        Ok(Characters(SCPI_STD_VERSION))
    }
}

/// Status Commands
///
/// # Implemented commands
/// * `*ESR?`
/// * `*ESE?`
/// * `*ESE`
/// * `*CLS`
/// * `*STB`
/// * `*SRE`
/// * `*SRE?`
/// * `*OPC?`
/// * `*OPC`
#[cfg(feature = "registers")]
pub trait StatusCommands: ErrorCommands {
    fn status_registers(&mut self) -> &mut StatusRegisters;

    fn operation_complete(&mut self) -> Result<bool, Error> {
        self.set_operation_complete()?;

        Ok(self
            .status_registers()
            .event_status
            .contains(EventStatus::OPERATION_COMPLETE))
    }

    fn set_operation_complete(&mut self) -> Result<(), Error> {
        self.status_registers()
            .event_status
            .set(EventStatus::OPERATION_COMPLETE, true);
        Ok(())
    }

    fn clear_event_status(&mut self) -> Result<(), Error> {
        self.error_queue().clear();
        self.status_registers().event_status = EventStatus::empty();
        Ok(())
    }

    fn event_status_enable(&mut self) -> Result<u8, Error> {
        Ok(self.status_registers().event_status_enable.bits())
    }

    fn set_event_status_enable(&mut self, value: u8) -> Result<(), Error> {
        self.status_registers().event_status = EventStatus::from_bits_retain(value);
        Ok(())
    }

    fn event_status_register(&mut self) -> Result<u8, Error> {
        let value = self.status_registers().event_status.bits();
        let mask = self.status_registers().event_status_enable.bits();
        Ok(value & mask)
    }

    fn status_byte(&mut self) -> Result<u8, Error> {
        let mut status: StatusByte = StatusByte::empty();

        if self.error_queue().error_count() > 0 {
            status.insert(StatusByte::ERROR_EVENT_QUEUE);
        }

        if self.event_status_register()? != 0 {
            status.insert(StatusByte::STANDARD_EVENT);
        }

        Ok(status.bits())
    }
}
