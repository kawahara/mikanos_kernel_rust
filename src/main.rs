#![no_std]
#![no_main]

pub mod graphics;

use core::panic::PanicInfo;
use core::arch::asm;
use graphics::{Graphics, FrameBuffer, PixelColor};

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer) {
    let fb_a = unsafe {*fb};
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

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
