pub const ROWS: usize = 30;
pub const COLUMNS: usize = 80;
pub const ARR_FORM_BUFFER: usize = COLUMNS * 10;

use crate::graphics::{Graphics, PixelColor};
use arrform::ArrForm;
use core::fmt;
use core::option::Option::{None, Some};
use spin::mutex::SpinMutex;

pub struct Console {
    graphics: Graphics,
    buffer: [[char; COLUMNS + 1]; ROWS],
    fg_color: PixelColor,
    bg_color: PixelColor,
    cursor_row: usize,
    cursor_column: usize,
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ($crate::console::_printk(format_args!($($arg)*)));
}

static mut CONSOLE: Option<SpinMutex<Console>> = None;

pub fn initialize_console(graphics: &Graphics, fg_color: &PixelColor, bg_color: &PixelColor) {
    unsafe {
        CONSOLE = Some(SpinMutex::new(Console::new(graphics, fg_color, bg_color)));
    }
}

#[doc(hidden)]
pub fn _printk(args: fmt::Arguments) {
    unsafe {
        if let Some(mut console) = CONSOLE.as_mut().unwrap().try_lock() {
            console.print(args);
        }
    }
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
        let mut af = ArrForm::<ARR_FORM_BUFFER>::new();
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
