use bootloader_api::info::FrameBuffer;

use crate::graphics::{Color, Display};

const ROWS: usize = 125;
const COLUMNS: usize = 80;

pub struct Console {
    display: Display,
    fg_color: Color,
    bg_color: Color,
    cursor_row: usize,
    cursor_column: usize,
    buffer: [[char; COLUMNS + 1]; ROWS],
}

impl Console {
    pub fn new(frame_buffer: &'static mut FrameBuffer, fg_color: Color, bg_color: Color) -> Self {
        Self {
            display: Display::new(frame_buffer),
            fg_color,
            bg_color,
            cursor_row: 0,
            cursor_column: 0,
            buffer: [['\0'; COLUMNS + 1]; ROWS],
        }
    }

    pub fn put_string(&mut self, str: &str) {
        for char in str.chars() {
            if char == '\n' || self.cursor_column >= COLUMNS - 1 {
                Console::new_line();
                if char == '\n' {
                    continue;
                }
            }

            self.display.draw_ascii(
                8 * self.cursor_column as i32,
                16 * self.cursor_row as i32,
                char,
                self.fg_color,
            );

            self.buffer[self.cursor_row][self.cursor_column] = char;
            self.cursor_column += 1;
        }
    }

    fn new_line() {
        todo!()
    }
}
