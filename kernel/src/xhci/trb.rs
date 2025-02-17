use core::{mem::transmute, pin::Pin};

use super::{rings::TrbRing, volatile::Volatile};

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
    const CTRL_BIT_INTERRUPT_ON_SHORT_PACKET: u32 = 1 << 2;
    const CTRL_BIT_INTERRUPT_ON_COMPLETION: u32 = 1 << 5;
    const CTRL_BIT_IMMEDIATE_DATA: u32 = 1 << 6;

    const CTRL_BIT_DATA_DIR_OUT: u32 = 0 << 16;
    const CTRL_BIT_DATA_DIR_IN: u32 = 1 << 16;

    pub fn data(&self) -> u64 {
        self.buffer.read()
    }

    pub fn cycle_bit_state(&self) -> bool {
        self.control.read_bits(0, 1) != 0
    }

    pub fn set_cycle_bit_state(&mut self, cycle: bool) {
        self.control.write_bits(0, 1, cycle.into());
    }

    pub fn set_toggle_cycle(&mut self, value: bool) {
        self.control.write_bits(1, 1, value.into());
    }

    pub fn trb_type(&self) -> u32 {
        self.control.read_bits(10, 6)
    }

    pub fn set_trb_type(&mut self, trb_type: TrbType) {
        self.control.write_bits(10, 6, trb_type as u32);
    }

    pub fn slot_id(&self) -> u8 {
        self.control.read_bits(24, 8) as u8
    }

    pub fn trb_link(ring: &TrbRing) -> Self {
        let mut trb = TrbBase::default();
        trb.set_trb_type(TrbType::Link);
        trb.buffer.write(ring.phys_addr());
        trb.set_toggle_cycle(true);
        trb
    }
}

impl From<NormalTrb> for TrbBase {
    fn from(trb: NormalTrb) -> Self {
        unsafe { transmute(trb) }
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

/// A Setup Stage TRB is created by system software to initiate a USB setup packet
/// on a control endpoint.
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct SetupStageTrb {
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    transfer_info: u32,
    control: u32,
}

impl SetupStageTrb {
    // bmRequest bit[7]: Data Transfer Direction
    //      0: Host to Device
    //      1: Device to Host
    pub const REQ_TYPE_DIR_DEVICE_TO_HOST: u8 = 1 << 7;
    pub const REQ_TYPE_DIR_HOST_TO_DEVICE: u8 = 0 << 7;
    // bmRequest bit[5..=6]: Request Type
    //      0: Standard
    //      1: Class
    //      2: Vendor
    //      _: Reserved
    //pub const REQ_TYPE_TYPE_STANDARD: u8 = 0 << 5;
    pub const REQ_TYPE_TYPE_CLASS: u8 = 1 << 5;
    pub const REQ_TYPE_TYPE_VENDOR: u8 = 2 << 5;
    // bmRequest bit[0..=4]: Recipient
    //      0: Device
    //      1: Interface
    //      2: Endpoint
    //      3: Other
    //      _: Reserved
    pub const REQ_TYPE_TO_DEVICE: u8 = 0;
    pub const REQ_TYPE_TO_INTERFACE: u8 = 1;
    //pub const REQ_TYPE_TO_ENDPOINT: u8 = 2;
    //pub const REQ_TYPE_TO_OTHER: u8 = 3;

    pub const REQ_GET_REPORT: u8 = 1;
    pub const REQ_GET_DESCRIPTOR: u8 = 6;
    pub const REQ_SET_CONFIGURATION: u8 = 9;
    pub const REQ_SET_INTERFACE: u8 = 11;
    pub const REQ_SET_PROTOCOL: u8 = 0x0b;

    pub fn new(
        bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        w_index: u16,
        w_length: u16,
    ) -> Self {
        const TRT_NO_DATA_STAGE: u32 = 0;
        const TRT_OUT_DATA_STAGE: u32 = 2;
        const TRT_IN_DATA_STAGE: u32 = 3;
        let transfer_type = if w_length == 0 {
            TRT_NO_DATA_STAGE
        } else if b_request & Self::REQ_TYPE_DIR_DEVICE_TO_HOST != 0 {
            TRT_IN_DATA_STAGE
        } else {
            TRT_OUT_DATA_STAGE
        };

        Self {
            bm_request_type,
            b_request,
            w_value,
            w_index,
            w_length,
            transfer_info: 8,
            control: transfer_type << 16
                | (TrbType::SetupStage as u32) << 10
                | TrbBase::CTRL_BIT_IMMEDIATE_DATA,
        }
    }
}

/// A Data Stage TRB is used generate the Data stage transaction of a USB Control transfer
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct DataStageTrb {
    buffer: u64,
    transfer_info: u32,
    control: u32,
}

impl DataStageTrb {
    pub fn new_in<T: Sized>(buffer: Pin<&mut [T]>) -> Self {
        Self {
            buffer: buffer.as_ptr() as u64,
            transfer_info: (buffer.len() * size_of::<T>()) as u32,
            control: (TrbType::DataStage as u32) << 10
                | TrbBase::CTRL_BIT_DATA_DIR_IN
                | TrbBase::CTRL_BIT_INTERRUPT_ON_COMPLETION
                | TrbBase::CTRL_BIT_INTERRUPT_ON_SHORT_PACKET,
        }
    }
    pub fn new_out<T: Sized>(buffer: Pin<&mut [T]>) -> Self {
        Self {
            buffer: buffer.as_ptr() as u64,
            transfer_info: (buffer.len() * size_of::<T>()) as u32,
            control: (TrbType::DataStage as u32) << 10
                | TrbBase::CTRL_BIT_DATA_DIR_OUT
                | TrbBase::CTRL_BIT_INTERRUPT_ON_COMPLETION
                | TrbBase::CTRL_BIT_INTERRUPT_ON_SHORT_PACKET,
        }
    }
}

/// A Status Stage TRB is used to generate
/// the transaction of a USB Control transfer.
#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct StatusStageTrb {
    reserved: u64,
    transfer_info: u32,
    control: u32,
}

impl StatusStageTrb {
    pub fn new_out() -> Self {
        Self {
            reserved: 0,
            transfer_info: 0,
            control: (TrbType::StatusStage as u32) << 10,
        }
    }
    pub fn new_in() -> Self {
        Self {
            reserved: 0,
            transfer_info: 0,
            control: (TrbType::StatusStage as u32) << 10
                | TrbBase::CTRL_BIT_DATA_DIR_IN
                | TrbBase::CTRL_BIT_INTERRUPT_ON_COMPLETION
                | TrbBase::CTRL_BIT_INTERRUPT_ON_SHORT_PACKET,
        }
    }
}
