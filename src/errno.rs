use std::io;

use crate::libc;

pub fn last() -> io::Error {
    io::Error::from_raw_os_error(errno())
}

fn errno() -> i32 {
    unsafe { *errno_location() }
}

unsafe fn errno_location() -> *mut libc::c_int {
    libc::__errno_location()
}
