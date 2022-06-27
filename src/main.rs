#![no_std]
#![no_main]

pub mod graphics;
pub mod fonts;

use core::panic::PanicInfo;
use graphics::{FrameBuffer, Graphics, PixelColor};

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer) {
    let fb_a = unsafe { *fb };
    let mut graphics = Graphics::new(fb_a);

    for x in 0..fb_a.horizontal_resolution as usize {
        for y in 0..fb_a.vertical_resolution as usize {
            graphics.write_pixel(x, y, &PixelColor(255, 255, 255));
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            graphics.write_pixel(x, y, &PixelColor(0, 255, 0))
        }
    }

    graphics.write_string(0, 0, "hello!", &PixelColor(0, 0, 0));

    hlt_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
