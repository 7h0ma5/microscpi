//! This module containts implementations of SCPI standard commands.
use crate::{Error, ErrorHandler, ErrorQueue, Mnemonic, SCPI_STD_VERSION};

/// Error Commands
///
/// The [ErrorCommands] trait implements the standard SCPI commands used for
/// error management. The only requirement to implement this trait is to provide
/// an [ErrorQueue] via the [ErrorCommands::error_queue] method. This crate
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
        }
        else {
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
    fn system_version(&mut self) -> Result<Mnemonic, Error> {
        Ok(Mnemonic(SCPI_STD_VERSION))
    }
}
