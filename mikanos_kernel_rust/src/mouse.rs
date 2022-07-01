use crate::graphics::{Graphics, PixelColor, Vector2D};

const CURSOR_WIDTH: usize = 15;
const CURSOR_HEIGHT: usize = 24;

const POINTER: [&str; CURSOR_HEIGHT] = [
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

pub struct MouseCursor {
    graphics: Graphics,
    pos: Vector2D<usize>,
    erase_color: PixelColor,
}

impl MouseCursor {
    pub fn new(
        graphics: &Graphics,
        initial_pos: &Vector2D<usize>,
        erase_color: &PixelColor,
    ) -> Self {
        let mut cursor = MouseCursor {
            graphics: *graphics,
            pos: *initial_pos,
            erase_color: *erase_color,
        };

        cursor.draw_mouse_cursor();

        cursor
    }

    pub fn move_relative(&mut self, displacement: &Vector2D<isize>) {
        self.erase_mouse_cursor();
        let x = (self.pos.x as isize + displacement.x) as usize;
        let y = (self.pos.y as isize + displacement.y) as usize;
        self.pos = Vector2D::<usize> { x, y };
        self.draw_mouse_cursor();
    }

    fn draw_mouse_cursor(&mut self) {
        for dy in 0..CURSOR_HEIGHT {
            for dx in 0..CURSOR_WIDTH {
                let c = POINTER[dy].chars().nth(dx).unwrap();
                if c == '@' {
                    self.graphics.write_pixel(
                        self.pos.x + dx,
                        self.pos.y + dy,
                        &PixelColor(0, 0, 0),
                    );
                } else if c == '.' {
                    self.graphics.write_pixel(
                        self.pos.x + dx,
                        self.pos.y + dy,
                        &PixelColor(255, 255, 255),
                    );
                }
            }
        }
    }

    fn erase_mouse_cursor(&mut self) {
        for dy in 0..CURSOR_HEIGHT {
            for dx in 0..CURSOR_WIDTH {
                if POINTER[dy].chars().nth(dx).unwrap() != ' ' {
                    self.graphics
                        .write_pixel(self.pos.x + dx, self.pos.y + dy, &self.erase_color);
                }
            }
        }
    }
}
