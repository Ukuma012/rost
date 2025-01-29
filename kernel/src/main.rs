#![no_std]
#![no_main]
use core::arch::asm;
use core::convert::Infallible;
use core::panic::PanicInfo;

use console::Console;
use display::Color;

mod console;
mod display;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut console = Console::new(
            framebuffer,
            Color {
                red: 0,
                green: 255,
                blue: 255,
            },
            Color {
                red: 0,
                green: 0,
                blue: 0,
            },
        );

        console.clear();
        for _ in 1..18 {
            console.put_string("Hello\n");
        }
        console.put_string("This is Rost");
    }

    loop {
        unsafe { asm!("hlt") }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn infallible<T>(v: Infallible) -> T {
    match v {}
}
