#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported");

use arguments::Args;
use ifaddrs::{getifaddrs, is_interface_up};
use std::{ffi, io};

mod errno;
#[macro_use]
mod macros;
mod bitflags;

pub mod arguments;
pub mod ifaddrs;

#[derive(Debug, Clone, Copy)]
pub struct NewtorkOnlineArguments<'a> {
    interfaces: Option<&'a [String]>,
}

/// Checks if network if online given the requirements provided by
/// `network_online_arguments`
///
/// # Errors
///
/// Will return `Err` if [`getifaddrs`] errors.
pub fn network_online(
    network_online_arguments: NewtorkOnlineArguments,
) -> Result<bool, io::Error> {
    let ret = if let Some(interface_names) = network_online_arguments.interfaces
    {
        getifaddrs()?.all(|ifaddr| {
            // up => true
            // down => in `interface_names`       => false
            //      => _not_ in `interface_names` => true
            is_interface_up(ifaddr)
                || !interface_names.iter().any(|interface_name| {
                    let ifa_name =
                        unsafe { ffi::CStr::from_ptr(ifaddr.ifa_name) };
                    interface_name.as_bytes() == ifa_name.to_bytes()
                })
        })
    } else {
        getifaddrs()?.all(is_interface_up)
    };

    Ok(ret)
}

impl<'a> From<&'a Args> for NewtorkOnlineArguments<'a> {
    fn from(args: &'a Args) -> Self {
        Self {
            interfaces: args.interface.as_deref(),
        }
    }
}
