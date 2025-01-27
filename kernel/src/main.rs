#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x00;
        }
    }
    loop {
        unsafe {asm!("hlt")}
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
