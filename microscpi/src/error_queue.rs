use crate::Error;

/// An error queue stores the occurred errors until they are queried by the
/// user. It should behave according to the SCPI standard.
pub trait ErrorQueue: Default {
    /// The number of errors currently stored in the error queue.
    fn error_count(&self) -> usize;
    /// Append a new error to the end of the error queue.
    ///
    /// How a queue overflow is handled depends on the implementation of the
    /// queue. IEEE 488.2 defines, that in the event of a queue overflow,
    /// the most recent element in the queue should be replaced by
    /// [Error::QueueOverflow].
    fn push_error(&mut self, error: Error);
    /// Get and remove the error in the front of the error queue. If the queue
    /// is empty, [None] is returned.
    fn pop_error(&mut self) -> Option<Error>;
}

/// An implementation of an [ErrorQueue] utilizing a statically allocated
/// queue holding a maximum of `N` errors.
#[derive(Default)]
pub struct StaticErrorQueue<const N: usize>(heapless::Deque<Error, N>);

impl<const N: usize> StaticErrorQueue<N> {
    pub fn new() -> StaticErrorQueue<N> {
        StaticErrorQueue::default()
    }
}

impl<const N: usize> ErrorQueue for StaticErrorQueue<N> {
    fn push_error(&mut self, error: Error) {
        #[cfg(feature = "defmt")]
        defmt::trace!("Push Error: {}", error);
        if self.0.push_back(error).is_err() {
            // If the queue is full, change the most recent added item to an *Queue
            // Overflow* error, as specified in IEEE 488.2, 21.8.1.
            if let Some(value) = self.0.back_mut() {
                *value = Error::QueueOverflow;
            }
        }
    }

    fn pop_error(&mut self) -> Option<Error> {
        self.0.pop_front()
    }

    fn error_count(&self) -> usize {
        self.0.len()
    }
}
