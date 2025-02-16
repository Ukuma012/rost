use core::{
    arch::asm,
    ptr::{read_volatile, write_volatile},
};

use spin::mutex::SpinMutex;

use crate::utils::extract_bits;

use super::{
    contexts::{DeviceContextBaseAddressArray, RawDeviceContextBaseAddressArray},
    rings::CommandRing,
    volatile::Volatile,
};

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

/// xHCIホストコントローラの実際の動作を制御するためのレジスタ
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OperationalRegisters {
    command: u32,
    status: u32,
    page_size: u32,
    rsvdz1: [u32; 2],
    notification_ctr1: u32,
    cmd_ring_ctrl: u64,
    rsvdz2: [u64; 2],
    // Device Context Base Address Array Pointer
    // デバイスコンテキストのためにメモリ領域を確保するのは
    // ソフトウェアの役目
    // デバイスコンテキストの先頭アドレスを並べた配列DCBAAを用意し、
    // その配列の先頭アドレスをDCBAAPレジスタに設定する
    dcbaap: *mut RawDeviceContextBaseAddressArray,
    config: u64,
}

impl OperationalRegisters {
    const CMD_RUN_STOP: u32 = 0b0001;
    const CMD_HC_RESET: u32 = 0b0010;
    const STATUS_HC_HALTED: u32 = 0b0001;
    fn command(&mut self) -> u32 {
        unsafe { read_volatile(&self.command) }
    }

    fn clear_command_bits(&mut self, bits: u32) {
        unsafe {
            write_volatile(&mut self.command, self.command() & !bits);
        }
    }

    fn set_command_bits(&mut self, bits: u32) {
        unsafe {
            write_volatile(&mut self.command, self.command() | bits);
        }
    }

    fn status(&mut self) -> u32 {
        unsafe { read_volatile(&self.status) }
    }

    pub fn page_size(&self) -> usize {
        let page_size_bits = unsafe { read_volatile(&self.page_size) } & 0xFFFF;
        if page_size_bits.count_ones() != 1 {
            panic!("PAGE_SIZE has multiple bits set");
        }
        let page_size_shift = page_size_bits.trailing_zeros();
        1 << (page_size_shift + 12)
    }

    pub fn set_num_device_slots(&mut self, num: usize) {
        unsafe {
            let c = read_volatile(&self.config);
            // 下位8bitをクリア
            let c = c & !0xFF;
            let c = c | u64::try_from(num).unwrap();
            write_volatile(&mut self.config, c);
        }
    }

    pub fn set_dcbaa_ptr(&mut self, dcbaa: &mut DeviceContextBaseAddressArray) {
        unsafe {
            write_volatile(&mut self.dcbaap, dcbaa.inner_mut_ptr());
        }
    }

    pub fn set_cmd_ring_ctrl(&mut self, ring: &CommandRing) {
        self.cmd_ring_ctrl = ring.ring_phys_addr() | 1 /* Consumer Ring Cycle State */
    }

    pub fn reset_xhc(&mut self) {
        self.clear_command_bits(Self::CMD_RUN_STOP);
        while self.status() & Self::STATUS_HC_HALTED == 0 {
            unsafe { asm!("pause") }
        }
        self.set_command_bits(Self::CMD_HC_RESET);
        while self.command() & Self::CMD_HC_RESET != 0 {
            unsafe { asm!("pause") }
        }
    }
    pub fn start_xhc(&mut self) {
        self.set_command_bits(Self::CMD_RUN_STOP);
        while self.status() & Self::STATUS_HC_HALTED != 0 {
            unsafe { asm!("pause") }
        }
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
