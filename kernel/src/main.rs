#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
use allocator::MemoryAllocator;
use core::arch::asm;
use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use console::{Console, CONSOLE};

mod allocator;
mod console;
mod gdt;
mod interrupts;
mod paging;
mod usb;
mod utils;
mod xhci;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    init(boot_info);
    println!("Hello World!");

    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

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

    interrupts::init_idt();

    println!("init: success!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        unsafe { asm!("hlt") }
    }
}

#[global_allocator]
static ALLOCATOR: MemoryAllocator = MemoryAllocator;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test()
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
