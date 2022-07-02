#![no_std]
#![no_main]

pub mod console;
pub mod cxx_support;
pub mod fonts;
pub mod graphics;
pub mod logger;
pub mod mouse;
pub mod pci;

use console::initialize_console;
use core::option::Option::{None, Some};
use core::panic::PanicInfo;
use graphics::{FrameBuffer, Graphics, PixelColor, Vector2D};
use logger::Level as LogLevel;
use mikanos_usb_driver as usb;
use mouse::MouseCursor;

static mut CURSOR: Option<MouseCursor> = None;

extern "C" fn mouse_observer(displacement_x: i8, displacement_y: i8) {
    // printk!("{}, {}\n", displacement_x, displacement_y);
    unsafe {
        CURSOR.as_mut().unwrap().move_relative(&Vector2D::<isize> {
            x: displacement_x as isize,
            y: displacement_y as isize,
        });
    }
}

#[no_mangle]
extern "C" fn kernel_main(fb: *mut FrameBuffer) {
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

    unsafe {
        CURSOR = Some(MouseCursor::new(
            &graphics,
            &Vector2D::<usize> { x: 200, y: 100 },
            &bg_color,
        ));
    }

    printk!("Welcome to MikanOS Rust!!\n");
    log!(LogLevel::Info, "Load PCI devices\n");

    let mut xhc_device = None;
    let devices = pci::scan_all_bus().expect("Failed to scan PCI devices");
    for device in &devices {
        log!(LogLevel::Info, "{}\n", device);
        if ((device.class_code >> 24) & 0xff) as u8 == 0x0c
            && ((device.class_code >> 16) & 0xff) as u8 == 0x03
            && ((device.class_code >> 8) & 0xff) as u8 == 0x30
        {
            xhc_device = Some(device);
            if device.vendor_id == 0x8086 {
                break;
            }
        }
    }
    xhc_device.expect("XHC Device is not found");
    let xhc_device = xhc_device.unwrap();
    log!(LogLevel::Info, "xHC has been found: {}\n", xhc_device);
    let xhc_bar = xhc_device.read_bar(0).expect("Read bar error");
    log!(LogLevel::Info, "xHC BAR0 = {:08x}\n", xhc_bar);
    let xhc_mmio_base = xhc_bar & !0xf;
    log!(LogLevel::Info, "xHC mmio_base = {:08x}\n", xhc_mmio_base);

    let xhc = unsafe { usb::XhciController::new(xhc_mmio_base) };
    if xhc_device.vendor_id == 0x8086 {
        // TODO: switch EHCI to XHCI
    }
    xhc.init();
    log!(LogLevel::Info, "xHC init\n");
    xhc.run();
    log!(LogLevel::Info, "xHC starting\n");
    xhc.configure_connected_ports();
    usb::HidMouseDriver::set_default_observer(mouse_observer);
    loop {
        xhc.process_event();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printk!("Panic!! {}", info);

    loop {}
}
