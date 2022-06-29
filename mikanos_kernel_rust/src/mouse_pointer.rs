use crate::graphics::{Graphics, PixelColor, Vector2D};

const POINTER: [&str; 24] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@.......@@@@@@@",
    "@.......@      ",
    "@....@@.@      ",
    "@...@  @.@     ",
    "@..@   @.@     ",
    "@.@     @.@    ",
    "@@      @.@    ",
    "@        @.@   ",
    "          @.@  ",
    "          @@@  ",
];

pub struct MousePointer {
    graphics: Graphics,
}

impl MousePointer {
    pub fn new(graphics: &Graphics) -> Self {
        MousePointer {
            graphics: *graphics,
        }
    }

    pub fn write(&mut self, pos: &Vector2D<usize>) {
        for dy in 0..24 {
            for dx in 0..15 {
                let c = POINTER[dy].chars().nth(dx).unwrap();
                if c == '@' {
                    self.graphics
                        .write_pixel(pos.x + dx, pos.y + dy, &PixelColor(0, 0, 0));
                } else if c == '.' {
                    self.graphics
                        .write_pixel(pos.x + dx, pos.y + dy, &PixelColor(255, 255, 255));
                }
            }
        }
    }
}
