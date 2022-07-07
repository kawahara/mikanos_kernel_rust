use crate::log;
use crate::logger::Level as LogLevel;
use crate::memory_manager::{memory_manager, FrameId};
use crate::paging::{as_phys_addr, as_virt_addr};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use spin::mutex::SpinMutex;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

enum AllocationMode {
    Block(usize),
    Frame(usize),
}

impl From<Layout> for AllocationMode {
    fn from(l: Layout) -> Self {
        let size = l.size().max(l.align());
        match BLOCK_SIZES.iter().position(|s| *s >= size) {
            Some(index) => Self::Block(index),
            None => Self::Frame((size + FrameId::SIZE - 1) / FrameId::SIZE),
        }
    }
}

pub struct KernelAllocator {
    blocks: SpinMutex<[*mut u8; BLOCK_SIZES.len()]>,
}

fn allocate_frame_for_block(index: usize) -> *mut u8 {
    let block_size = BLOCK_SIZES[index];
    let num_block_per_frame = FrameId::SIZE / block_size;
    let ptr: *mut u8 = match memory_manager().allocate(1) {
        Ok(frame) => as_virt_addr(frame.to_physical_address())
            .unwrap()
            .as_mut_ptr(),
        Err(_) => return ptr::null_mut(),
    };
    for i in 0..num_block_per_frame {
        let current = unsafe { ptr.add(i * block_size) };
        let next = if i == num_block_per_frame - 1 {
            ptr::null_mut()
        } else {
            unsafe { current.add(block_size) }
        };
        unsafe { (current as *mut u64).write(next as u64) };
    }
    ptr
}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {
            blocks: SpinMutex::new([ptr::null_mut(); BLOCK_SIZES.len()]),
        }
    }
}

unsafe impl Sync for KernelAllocator {}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match layout.into() {
            // 2048以下
            AllocationMode::Block(index) => {
                let mut blocks = self.blocks.lock();
                let mut ptr = blocks[index];
                if ptr.is_null() {
                    ptr = allocate_frame_for_block(index);
                }
                if !ptr.is_null() {
                    blocks[index] = (ptr as *const u64).read() as *mut u8;
                }
                log!(
                    LogLevel::Trace,
                    "allocator: allocator block (size = {}) -> {:?}\n",
                    BLOCK_SIZES[index],
                    x86_64::VirtAddr::from_ptr(ptr)
                );
                ptr
            }
            // フレーム数
            AllocationMode::Frame(num) => match memory_manager().allocate(num) {
                Ok(frame) => {
                    let phys_addr = frame.to_physical_address();
                    let virt_addr = as_virt_addr(phys_addr).unwrap();
                    log!(
                        LogLevel::Trace,
                        "allocator: allocator frame (num = {}) -> {:?}\n",
                        num,
                        virt_addr
                    );
                    virt_addr.as_mut_ptr()
                }
                Err(_) => ptr::null_mut(),
            },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match layout.into() {
            AllocationMode::Block(index) => {
                log!(
                    LogLevel::Trace,
                    "allocator: deallocate block (size = {}) -> {:?}\n",
                    BLOCK_SIZES[index],
                    x86_64::VirtAddr::from_ptr(ptr)
                );
                let mut blocks = self.blocks.lock();
                let next = blocks[index];
                (ptr as *mut u64).write(next as u64);
                blocks[index] = ptr;
            }
            AllocationMode::Frame(num) => {
                let addr = x86_64::VirtAddr::from_ptr(ptr as *const u8);
                let frame = FrameId::from_physical_address(as_phys_addr(addr).unwrap());
                log!(
                    LogLevel::Trace,
                    "allocator: deallocate frame (num = {}) -> {:?}\n",
                    num,
                    addr
                );
                memory_manager().free(frame, num);
            }
        }
    }
}

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator::new();

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error {:?}", layout)
}
