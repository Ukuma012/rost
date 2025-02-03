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
    const CTRL_BIT_INTERRUPT_ON_SHORT_PACKET: u32 = 1 << 2;
    const CTRL_BIT_INTERRUPT_ON_COMPLETION: u32 = 1 << 5;
    const CTRL_BIT_IMMEDIATE_DATA: u32 = 1 << 6;

    const CTRL_BIT_DATA_DIR_OUT: u32 = 0 << 16;
    const CTRL_BIT_DATA_DIR_IN: u32 = 1 << 16;

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

/// USBデバイスの初期化時に、USBデバイスからデバイス情報を得たり設定を
/// 書き込んだりするのに用いる
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
