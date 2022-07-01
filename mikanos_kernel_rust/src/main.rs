#![no_std]
#![no_main]

pub mod console;
pub mod fonts;
pub mod graphics;
pub mod mouse_pointer;
pub mod pci;

use console::initialize_console;
use core::panic::PanicInfo;
use graphics::{FrameBuffer, Graphics, PixelColor, Vector2D};
use mouse_pointer::MousePointer;
use mikanos_usb_driver as usb;

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer) {
    let fb_a = unsafe { *fb };
    let bg_color = PixelColor(45, 118, 237);
    let fg_color = PixelColor(255, 255, 255);
    let mut graphics = Graphics::new(fb_a);
    initialize_console(&graphics, &fg_color, &bg_color);
    let mut mouse: MousePointer = MousePointer::new(&graphics);

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
    mouse.write(&Vector2D::<usize> { x: 200, y: 100 });

    printk!("Welcome to MikanOS Rust!!\n");
    printk!("Load PCI devices\n");

    let mut xhc_device = None;
    let devices = pci::scan_all_bus().expect("Failed to scan PCI devices");
    for device in &devices {
        printk!("{}\n", device);
        if ((device.class_code >> 24) & 0xff) as u8 == 0x0c && ((device.class_code >> 16) & 0xff) as u8 == 0x03 && ((device.class_code >> 8) & 0xff) as u8 == 0x30 {
            xhc_device = Some(device);
            if device.vendor_id == 0x8086 {
                break;
            }
        }
    }
    xhc_device.expect("XHC Device is not found");
    let xhc_device = xhc_device.unwrap();
    printk!("xHC has been found: {}\n", xhc_device);
    let xhc_bar = xhc_device.read_bar(0).expect("Read bar error");
    printk!("xHC BAR0 = {:08x}\n", xhc_bar);
    let xhc_mmio_base = xhc_bar & !0xf;
    printk!("xHC mmio_base = {:08x}\n", xhc_mmio_base);

    unsafe {
        let xhc = usb::xhc_controller_new(xhc_mmio_base);
        xhc.init();
    }

    hlt_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
