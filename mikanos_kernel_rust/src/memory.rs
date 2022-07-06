use core::ffi::c_void;
use core::fmt::Formatter;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MemoryMap {
    pub buffer_size: u64,
    pub buffer: *const c_void,
    pub map_size: u64,
    pub map_key: u64,
    pub descriptor_size: u64,
    pub descriptor_version: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MemoryDescriptor {
    pub memory_type: MemoryType,
    pub physical_start: *const usize,
    pub virtual_start: *const usize,
    pub number_of_pages: u64,
    pub attribute: u64,
}

impl MemoryDescriptor {
    pub fn physical_end(&self) -> *const usize {
        unsafe {
            self.physical_start
                .add(self.number_of_pages as usize * 4096)
        }
    }
}

impl core::fmt::Display for MemoryDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "type={:?}, phys = {:?} - {:?}, pages = {}, attr = {:08x}",
            self.memory_type,
            self.physical_start,
            unsafe { self.physical_end().sub(1) },
            self.number_of_pages,
            self.attribute
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    EfiReservedMemoryType = 0,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiMaxMemoryType,
}
