use std::{ffi, io, mem};

use crate::{
    errno,
    libc::{self},
};

pub use libc::ifaddrs;
pub use nix::net::if_::InterfaceFlags;

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
    #[allow(clippy::cast_sign_loss)]
    let mask = (InterfaceFlags::IFF_LOOPBACK.bits()
        | InterfaceFlags::IFF_LOWER_UP.bits()) as u32;

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

/// Checks if waiting is needed for provided `ifa_name`
///
/// # Safety
///
/// `ifa_name` must be a valid ptr from [`ifaddrs`]
pub unsafe fn check_require_or_ignore(
    ifa_name: *mut libc::c_char,
    require_or_irgnore_argument: InterfacesRequireOrIgnoreArgument,
) -> bool {
    // _not_ in interfaces => require => true
    //                     => ignore  => false
    //
    // _in_ interfaces     => require => false
    //                     => ignore  => true
    let ifa_name = ffi::CStr::from_ptr(ifa_name);
    let ifa_name = ifa_name.to_bytes();
    let in_interface = require_or_irgnore_argument
        .interfaces
        .iter()
        .any(|interface| interface.as_bytes() == ifa_name);

    (require_or_irgnore_argument.action == InterfacesActionArgument::Ignore)
        ^ in_interface
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfacesActionArgument {
    Ignore,
    Require,
}

#[derive(Debug, Clone, Copy)]
pub struct InterfacesRequireOrIgnoreArgument<'a> {
    pub interfaces: &'a [String],
    pub action: InterfacesActionArgument,
}

impl<'a> InterfacesRequireOrIgnoreArgument<'a> {
    #[must_use]
    pub const fn new(
        interfaces: &'a [String],
        action: InterfacesActionArgument,
    ) -> Self {
        Self { interfaces, action }
    }

    #[must_use]
    pub fn from_args(
        interface: Option<&'a [String]>,
        ignore: Option<&'a [String]>,
    ) -> Option<Self> {
        use InterfacesActionArgument::{Ignore, Require};
        type Iroia<'a> = InterfacesRequireOrIgnoreArgument<'a>;
        match (interface, ignore) {
            (None, None) => None,
            (Some(interfaces), None) => Some(Iroia::new(interfaces, Require)),
            (None, Some(interfaces)) => Some(Iroia::new(interfaces, Ignore)),
            _ => unreachable!(
                "`interfaces` and `ignore` can never be set at the same time"
            ),
        }
    }
    #[must_use]
    pub fn parse_args(
        interface: Option<&'a [String]>,
        ignore: Option<&'a [String]>,
    ) -> Option<(&'a [String], InterfacesActionArgument)> {
        use InterfacesActionArgument::{Ignore, Require};
        match (interface, ignore) {
            (None, None) => None,
            (Some(interfaces), None) => Some((interfaces, Require)),
            (None, Some(interfaces)) => Some((interfaces, Ignore)),
            _ => unreachable!(
                "`interfaces` and `ignore` can never be set at the same time"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::CString;

    use crate::libc;

    #[test]
    fn check_require_or_ignore_ignore() {
        let action = InterfacesActionArgument::Ignore;

        let interfaces = vec!["eth0".to_string()];
        let arg = InterfacesRequireOrIgnoreArgument::new(&interfaces, action);
        let combinations = [("eth0", arg, false), ("eth1", arg, true)];
        check_combinations(&combinations);

        let interfaces = vec!["eth0".to_string(), "eth1".to_string()];
        let action = InterfacesActionArgument::Ignore;
        let arg = InterfacesRequireOrIgnoreArgument::new(&interfaces, action);
        let combinations = [
            ("eth0", arg, false),
            ("eth1", arg, false),
            ("eth2", arg, true),
        ];

        check_combinations(&combinations);
    }

    #[test]
    fn check_require_or_ignore_require() {
        let action = InterfacesActionArgument::Require;

        let interfaces = vec!["eth0".to_string()];
        let arg = InterfacesRequireOrIgnoreArgument::new(&interfaces, action);
        let combinations = [("eth0", arg, true), ("eth1", arg, false)];
        check_combinations(&combinations);

        let interfaces = vec!["eth0".to_string(), "eth1".to_string()];
        let arg = InterfacesRequireOrIgnoreArgument::new(&interfaces, action);
        let combinations = [
            ("eth0", arg, true),
            ("eth1", arg, true),
            ("eth2", arg, false),
        ];

        check_combinations(&combinations);
    }

    fn check_combinations(
        combinations: &[(&str, InterfacesRequireOrIgnoreArgument, bool)],
    ) {
        for combination in combinations {
            let ifa_name = CString::new(combination.0).unwrap();
            let ifa_name_ptr: *mut libc::c_char =
                ifa_name.as_ptr().cast_mut().cast::<libc::c_char>();
            let ret =
                unsafe { check_require_or_ignore(ifa_name_ptr, combination.1) };
            assert_eq!(ret, combination.2);
        }
    }
}
