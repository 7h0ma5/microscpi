bitflags::bitflags! {
    #[derive(Clone, Copy)]
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

    #[derive(Clone, Copy)]
    pub struct StatusByte: u8 {
        const OPERATION_STATUS = (1 << 7);
        const REQUEST_SERVICE = (1 << 6);
        const STANDARD_EVENT = (1 << 5);
        const MESSAGE_AVAILABLE = (1 << 4);
        const DATA_QUESTIONABLE = (1 << 3);
        const ERROR_EVENT_QUEUE = (1 << 2);
        const IMPLEMENTOR_DEFINED_1 = (1 << 1);
        const IMPLEMENTOR_DEFINED_0 = (1 << 0);
    }
}

pub struct StatusRegisters {
    pub event_status: EventStatus,
    /// The event enable register contains a mask which bits of the event status
    /// will be considered.
    pub event_status_enable: EventStatus,
    /// The status enable register contains a mask which bits of the status byte
    /// will be considered.
    pub status_byte_enable: StatusByte,
}

impl Default for StatusRegisters {
    fn default() -> Self {
        Self {
            event_status: EventStatus::POWER_ON,
            event_status_enable: EventStatus::all(),
            status_byte_enable: StatusByte::all() 
                & !StatusByte::IMPLEMENTOR_DEFINED_1 
                & !StatusByte::IMPLEMENTOR_DEFINED_0,
        }
    }
}
