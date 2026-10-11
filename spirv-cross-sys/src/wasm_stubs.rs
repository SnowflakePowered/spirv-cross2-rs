//! Stubs for the WASI imports left over by wasi-libc when building for
//! `wasm32-unknown-unknown`. 

const ERRNO_SUCCESS: u32 = 0;
const ERRNO_BADF: u32 = 8;
const ERRNO_NOSYS: u32 = 52;

#[repr(C)]
pub struct Ciovec {
    buf: *const u8,
    buf_len: usize,
}

#[export_name = "__imported_wasi_snapshot_preview1_clock_time_get"]
pub extern "C" fn clock_time_get(_id: u32, _precision: u64, _out: *mut u64) -> u32 {
    ERRNO_NOSYS
}

#[export_name = "__imported_wasi_snapshot_preview1_fd_close"]
pub extern "C" fn fd_close(_fd: u32) -> u32 {
    ERRNO_SUCCESS
}

#[export_name = "__imported_wasi_snapshot_preview1_fd_seek"]
pub extern "C" fn fd_seek(_fd: u32, _offset: i64, _whence: u32, _out: *mut u64) -> u32 {
    ERRNO_BADF
}

/// Discards all output, reporting it as written.
#[export_name = "__imported_wasi_snapshot_preview1_fd_write"]
pub unsafe extern "C" fn fd_write(
    _fd: u32,
    iovs: *const Ciovec,
    iovs_len: usize,
    nwritten: *mut usize,
) -> u32 {
    let mut total = 0;
    for i in 0..iovs_len {
        total += (*iovs.add(i)).buf_len;
    }
    *nwritten = total;
    ERRNO_SUCCESS
}

/// Reports an empty environment.
#[export_name = "__imported_wasi_snapshot_preview1_environ_sizes_get"]
pub unsafe extern "C" fn environ_sizes_get(count: *mut usize, buf_size: *mut usize) -> u32 {
    *count = 0;
    *buf_size = 0;
    ERRNO_SUCCESS
}

#[export_name = "__imported_wasi_snapshot_preview1_environ_get"]
pub extern "C" fn environ_get(_environ: *mut *mut u8, _buf: *mut u8) -> u32 {
    ERRNO_SUCCESS
}

/// Reached through `abort()`, so trap.
#[export_name = "__imported_wasi_snapshot_preview1_proc_exit"]
pub extern "C" fn proc_exit(_code: u32) -> ! {
    core::arch::wasm32::unreachable()
}
