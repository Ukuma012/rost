use core::ptr::write_volatile;

use spin::mutex::SpinMutex;

// Transfer RingやCommand RingにTRBが追加されたことをxHCに通知するための仕組み
// 32bit幅で、最大256個
// ドアベルレジスタ0:
// ホストコントローラに紐づく。０を書き込むことでCommand RingにTRBを追加したことを通知する
pub struct Doorbell {
    ptr: SpinMutex<*mut u32>,
}

impl Doorbell {
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
