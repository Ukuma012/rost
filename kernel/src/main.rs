#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
use alloc::boxed::Box;
use alloc::vec::Vec;
use allocator::MemoryAllocator;
use bootloader_api::config::Mapping;
use bootloader_api::BootloaderConfig;
use console::{Console, CONSOLE};
use core::arch::asm;
use core::panic::PanicInfo;
use memory::BootInfoFrameAllocator;
use x86_64::VirtAddr;

mod allocator;
mod console;
mod gdt;
mod interrupts;
mod memory;
mod usb;
mod utils;
mod xhci;

extern crate alloc;

static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        CONSOLE.init_once(|| {
            let info = framebuffer.info();
            let buffer = framebuffer.buffer_mut();
            spinning_top::Spinlock::new(Console::new(buffer, info))
        });
    }

    gdt::init();
    interrupts::init_idt();

    let phys_mem_offset = VirtAddr::new(*boot_info.physical_memory_offset.as_ref().unwrap());
    let mut mapper: x86_64::structures::paging::OffsetPageTable<'_> =
        unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let x = Box::new(41);

    println!("Hello World: {}", x);

    loop {
        unsafe { asm!("hlt") }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        unsafe { asm!("hlt") }
    }
}
