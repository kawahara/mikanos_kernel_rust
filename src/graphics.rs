use crate::fonts::FONTS;

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct FrameBuffer {
    pub frame_buffer: *mut u8,
    pub pixels_per_scan_line: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub format: PixelFormat,
}

impl FrameBuffer {
    pub unsafe fn write_byte(&mut self, index: usize, val: u8) {
        self.frame_buffer.add(index).write_volatile(val)
    }

    pub unsafe fn write_value(&mut self, index: usize, value: [u8; 3]) {
        (self.frame_buffer.add(index) as *mut [u8; 3]).write_volatile(value)
    }
}

pub struct PixelColor(pub u8, pub u8, pub u8);

pub struct Graphics {
    fb: FrameBuffer,
    pixel_writer: unsafe fn(&mut FrameBuffer, usize, &PixelColor),
}

impl Graphics {
    pub fn new(fb: FrameBuffer) -> Self {
        unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, index: usize, rgb: &PixelColor) {
            fb.write_value(index, [rgb.0, rgb.1, rgb.2]);
        }
        unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, index: usize, rgb: &PixelColor) {
            fb.write_value(index, [rgb.2, rgb.1, rgb.0]);
        }
        let pixel_writer = match fb.format {
            PixelFormat::Rgb => write_pixel_rgb,
            PixelFormat::Bgr => write_pixel_bgr,
        };

        Graphics { fb, pixel_writer }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: &PixelColor) {
        if x > self.fb.horizontal_resolution as usize {
            panic!("bad x coord");
        }
        if y > self.fb.vertical_resolution as usize {
            panic!("bad y coord");
        }
        let pixel_index = y * (self.fb.pixels_per_scan_line as usize) + x;
        let base = 4 * pixel_index;
        unsafe {
            (self.pixel_writer)(&mut self.fb, base, &color);
        }
    }

    pub fn write_ascii(&mut self, x: usize, y: usize, c: char, color: &PixelColor) {
        if c as u32 > 0x7f {
            return;
        }

        let font: [u8; 16] = FONTS[c as usize];

        for (dy, line) in font.iter().enumerate() {
            for dx in 0..8 {
                if (line << dx) & 0x80 != 0 {
                    self.write_pixel(x + dx, y + dy, &color)
                }
            }
        }
    }

    pub fn write_string(&mut self, x: usize, y: usize, str: &str, color: &PixelColor) {
        for i in 0..str.len() {
            self.write_ascii(x + 8 * i, y, str.chars().nth(i).unwrap(), color);
        }
    }
}
