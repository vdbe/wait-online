use std::{io, mem};

use crate::errno;

pub use crate::bitflags::InterfaceFlags;

/// Checks if all interfaces are up
///
/// # Errors
///
/// Will return `Err` if [`getifaddrs`] errors.
pub fn all_interfaces_up() -> Result<bool, io::Error> {
    Ok(getifaddrs()?.all(is_interface_up))
}

/// Checks if an interface is up
///
/// An interface is up when the flags [`InterfaceFlags::IFF_LOOPBACK`] _or_
/// [`InterfaceFlags::IFF_LOWER_UP`] are set.
///
/// `IFF_LOOPBACK` is seen as up because the `operstate` of an loopback
/// interface is always unkown.
///
/// `IFF_LOWER_UP` is used instead o `IFF_LOWER_UP` to match `ip addresses`
/// _oper states_ (see table below).
///
/// | `IFF_FLAG`       | oper state     |
/// | -------------- | -------------- |
/// | `IFF_UP`       | LOWERLAYERDOWN |
/// | `IFF_LOWER_UP` | UP             |
#[must_use]
pub const fn is_interface_up(ifaddr: libc::ifaddrs) -> bool {
    let mask =
        (InterfaceFlags::IFF_LOOPBACK | InterfaceFlags::IFF_LOWER_UP) as u32;

    (ifaddr.ifa_flags & mask) != 0
}

/// Get interfaces addresses using libc's [`libc::getifaddrs`].
///
/// # Errors
///
/// Will return `Err` if [`libc::getifaddrs`] errors.
/// For more info see [getifaddrs(3)](https://man7.org/linux/man-pages/man3/getifaddrs.3.html#ERRORS)
pub fn getifaddrs() -> Result<InterfaceAddressIterator, io::Error> {
    let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
    unsafe {
        let ret: libc::c_int = libc::getifaddrs(addrs.as_mut_ptr());
        if ret == -1 {
            return Err(errno::last());
        };

        Ok(InterfaceAddressIterator {
            base: addrs.assume_init(),
            next: addrs.assume_init(),
        })
    }
}

pub struct InterfaceAddressIterator {
    /// Head linked list returned by `ifaddrs`
    ///
    /// needed for [`libc::freeifaddrs()`].
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Drop for InterfaceAddressIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) };
    }
}

impl Iterator for InterfaceAddressIterator {
    type Item = libc::ifaddrs;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddr) => {
                self.next = ifaddr.ifa_next;
                Some(*ifaddr)
            }
            None => None,
        }
    }
}
