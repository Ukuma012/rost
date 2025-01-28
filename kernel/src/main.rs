#![no_std]
#![no_main]
use core::arch::asm;
use core::convert::Infallible;
use core::panic::PanicInfo;
use embedded_graphics::Drawable;

use embedded_graphics::mono_font::iso_8859_13::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{DrawTarget, Point, RgbColor};
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Text;

mod framebuffer;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        for byte in framebuffer.buffer_mut() {
            *byte = 0x00;
        }

        let mut display = framebuffer::Display::new(framebuffer);

        let character_style = MonoTextStyle::new(&FONT_10X20, Rgb888::GREEN);
        let text = Text::new("Hello, World!", Point::new(0, 20), character_style);
        text.draw(&mut display).unwrap_or_else(infallible);
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
