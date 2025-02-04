extern crate alloc;

use alloc::boxed::Box;
use core::{marker::PhantomPinned, mem::MaybeUninit, pin::Pin};

#[repr(C, align(64))]
pub struct RawDeviceContextBaseAddressArray {
    context: [u64; 256],
    // メモリ位置を固定
    _pinned: PhantomPinned,
}
impl RawDeviceContextBaseAddressArray {
    fn new() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

pub struct DeviceContextBaseAddressArray {
    inner: Pin<Box<RawDeviceContextBaseAddressArray>>,
}

impl DeviceContextBaseAddressArray {
    pub unsafe fn inner_mut_ptr(&mut self) -> *mut RawDeviceContextBaseAddressArray {
        self.inner.as_mut().get_unchecked_mut() as *mut RawDeviceContextBaseAddressArray
    }
}
