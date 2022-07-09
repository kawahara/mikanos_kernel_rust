use crate::fonts::FONTS;
use crate::{Graphics, PixelColor, Vector2D};
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Window {
    width: usize,
    height: usize,
    data: Vec<Vec<Option<PixelColor>>>,
}

impl Window {
    pub fn new(width: usize, height: usize) -> Self {
        let mut data = Vec::<Vec<Option<PixelColor>>>::with_capacity(height);
        for y in 0..height {
            let width_data = Vec::<Option<PixelColor>>::with_capacity(width);
            data[y] = width_data
        }

        Window {
            width,
            height,
            data,
        }
    }

    pub fn draw_to(&self, graphics: &mut Graphics, position: &Vector2D<isize>) {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(color) = self.get(x, y) {
                    let px = position.x + x as isize;
                    let py = position.y + y as isize;
                    if px >= 0 && py >= 0 {
                        graphics.write_pixel(px as usize, py as usize, &color);
                    }
                }
            }
        }
    }

    pub fn write(&mut self, x: usize, y: usize, c: Option<PixelColor>) {
        if self.data.len() > y && self.data[y].len() > x {
            self.data[y][x] = c;
        }
    }

    pub fn write_ascii(&mut self, x: usize, y: usize, c: char, color: PixelColor) {
        if c as u32 > 0x7f {
            return;
        }

        let font: [u8; 16] = FONTS[c as usize];

        for (dy, line) in font.iter().enumerate() {
            for dx in 0..8 {
                if (line << dx) & 0x80 != 0 {
                    self.write(x + dx, y + dy, Some(color))
                }
            }
        }
    }

    pub fn write_string(&mut self, x: usize, y: usize, str: &str, color: PixelColor) {
        for i in 0..str.len() {
            self.write_ascii(x + 8 * i, y, str.chars().nth(i).unwrap(), color);
        }
    }

    pub fn fill_rectangle(
        &mut self,
        pos: Vector2D<usize>,
        size: Vector2D<usize>,
        color: Option<PixelColor>,
    ) {
        for dy in 0..size.y {
            for dx in 0..size.x {
                self.write(pos.x + dx, pos.y + dy, color);
            }
        }
    }

    pub fn draw_rectangle(
        &mut self,
        pos: Vector2D<usize>,
        size: Vector2D<usize>,
        color: PixelColor,
    ) {
        for dx in 0..size.x {
            self.write(pos.x + dx, pos.y, Some(color));
            self.write(pos.x + dx, pos.y + size.y, Some(color));
        }
        for dy in 0..size.y {
            self.write(pos.x, pos.y + dy, Some(color));
            self.write(pos.x + size.x, pos.y + dy, Some(color));
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<PixelColor> {
        *self.data.get(y)?.get(x)?
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
