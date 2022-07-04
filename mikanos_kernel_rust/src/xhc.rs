use crate::logger::Level as LogLevel;
use crate::pci::{Device, Devices, MsiDeliveryMode, MsiTriggerMode};
use crate::sync::once_cell::OnceCell;
use crate::{log, mouse, pci};
use core::option::Option::{None, Some};
use mikanos_usb_driver::{HidMouseDriver, XhciController};
use spin::mutex::SpinMutex;
use volatile::Volatile;
use x86_64::structures::idt::InterruptStackFrame;

static XHC: OnceCell<SpinMutex<&'static mut XhciController>> = OnceCell::uninit();

fn notify_end_of_interrupt() {
    let mut memory = Volatile::new(unsafe { (0xfee000b0 as *mut u32).as_mut().unwrap() });
    memory.write(0);
}

pub extern "x86-interrupt" fn xhc_interrupt_handler(_stack_frame: InterruptStackFrame) {
    if let Some(mut xhc) = XHC.get().try_lock() {
        while xhc.has_event() {
            xhc.process_event();
        }
    }
    notify_end_of_interrupt();
}

pub fn init(devices: &Devices) -> Result<(), ()> {
    let mut xhc_device: Option<&Device> = None;
    for device in devices {
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

    // MSI Config
    let bsp_local_apic_id = unsafe { *(0xfee00020 as *const u32) } >> 24;
    pci::configure_msi_fixed_destination(
        xhc_device,
        bsp_local_apic_id,
        MsiTriggerMode::Level,
        MsiDeliveryMode::Fixed,
        0x40,
        0,
    )?;

    let xhc_bar = xhc_device.read_bar(0).expect("Read bar error");
    log!(LogLevel::Info, "xHC BAR0 = {:08x}\n", xhc_bar);
    let xhc_mmio_base = xhc_bar & !0xf;
    log!(LogLevel::Info, "xHC mmio_base = {:08x}\n", xhc_mmio_base);

    let xhc = unsafe { XhciController::new(xhc_mmio_base) };
    if xhc_device.vendor_id == 0x8086 {
        pci::switch_ehci_to_xhci(&xhc_device, &devices);
    }
    xhc.init();
    log!(LogLevel::Info, "xHC init\n");
    xhc.run();
    log!(LogLevel::Info, "xHC starting\n");

    HidMouseDriver::set_default_observer(mouse::mouse_observer);

    xhc.configure_connected_ports();

    XHC.init_once(move || SpinMutex::new(xhc));

    Ok(())
}
