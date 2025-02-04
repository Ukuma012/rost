#![no_std]
#![no_main]
use core::arch::asm;
use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use console::{Console, CONSOLE};

mod console;
mod usb;
mod xhci;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    init(boot_info);
    println!("Hello World!");
    loop {
        unsafe { asm!("hlt") }
    }
}

fn init(boot_info: &'static mut BootInfo) {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        CONSOLE.init_once(|| {
            let info = framebuffer.info();
            let buffer = framebuffer.buffer_mut();
            spinning_top::Spinlock::new(Console::new(buffer, info))
        });
    }

    println!("init: success!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        unsafe { asm!("hlt") }
    }
}
