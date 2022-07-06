#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod console;
pub mod cxx_support;
pub mod fonts;
pub mod graphics;
pub mod interrupt;
pub mod logger;
pub mod memory;
pub mod memory_manager;
pub mod mouse;
pub mod paging;
pub mod pci;
pub mod segments;
pub mod sync;
pub mod xhc;

use crate::console::initialize_console;
use crate::graphics::{FrameBuffer, Graphics, PixelColor, Vector2D};
use crate::logger::Level as LogLevel;
use crate::memory::{MemoryDescriptor, MemoryMap, MemoryType};
use crate::memory_manager::{BitmapMemoryManager, FrameId};
use core::panic::PanicInfo;

fn hlt_loop() {
    loop {
        x86_64::instructions::hlt();
    }
}

static mut MEMORY_MANAGER: BitmapMemoryManager = BitmapMemoryManager::new();

#[no_mangle]
extern "C" fn kernel_main2(fb: *mut FrameBuffer, mc: *const MemoryMap) {
    let fb_a = unsafe { *fb };
    let bg_color = PixelColor(45, 118, 237);
    let fg_color = PixelColor(255, 255, 255);
    let mut graphics = Graphics::new(fb_a);
    initialize_console(&graphics, &fg_color, &bg_color);
    logger::set_level(LogLevel::Info);

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

    segments::init();
    paging::init();
    let mc = unsafe { *mc };
    // setup memory manager
    let mut phys_available_end: usize = 0;
    let mut iter = mc.buffer;
    while iter < unsafe { mc.buffer.add(mc.map_size as usize) } {
        let desc = unsafe { *(iter as *const MemoryDescriptor) };
        let phys_start = desc.physical_start as usize;
        let phys_end = desc.physical_end() as usize;

        if phys_available_end < phys_start {
            unsafe {
                MEMORY_MANAGER.mark_allocated_in_bytes(
                    &FrameId::from_physical_address(phys_available_end),
                    phys_start - phys_available_end,
                )
            }
        }

        if desc.memory_type == MemoryType::EfiBootServicesCode
            || desc.memory_type == MemoryType::EfiBootServicesData
            || desc.memory_type == MemoryType::EfiConventionalMemory
        {
            phys_available_end = phys_end;
        } else {
            unsafe {
                MEMORY_MANAGER.mark_allocated_in_bytes(
                    &FrameId::from_physical_address(phys_start),
                    phys_end - phys_start,
                )
            }
        }

        iter = unsafe { iter.add(mc.descriptor_size as usize) };
    }
    unsafe {
        MEMORY_MANAGER.set_memory_range(
            FrameId::MIN,
            FrameId::from_physical_address(phys_available_end),
        );
    }

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
