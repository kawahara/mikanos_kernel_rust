#![warn(unsafe_op_in_unsafe_fn)]
#![no_std]

type MouseObserverType = extern "C" fn(displacement_x: i8, displacement_y: i8);

extern "C" {
    fn cxx_xhci_controller_new(xhc_mmio_base: u64) -> *mut XhciController;
    fn cxx_xhci_controller_initialize(xhc: *mut XhciController) -> i32;
    fn cxx_xhci_controller_run(xhc: *mut XhciController) -> i32;
    fn cxx_xhci_controller_configure_connected_ports(xhc: *mut XhciController);
    fn cxx_xhci_hid_mouse_driver_set_default_observer(observer: MouseObserverType);
    fn cxx_xhci_controller_process_event(xhc: *mut XhciController) -> i32;
}

pub enum XhciController {}

impl XhciController {
    pub unsafe fn new(xhc_mmio_base: u64) -> &'static mut XhciController {
        unsafe { &mut *cxx_xhci_controller_new(xhc_mmio_base) }
    }

    pub fn init(&mut self) -> i32 {
        unsafe { cxx_xhci_controller_initialize(self) }
    }

    pub fn run(&mut self) -> i32 {
        unsafe { cxx_xhci_controller_run(self) }
    }

    pub fn process_event(&mut self) -> i32 {
        unsafe { cxx_xhci_controller_process_event(self) }
    }

    pub fn configure_connected_ports(&mut self) {
        unsafe { cxx_xhci_controller_configure_connected_ports(self) }
    }
}

pub enum HidMouseDriver {}

pub type HidMouseObserver = extern "C" fn(displacement_x: i8, displacement_y: i8);

impl HidMouseDriver {
    pub fn set_default_observer(observer: HidMouseObserver) {
        unsafe { cxx_xhci_hid_mouse_driver_set_default_observer(observer) }
    }
}
