use core::ptr;

extern "C" {
    fn __errno() -> *mut i32;
}

#[allow(non_camel_case_types)]
type pid_t = i32;
const EBADF: i32 = 9;
const ENOMEM: i32 = 12;
const EINVAL: i32 = 22;

#[no_mangle]
extern "C" fn sbrk(_increment: isize) -> *const u8 {
    ptr::null()
}

#[no_mangle]
extern "C" fn _exit() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
extern "C" fn kill(_pid: pid_t, _sig: i32) -> i32 {
    unsafe {
        *__errno() = EINVAL;
    }
    -1
}

#[no_mangle]
extern "C" fn getpid() -> pid_t {
    unsafe {
        *__errno() = EINVAL;
    }
    -1
}

#[no_mangle]
extern "C" fn close() -> i32 {
    unsafe {
        *__errno() = EBADF;
    }
    -1
}

#[no_mangle]
extern "C" fn read(_fd: i32, _buf: *mut u8, _count: usize) -> isize {
    unsafe {
        *__errno() = EBADF;
    }
    -1
}

#[no_mangle]
extern "C" fn write(_fd: i32, _buf: *const u8, _count: usize) -> isize {
    unsafe {
        *__errno() = EBADF;
    }
    -1
}

#[no_mangle]
extern "C" fn lseek(_fd: i32, _offset: isize, _whence: i32) -> isize {
    unsafe {
        *__errno() = EBADF;
    }
    -1
}

#[no_mangle]
extern "C" fn fstat(_fd: i32, _buf: *mut u8) -> i32 {
    unsafe {
        *__errno() = EBADF;
    }
    -1
}

#[no_mangle]
extern "C" fn isatty(_fd: i32) -> i32 {
    unsafe {
        *__errno() = EBADF;
    }
    -1
}

#[no_mangle]
extern "C" fn posix_memalign(_memptr: *mut *mut u8, _alignment: usize, _size: usize) -> i32 {
    ENOMEM
}
