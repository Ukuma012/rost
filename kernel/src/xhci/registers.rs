use core::ptr::write_volatile;

use spin::mutex::SpinMutex;

use crate::utils::extract_bits;

use super::volatile::Volatile;

/// These Registers specify the limits and capabilities of
/// the host controller implementation.
#[repr(C)]
pub struct CapabilityRegisters {
    caplength: Volatile<u8>,
    rsvd: Volatile<u8>,
    hciversion: Volatile<u16>,
    hcsparams1: Volatile<u32>,
    hcsparams2: Volatile<u32>,
    hcsparams3: Volatile<u32>,
    hccparams1: Volatile<u32>,
    dboff: Volatile<u32>,
    rtsoff: Volatile<u32>,
    hccparams2: Volatile<u32>,
}
impl CapabilityRegisters {
    pub fn length(&self) -> usize {
        self.caplength.read() as usize
    }

    pub fn rtsoff(&self) -> usize {
        self.rtsoff.read() as usize
    }

    pub fn dboff(&self) -> usize {
        self.dboff.read() as usize
    }

    pub fn num_of_device_slots(&self) -> usize {
        extract_bits(self.hcsparams1.read(), 0, 8) as usize
    }

    pub fn num_of_interrupters(&self) -> usize {
        extract_bits(self.hcsparams1.read(), 8, 11) as usize
    }

    pub fn num_of_ports(&self) -> usize {
        extract_bits(self.hcsparams1.read(), 24, 8) as usize
    }

    pub fn num_of_scratch_pad_buffers(&self) -> usize {
        (extract_bits(self.hcsparams2.read(), 21, 5) << 5
            | extract_bits(self.hcsparams2.read(), 27, 5)) as usize
    }
}

// Transfer RingやCommand RingにTRBが追加されたことをxHCに通知するための仕組み
// 32bit幅で、最大256個
// ドアベルレジスタ0:
// ホストコントローラに紐づく。０を書き込むことでCommand RingにTRBを追加したことを通知する
pub struct DoorbellRegisters {
    ptr: SpinMutex<*mut u32>,
}

impl DoorbellRegisters {
    pub fn new(ptr: *mut u32) -> Self {
        Self {
            ptr: SpinMutex::new(ptr),
        }
    }

    // bit 7:0 Doorbell Target.
    // Doorbell Register 0 is dedicated to Command Ring
    // bit 15:8 RsvdZ
    // bit 31:16 Doorbell Stream ID.
    // If the endpoint of a Device Context Doorbell defines Streams, then this field shall be used to identify which Stream of the endpoint the doorbell reference is targeting.
    // Command Ringの場合(doorbell.notify(0, 0))
    pub fn notify(&self, target: u8, stream_id: u16) {
        let value = (target as u32) | (stream_id as u32) << 16;

        unsafe {
            write_volatile(*self.ptr.lock(), value);
        }
    }
}
