use x86_64::structures::paging::PageSize;

const PAGE_SIZE_4K: u64 = 4096;
const PAGE_SIZE_2M: u64 = 512 * PAGE_SIZE_4K;
const PAGE_SIZE_1G: u64 = 512 * PAGE_SIZE_2M;

#[repr(align(4096))]
struct Pml4Table([u64; 512]);

#[repr(align(4096))]
struct PdpTable([u64; 512]);

#[repr(align(4096))]
struct PageDirectory([[u64; 512]; 64]);

static mut PML4_TABLE: Pml4Table = Pml4Table([0; 512]);
static mut PDP_TABLE: PdpTable = PdpTable([0; 512]);
static mut PAGE_DIRECTORY: PageDirectory = PageDirectory([[0; 512]; 64]);

pub fn init() {
    unsafe {
        PML4_TABLE.0[0] = (&PDP_TABLE.0[0] as *const u64 as u64) | 0x3;
        for (i, d) in PAGE_DIRECTORY.0.iter_mut().enumerate() {
            PDP_TABLE.0[i] = (d as *const u64 as u64) | 0x3;

            for (j, p) in PAGE_DIRECTORY.0[i].iter_mut().enumerate() {
                *p = i as u64 * PAGE_SIZE_1G + j as u64 * PAGE_SIZE_2M | 0x83;
            }
        }
        set_cr3(&PML4_TABLE.0[0] as *const u64 as u64);
    }
}

pub fn as_virt_addr(addr: x86_64::PhysAddr) -> Option<x86_64::VirtAddr> {
    if addr.as_u64() < x86_64::structures::paging::Size1GiB::SIZE * 64 {
        Some(x86_64::VirtAddr::new(addr.as_u64()))
    } else {
        None
    }
}

pub fn as_phys_addr(addr: x86_64::VirtAddr) -> Option<x86_64::PhysAddr> {
    if addr.as_u64() < x86_64::structures::paging::Size1GiB::SIZE * 64 {
        Some(x86_64::PhysAddr::new(addr.as_u64()))
    } else {
        None
    }
}

extern "C" {
    fn set_cr3(address: u64);
}
