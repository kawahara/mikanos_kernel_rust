#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod console;
pub mod cxx_support;
pub mod fonts;
pub mod graphics;
pub mod interrupt;
pub mod logger;
pub mod mouse;
pub mod pci;
pub mod sync;
pub mod xhc;

use console::initialize_console;
use core::panic::PanicInfo;
use graphics::{FrameBuffer, Graphics, PixelColor, Vector2D};
use logger::Level as LogLevel;

fn hlt_loop() {
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer) {
    logger::set_level(logger::Level::Info);

    let fb_a = unsafe { *fb };
    let bg_color = PixelColor(45, 118, 237);
    let fg_color = PixelColor(255, 255, 255);
    let mut graphics = Graphics::new(fb_a);
    initialize_console(&graphics, &fg_color, &bg_color);

    graphics.fill_rectangle(
        &Vector2D::<usize> { x: 0, y: 0 },
        &Vector2D::<usize> {
            x: fb_a.horizontal_resolution as usize,
            y: (fb_a.vertical_resolution as usize) - 50,
        },
        &bg_color,
    );
    graphics.fill_rectangle(
        &Vector2D::<usize> {
            x: 0,
            y: (fb_a.vertical_resolution as usize) - 50,
        },
        &Vector2D::<usize> {
            x: fb_a.horizontal_resolution as usize,
            y: 50,
        },
        &PixelColor(1, 8, 17),
    );
    graphics.fill_rectangle(
        &Vector2D::<usize> {
            x: 0,
            y: (fb_a.vertical_resolution as usize) - 50,
        },
        &Vector2D::<usize> {
            x: (fb_a.horizontal_resolution as usize) / 5,
            y: 50,
        },
        &PixelColor(80, 80, 80),
    );
    graphics.draw_rectangle(
        &Vector2D::<usize> {
            x: 10,
            y: (fb_a.vertical_resolution as usize) - 40,
        },
        &Vector2D::<usize> { x: 30, y: 30 },
        &PixelColor(160, 160, 160),
    );
    mouse::init(&graphics, &&Vector2D::<usize> { x: 200, y: 100 }, &bg_color);

    printk!("Welcome to MikanOS Rust!!\n");

    interrupt::init();
    log!(LogLevel::Info, "Load PCI devices\n");
    let devices = pci::scan_all_bus().expect("Failed to scan PCI devices");
    xhc::init(&devices).expect("Failed to init xHC device");
    x86_64::instructions::interrupts::enable();

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printk!("Panic!! {}", info);

    loop {}
}
