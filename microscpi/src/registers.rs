bitflags::bitflags! {
    pub struct EventStatus: u8 {
        const POWER_ON = (1 << 7);
        const USER_REQUEST = (1 << 6);
        const COMMAND_ERROR = (1 << 5);
        const EXECUTION_ERROR = (1 << 4);
        const DEVICE_DEPENDENT_ERROR = (1 << 3);
        const QUERY_ERROR = (1 << 2);
        const REQUEST_CONTROL = (1 << 1);
        const OPERATION_COMPLETE = (1 << 0);
    }

    pub struct StatusByte: u8 {
        const OPERATION_STATUS = (1 << 7);
        const REQUEST_SERVICE = (1 << 6);
        const STANDARD_EVENT = (1 << 5);
        const MESSAGE_AVAILABLE = (1 << 4);
        const DATA_QUESTIONABLE = (1 << 3);
        const ERROR_EVENT_QUEUE = (1 << 2);
    }
}

pub struct StatusRegisters {
    pub event_status: EventStatus,
    /// The enable register contains a mask which bits of the event status
    /// will be considered.
    pub event_status_enable: EventStatus,
}

impl Default for StatusRegisters {
    fn default() -> Self {
        Self {
            event_status: EventStatus::POWER_ON,
            event_status_enable: EventStatus::all(),
        }
    }
}
