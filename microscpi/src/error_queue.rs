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
    /// Clear the error queue.
    fn clear(&mut self);
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

    fn clear(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop_error() {
        let mut queue: StaticErrorQueue<3> = StaticErrorQueue::new();
        assert_eq!(queue.error_count(), 0);

        queue.push_error(Error::ExpressionError);
        assert_eq!(queue.error_count(), 1);

        let error = queue.pop_error();
        assert_eq!(error, Some(Error::ExpressionError));
        assert_eq!(queue.error_count(), 0);
    }

    #[test]
    fn test_queue_overflow() {
        let mut queue: StaticErrorQueue<2> = StaticErrorQueue::new();
        queue.push_error(Error::CalibrationFailed);
        queue.push_error(Error::HardwareError);
        assert_eq!(queue.error_count(), 2);

        // This push should cause an overflow
        queue.push_error(Error::DataTypeError);
        assert_eq!(queue.error_count(), 2);

        // The last error should be QueueOverflow
        let error = queue.pop_error();
        assert_eq!(error, Some(Error::CalibrationFailed));
        let error = queue.pop_error();
        assert_eq!(error, Some(Error::QueueOverflow));
    }

    #[test]
    fn test_pop_empty_queue() {
        let mut queue: StaticErrorQueue<2> = StaticErrorQueue::new();
        let error = queue.pop_error();
        assert_eq!(error, None);
    }

    #[test]
    fn test_error_count() {
        let mut queue: StaticErrorQueue<3> = StaticErrorQueue::new();
        assert_eq!(queue.error_count(), 0);

        queue.push_error(Error::BlockDataNotAllowed);
        assert_eq!(queue.error_count(), 1);

        queue.push_error(Error::OutOfMemory);
        assert_eq!(queue.error_count(), 2);

        queue.pop_error();
        assert_eq!(queue.error_count(), 1);

        queue.pop_error();
        assert_eq!(queue.error_count(), 0);
    }
}
