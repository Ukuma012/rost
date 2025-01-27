#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "sysv64" fn kernel_main() {
    loop {
        unsafe { asm!("hlt") }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
