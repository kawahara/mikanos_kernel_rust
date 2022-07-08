use crate::{Graphics, PixelColor, Vector2D};
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Window {
    width: usize,
    height: usize,
    data: Vec<Vec<PixelColor>>,
}

impl Window {
    pub fn new(width: usize, height: usize) -> Self {
        let mut data = Vec::<Vec<PixelColor>>::with_capacity(height);
        for y in 0..height {
            let width_data = Vec::<PixelColor>::with_capacity(width);
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
                if let Some(color) = self.at(x, y) {
                    let px = position.x + x as isize;
                    let py = position.y + y as isize;
                    if px >= 0 && py >= 0 {
                        graphics.write_pixel(px as usize, py as usize, color);
                    }
                }
            }
        }
    }

    fn at(&self, x: usize, y: usize) -> Option<&PixelColor> {
        self.data[y].get(x)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
