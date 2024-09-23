use heapless::Deque;

use crate::{Error, MAX_ERRORS};

/// SCPI Context
///
/// The SCPI context contains the current state of the SCPI interface including
/// the parser state, the current selected node in the SCPI command tree and the
/// error queue.
pub struct Context {
    errors: heapless::Deque<Error, MAX_ERRORS>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            errors: Deque::new(),
        }
    }

    pub fn pop_error(&mut self) -> Option<Error> {
        self.errors.pop_front()
    }

    pub fn push_error(&mut self, error: Error) {
        #[cfg(feature = "defmt")]
        defmt::trace!("Push Error: {}", error);
        if self.errors.push_back(error).is_err() {
            // If the queue is full, change the most recent added item to an *Queue
            // Overflow* error, as specified in IEEE 488.2, 21.8.1.
            if let Some(value) = self.errors.back_mut() {
                *value = Error::QueueOverflow;
            }
        }
    }
}

impl Context {
    pub async fn system_error_next(&mut self) -> Result<i32, Error> {
        if let Some(error) = self.pop_error() {
            Ok(error.number() as i32)
        }
        else {
            Ok(0)
        }
    }
}