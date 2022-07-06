#![allow(dead_code)]

use core::mem;
use modular_bitfield::prelude::*;

pub fn init() {
    unsafe {
        GDT[1].initialize_code_segment(0);
        GDT[2].initialize_data_segment(0);
        load_gdt(
            (GDT.len() * mem::size_of::<SegmentDescriptor>() - 1) as u16,
            mem::transmute(&GDT[0]),
        );
        set_ds_all(0);
        set_csss(1 << 3, 2 << 3);
    }
}

static mut GDT: [SegmentDescriptor; 3] = [SegmentDescriptor::new(); 3];

#[derive(BitfieldSpecifier, Debug)]
#[bits = 4]
pub enum DescriptorType {
    Upper8Bytes = 0,
    LDT = 2,
    TSSAvailable = 9,
    ExecuteRead = 10,
    TSSBusy = 11,
    CallGate = 12,
    InterruptGate = 14,
    TrapGate = 15,
}

#[bitfield(bits = 64)]
#[derive(Debug, Clone, Copy)]
pub struct SegmentDescriptor {
    limit_low: B16,
    base_low: B16,
    base_middle: B8,
    #[bits = 4]
    descriptor_type: DescriptorType,
    system_segment: bool, // 0 = system segment, 1 = code or data segment
    dpl: B2,
    present: bool,
    limit_high: B4,
    available: bool,
    long_mode: bool,
    default_operation_size: bool,
    granularity: bool,
    base_high: B8,
}

impl SegmentDescriptor {
    fn initialize_code_segment(&mut self, descriptor_privilege_level: u8) {
        self.set_descriptor_type(DescriptorType::ExecuteRead);
        self.set_system_segment(true);
        self.set_dpl(descriptor_privilege_level);
        self.set_present(true);
        self.set_available(false);
        self.set_long_mode(true);
        self.set_default_operation_size(false);
    }

    fn initialize_data_segment(&mut self, descriptor_privilege_level: u8) {
        self.set_descriptor_type(DescriptorType::LDT);
        self.set_system_segment(true);
        self.set_dpl(descriptor_privilege_level);
        self.set_present(true);
        self.set_available(false);
        self.set_long_mode(false);
        self.set_default_operation_size(true);
    }
}

extern "C" {
    fn load_gdt(limit: u16, offset: *const u64);
    fn set_csss(cs: u16, ss: u16);
    fn set_ds_all(value: u16);
}
