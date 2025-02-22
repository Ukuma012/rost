#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(try_blocks)]
use anyhow;
use bootloader_api::config::Mapping;
use bootloader_api::BootloaderConfig;
use console::{Console, CONSOLE};
use core::arch::asm;
use core::panic::PanicInfo;
use memory::BootInfoFrameAllocator;
use task::executor::{Executor, Spawner};
use task::keyboard;
use x86_64::VirtAddr;

mod allocator;
mod console;
mod gdt;
mod interrupts;
mod memory;
mod task;
mod usb;
mod utils;
mod xhci;

extern crate alloc;

static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

const OWL: &str = "
             _______________________
    ___     |                       | 
   (o,o)   <  Hello! I'm ROST       |
   {`\"'}    |_______________________|  
   -\"-\"-  
";

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

    println!("{}", OWL);
    println!("Welcome to my hobby OS!");

    let _result: anyhow::Result<()> = try {
        let spawner = Spawner::new(100);
        let mut executor = Executor::new(spawner.clone());
        spawner.add(keyboard::print_keypresses());
        executor.run();
    };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        unsafe { asm!("hlt") }
    }
}
