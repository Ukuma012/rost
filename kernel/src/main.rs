#![no_std]
#![no_main]
use common::boot_info::BootInfo;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
