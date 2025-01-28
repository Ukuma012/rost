use core::panic;

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{self, Size},
    pixelcolor::{Rgb888, RgbColor},
    prelude::Point,
    Pixel,
};

pub fn set_pixel_in(
    framebuffer: &mut [u8],
    info: FrameBufferInfo,
    position: Position,
    color: Color,
) {
    let byte_offset = {
        // 縦
        let line_offset = position.y * info.stride;
        // 横
        let pixel_offset = line_offset + position.x;
        // 何byteか
        pixel_offset * info.bytes_per_pixel
    };

    let pixel_buffer = &mut framebuffer[byte_offset..];
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

pub struct Display {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl Display {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Display {
        Self {
            info: framebuffer.info(),
            framebuffer: framebuffer.buffer_mut(),
        }
    }

    fn draw_pixel(&mut self, coordinates: Point, color: Rgb888) {
        let position = match (coordinates.x.try_into(), coordinates.y.try_into()) {
            (Ok(x), Ok(y)) if x < self.info.width && y < self.info.height => Position { x, y },
            _ => return,
        };
        let color = Color {
            red: color.r(),
            green: color.g(),
            blue: color.b(),
        };
        set_pixel_in(self.framebuffer, self.info, position, color);
    }

    pub fn split_at_line(self, line_index: usize) -> (Self, Self) {
        assert!(line_index < self.info.height);

        let byte_offset = line_index * self.info.stride * self.info.bytes_per_pixel;
        let (first_buffer, second_buffer) = self.framebuffer.split_at_mut(byte_offset);

        let first = Self {
            framebuffer: first_buffer,
            info: FrameBufferInfo {
                byte_len: byte_offset,
                height: line_index,
                ..self.info
            },
        };

        let second = Self {
            framebuffer: second_buffer,
            info: FrameBufferInfo {
                byte_len: self.info.byte_len - byte_offset,
                height: self.info.height - line_index,
                ..self.info
            },
        };

        (first, second)
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coordinates, color) in pixels.into_iter() {
            self.draw_pixel(coordinates, color);
        }
        Ok(())
    }
}

impl geometry::OriginDimensions for Display {
    fn size(&self) -> Size {
        geometry::Size::new(
            self.info.width.try_into().unwrap(),
            self.info.height.try_into().unwrap(),
        )
    }
}
