use super::volatile::Volatile;

/// The Transfer Request Block is the basic building block upon which all xHC USB transfers are constructed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum TrbType {
    Normal = 1,
    SetupStage = 2,
    DataStage = 3,
    StatusStage = 4,
    Link = 6,
    EnableSlotCommand = 9,
    AddressDeviceCommand = 11,
    ConfigureEndpointCommand = 12,
    EvaluateContextCommand = 13,
    NoOpCommand = 23,
    TransferEvent = 32,
    CommandCompletionEvent = 33,
    PortStatusChangeEvent = 34,
    HostControllerEvent = 37,
}

#[derive(Default, Clone)]
#[repr(C, align(16))]
pub struct TrbBase {
    buffer: Volatile<u64>,
    transfer_info: Volatile<u32>,
    control: Volatile<u32>,
}

impl TrbBase {
    pub fn cycle_bit_state(&self) -> bool {
        self.control.read_bits(0, 1) != 0
    }

    pub fn set_cycle_bit_state(&mut self, cycle: bool) {
        self.control.write_bits(0, 1, cycle.into());
    }
}

#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct NormalTrb {
    buffer: u64,
    transfer_info: u32,
    control: u32,
}

impl NormalTrb {
    const CONTROL_INTERRUPT_ON_COMPLETION: u32 = 1 << 5;
    const CONTROL_INTERRUPT_ON_SHORT_PACKET: u32 = 1 << 2;
    pub fn new(buffer: *mut u8, transfer_info: u16) -> Self {
        Self {
            buffer: buffer as u64,
            transfer_info: transfer_info as u32,
            control: (TrbType::Normal as u32) << 10
                | Self::CONTROL_INTERRUPT_ON_COMPLETION
                | Self::CONTROL_INTERRUPT_ON_SHORT_PACKET,
        }
    }
}
