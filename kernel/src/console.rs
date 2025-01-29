use bootloader_api::info::FrameBuffer;

use crate::display::{Color, Display};

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

    pub fn clear(&mut self) {
        self.display.clear(self.bg_color);
    }

    pub fn put_string(&mut self, str: &str) {
        for char in str.chars() {
            if char == '\n' || self.cursor_column >= COLUMNS - 1 {
                self.new_line();
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

    fn new_line(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
            return;
        } else {
            for row in 1..ROWS {
                self.buffer[row - 1] = self.buffer[row];
                for col in 0..COLUMNS {
                    let char = self.buffer[row - 1][col];
                    self.display.draw_ascii(
                        8 * col as i32,
                        16 * (row - 1) as i32,
                        char,
                        self.fg_color,
                    );
                }
            }
        }
        self.buffer[ROWS - 1] = [char::from(0); COLUMNS + 1];

        for col in 0..COLUMNS {
            self.display
                .draw_ascii(8 * col as i32, 16 * (ROWS - 1) as i32, ' ', self.fg_color);
        }
    }
}
