#![warn(unsafe_op_in_unsafe_fn)]
#![no_std]

extern "C" {
    fn cxx_xhc_controller_new(xhc_mmio_base: u64) -> *mut XhciController;
    fn cxx_xhc_controller_initialize(xhc: *mut XhciController);
}

pub enum XhciController {}

pub unsafe fn xhc_controller_new(xhc_mmio_base: u64) -> &'static mut XhciController {
    unsafe { &mut *cxx_xhc_controller_new(xhc_mmio_base) }
}

impl XhciController {
    pub unsafe fn init(&'static mut self) {
        unsafe { cxx_xhc_controller_initialize(self) }
    }
}




