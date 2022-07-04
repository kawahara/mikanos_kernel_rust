use crate::sync::once_cell::OnceCell;
use crate::xhc;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static IDT: OnceCell<InterruptDescriptorTable> = OnceCell::uninit();

extern "x86-interrupt" fn breakpoint_handler(_stack_name: InterruptStackFrame) {}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub fn init() {
    IDT.init_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[0x40 as usize].set_handler_fn(xhc::xhc_interrupt_handler);
        idt
    });
    IDT.get().load();
}
