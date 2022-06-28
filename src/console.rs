pub const ROWS: usize = 40;
pub const COLUMNS: usize = 80;

use crate::graphics::{Graphics, PixelColor};
use arrform::ArrForm;
use core::fmt;

pub struct Console {
    graphics: Graphics,
    buffer: [[char; COLUMNS + 1]; ROWS],
    fg_color: PixelColor,
    bg_color: PixelColor,
    cursor_row: usize,
    cursor_column: usize,
}

impl Console {
    pub fn new(graphics: &Graphics, fg_color: &PixelColor, bg_color: &PixelColor) -> Self {
        Console {
            graphics: *graphics,
            buffer: [[0.into(); COLUMNS + 1]; ROWS],
            fg_color: *fg_color,
            bg_color: *bg_color,
            cursor_row: 0,
            cursor_column: 0,
        }
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        let mut af = ArrForm::<200>::new();
        af.format(args).expect("Buffer overflow");
        self.put_string(af.as_str())
    }

    pub fn put_string(&mut self, str: &str) {
        for i in 0..str.len() {
            let c = str.chars().nth(i).unwrap();
            if c == '\n' {
                self.new_line();
            } else {
                if self.cursor_column >= COLUMNS {
                    self.new_line()
                }

                self.graphics.write_ascii(
                    8 * self.cursor_column,
                    16 * self.cursor_row,
                    c,
                    &self.fg_color,
                );
                self.buffer[self.cursor_row][self.cursor_column] = c;
                self.cursor_column += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            // clear line
            for y in 0..(16 * ROWS) {
                for x in 0..(8 * COLUMNS) {
                    self.graphics.write_pixel(x, y, &self.bg_color);
                }
            }
            for row in 0..(ROWS - 1) {
                self.buffer[row] = self.buffer[row + 1];
                for col in 0..COLUMNS {
                    self.graphics.write_ascii(
                        8 * col,
                        16 * row,
                        self.buffer[row][col],
                        &self.fg_color,
                    );
                }
            }
            self.buffer[ROWS - 1] = [0.into(); COLUMNS + 1];
        }
    }
}
