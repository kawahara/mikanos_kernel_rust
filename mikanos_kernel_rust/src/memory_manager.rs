use crate::{printk, MemoryDescriptor, MemoryMap, MemoryType};
use core::mem;
use spin::mutex::{SpinMutex, SpinMutexGuard};

// 128GiB
const MAX_PHYSICAL_MEMORY_BYTES: usize = 128 * 1024 * 1024 * 1024;
const FRAME_COUNT: usize = MAX_PHYSICAL_MEMORY_BYTES / FrameId::SIZE;

type MapLine = usize;

const BITS_PER_MAP_LINE: usize = 8 * mem::size_of::<MapLine>();
const MAP_LINE_COUNT: usize = FRAME_COUNT / BITS_PER_MAP_LINE;

#[derive(Debug)]
pub struct FrameId(usize);

impl FrameId {
    pub fn from_physical_address(addr: x86_64::PhysAddr) -> Self {
        Self(addr.as_u64() as usize / FrameId::SIZE)
    }

    pub fn to_physical_address(&self) -> x86_64::PhysAddr {
        x86_64::PhysAddr::new((self.0 * FrameId::SIZE) as u64)
    }

    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(FRAME_COUNT);

    // 4KiB
    pub const SIZE: usize = 4096;
}

pub struct BitmapMemoryManager {
    alloc_map: [MapLine; MAP_LINE_COUNT],
    pub begin: FrameId,
    pub end: FrameId,
}

impl BitmapMemoryManager {
    pub const fn new() -> Self {
        Self {
            alloc_map: [0; MAP_LINE_COUNT],
            begin: FrameId::MIN,
            end: FrameId::MAX,
        }
    }

    pub fn allocate(&mut self, num_frames: usize) -> Result<FrameId, ()> {
        let mut start_frame_id = self.begin.0;
        'search: loop {
            for i in 0..num_frames {
                if start_frame_id + 1 >= self.end.0 {
                    // フレームがもうない
                    return Err(())?;
                }

                if self.get_bit(FrameId(start_frame_id + i)) {
                    // 使用中なので次を検索
                    start_frame_id = start_frame_id + i + 1;
                    continue 'search;
                }
            }
            let new_start_frame_id = FrameId(start_frame_id);
            self.mark_allocated(&new_start_frame_id, num_frames);
            return Ok(new_start_frame_id);
        }
    }

    pub fn free(&mut self, start_frame: FrameId, num_frames: usize) {
        for i in 0..num_frames {
            self.set_bit(FrameId(start_frame.0 + i), false);
        }
    }

    pub fn mark_allocated(&mut self, start_frame: &FrameId, num_frames: usize) {
        for i in 0..num_frames {
            self.set_bit(FrameId(start_frame.0 + i), true);
        }
    }

    pub fn mark_allocated_in_bytes(&mut self, start_frame: &FrameId, bytes: usize) {
        self.mark_allocated(start_frame, bytes / FrameId::SIZE);
    }

    pub fn set_memory_range(&mut self, range_begin: FrameId, range_end: FrameId) {
        self.begin = range_begin;
        self.end = range_end;
    }

    pub fn get_bit(&self, frame: FrameId) -> bool {
        let line_index = frame.0 / BITS_PER_MAP_LINE;
        let bit_index = frame.0 % BITS_PER_MAP_LINE;

        (self.alloc_map[line_index] & (1 << bit_index)) != 0
    }

    pub fn set_bit(&mut self, frame: FrameId, allocated: bool) {
        let line_index = frame.0 / BITS_PER_MAP_LINE;
        let bit_index = frame.0 % BITS_PER_MAP_LINE;

        if allocated {
            self.alloc_map[line_index] |= 1 << bit_index;
        } else {
            self.alloc_map[line_index] &= !(1 << bit_index);
        }
    }
}

static MEMORY_MANAGER: SpinMutex<BitmapMemoryManager> = SpinMutex::new(BitmapMemoryManager::new());

pub fn memory_manager() -> SpinMutexGuard<'static, BitmapMemoryManager> {
    MEMORY_MANAGER.lock()
}

pub fn init(mc: &MemoryMap) {
    let mut mm = MEMORY_MANAGER.try_lock().unwrap();

    let mut phys_available_end: usize = 0;
    let mut iter = mc.buffer;

    while iter < unsafe { mc.buffer.add(mc.map_size as usize) } {
        let desc = unsafe { *(iter as *const MemoryDescriptor) };
        let phys_start = desc.physical_start as usize;
        let phys_end = desc.physical_end() as usize;

        if phys_available_end < phys_start {
            mm.mark_allocated_in_bytes(
                &FrameId::from_physical_address(x86_64::PhysAddr::new(phys_available_end as u64)),
                phys_start - phys_available_end,
            )
        }

        if desc.memory_type == MemoryType::EfiBootServicesCode
            || desc.memory_type == MemoryType::EfiBootServicesData
            || desc.memory_type == MemoryType::EfiConventionalMemory
        {
            phys_available_end = phys_end;
        } else {
            mm.mark_allocated_in_bytes(
                &FrameId::from_physical_address(x86_64::PhysAddr::new(phys_start as u64)),
                phys_end - phys_start,
            )
        }

        iter = unsafe { iter.add(mc.descriptor_size as usize) };
    }
    mm.set_memory_range(
        FrameId::MIN,
        FrameId::from_physical_address(x86_64::PhysAddr::new(phys_available_end as u64)),
    );
}
