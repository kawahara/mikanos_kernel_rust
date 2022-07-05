use core::ffi::c_void;

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
