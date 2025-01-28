use core::panic;

use bootloader_api::info::{FrameBuffer, PixelFormat};

pub fn set_pixel_in(framebuffer: &mut FrameBuffer, position: Position, color: Color) {
    let info = framebuffer.info();

    let byte_offset = {
        // 縦
        let line_offset = position.y * info.stride;
        // 横
        let pixel_offset = line_offset + position.x;
        // 何byteか
        pixel_offset * info.bytes_per_pixel
    };

    let pixel_buffer = &mut framebuffer.buffer_mut()[byte_offset..];
    match info.pixel_format {
        PixelFormat::Rgb => {
            pixel_buffer[0] = color.red;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.blue;
        }
        PixelFormat::Bgr => {
            pixel_buffer[0] = color.blue;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.red;
        }
        PixelFormat::U8 => {
            let gray = color.red / 3 + color.green / 3 + color.blue / 3;
            pixel_buffer[0] = gray;
        }
        other => panic!("unknown pixel format {other:?}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
