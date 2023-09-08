#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported");

use std::{collections::HashMap, ffi};

use arguments::Args;
use ifaddrs::{
    check_require_or_ignore, is_interface_up, InterfaceFlags,
    InterfacesActionArgument, InterfacesRequireOrIgnoreArgument,
};
use sockaddr::{
    check_family_type, get_addres_family, AddressFamily,
    InterfacesFamilyTypeArgument,
};

// Re-exported external crates
pub use nix::libc;

mod errno;

pub mod arguments;
pub mod ifaddrs;
pub mod operstate;
pub mod sockaddr;

#[derive(Debug, Clone, Copy, Default)]
struct InterfacesArgument<'a> {
    require_or_ignore: Option<InterfacesRequireOrIgnoreArgument<'a>>,
    family_type: Option<InterfacesFamilyTypeArgument>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkArgument<'a> {
    interfaces_argument: Option<InterfacesArgument<'a>>,
    exact: bool,
    any: bool,
}

type InterfaceMap<'a> = HashMap<&'a [u8], Option<(bool, bool)>>;

/// Checks if network if online given the requirements provided by
/// `network_online_arguments`
pub fn network_online<I>(
    mut ifaddrs: I,
    network_argument: NetworkArgument,
) -> bool
where
    I: Iterator<Item = ifaddrs::ifaddrs>,
{
    match (
        network_argument.exact,
        network_argument.any,
        network_argument.interfaces_argument,
    ) {
        (_, false, None) => ifaddrs.all(is_interface_up),
        (_, true, None) => ifaddrs.any(is_interface_up),
        (true, any, Some(interface_argument)) => {
            network_online_exact(ifaddrs, any, interface_argument)
        }
        (false, any, Some(interface_argument)) => {
            network_online_lazy(ifaddrs, any, interface_argument)
        }
    }
}

fn network_online_lazy<I>(
    mut ifaddrs: I,
    any: bool,
    interface_argument: InterfacesArgument<'_>,
) -> bool
where
    I: Iterator<Item = ifaddrs::ifaddrs>,
{
    if any {
        ifaddrs
            .any(|ifaddr| is_interface_online_lazy(ifaddr, interface_argument))
    } else {
        ifaddrs
            .all(|ifaddr| is_interface_online_lazy(ifaddr, interface_argument))
    }
}

fn network_online_exact<I>(
    mut ifaddrs: I,
    any: bool,
    interface_argument: InterfacesArgument<'_>,
) -> bool
where
    I: Iterator<Item = ifaddrs::ifaddrs>,
{
    if any {
        return ifaddrs.any(|ifaddr| {
            is_interface_online_exact(ifaddr, interface_argument, None)
                .unwrap_or(false)
        });
    }

    let mut map: InterfaceMap=
        // Prepopulate Hashmap with required interfaces
        if let InterfacesArgument {
            require_or_ignore:
                Some(InterfacesRequireOrIgnoreArgument {
                    interfaces,
                    action: InterfacesActionArgument::Require,
                }),
            ..
        } = interface_argument
        {
            interfaces.iter().map(|name| (name.as_bytes(), None)).collect()
        } else {
            HashMap::default()
        };

    let all_up = ifaddrs
        .filter_map(|ifaddr| {
            is_interface_online_exact(
                ifaddr,
                interface_argument,
                Some(&mut map),
            )
        })
        .all(|x| x);

    let all_present = map.values().all(|value| {
        if let Some((has_ipv4, has_ipv6)) = value {
            if let InterfacesArgument {
                family_type:
                    Some(InterfacesFamilyTypeArgument {
                        ipv4: require_ipv4,
                        ipv6: require_ipv6,
                    }),
                ..
            } = interface_argument
            {
                return *has_ipv4 && *has_ipv6
                    || ((require_ipv4 ^ require_ipv6)
                        && (require_ipv4 && *has_ipv4
                            || require_ipv6 && *has_ipv6));
            }
            return *has_ipv4 || *has_ipv6;
        }

        false
    });
    all_up && all_present
}

fn is_interface_online_lazy(
    ifaddr: ifaddrs::ifaddrs,
    interfaces_argument: InterfacesArgument,
) -> bool {
    #[allow(clippy::cast_sign_loss)]
    let mask = (InterfaceFlags::IFF_LOOPBACK.bits()
        | InterfaceFlags::IFF_LOWER_UP.bits()) as u32;

    ifaddr.ifa_flags & mask != 0
        || interfaces_argument
            .family_type
            .map_or(false, |family_arg|
                // SAFETY: We know `ifa_addr` is a valid or null ptr from `ifaddr`
                unsafe {
                    !check_family_type(ifaddr.ifa_addr, family_arg)
                }
            )
        || interfaces_argument
            .require_or_ignore
            .map_or(false, |require_or_ignore_arg|
                // SAFETY: We know `ifa_name` if a valid ptr from `ifaddr`
                unsafe {
                        !check_require_or_ignore(ifaddr.ifa_name, require_or_ignore_arg)
                },
            )
}

fn is_interface_online_exact(
    ifaddr: libc::ifaddrs,
    interface_argument: InterfacesArgument<'_>,
    map: Option<&mut InterfaceMap>,
) -> Option<bool> {
    #[allow(clippy::cast_sign_loss)]
    let mask = (InterfaceFlags::IFF_LOOPBACK.bits()
        | InterfaceFlags::IFF_LOWER_UP.bits()) as u32;

    // SAFETY: We know `ifa_name` if a valid ptr from `ifaddr`
    let ifa_name = unsafe { ffi::CStr::from_ptr(ifaddr.ifa_name) };
    // SAFETY: We know `ifa_addr` is a valid or null ptr from `ifaddr`
    let ifa_addr_family = unsafe { get_addres_family(ifaddr.ifa_addr) };

    let interface_up = ifaddr.ifa_flags & mask != 0;
    let correct_family: Option<AddressFamily> = interface_argument
        .family_type
        .map_or(ifa_addr_family, |family_arg| match ifa_addr_family {
            Some(AddressFamily::Inet) if family_arg.ipv4 => {
                Some(AddressFamily::Inet)
            }
            Some(AddressFamily::Inet6) if family_arg.ipv6 => {
                Some(AddressFamily::Inet6)
            }
            _ => None,
        });
    let (correct_name, insert): (bool, bool) = interface_argument
        .require_or_ignore
        .map_or((true, true), |require_or_ignore_arg| {
            let in_interface = require_or_ignore_arg
                .interfaces
                .iter()
                .any(|interface| interface.as_bytes() == ifa_name.to_bytes());

            let a = require_or_ignore_arg.action
                == InterfacesActionArgument::Ignore;

            (a ^ in_interface, a)
        });

    // Insert interface into the hash map if needed
    if correct_name {
        if let Some(map) = map {
            let current = match ifa_addr_family {
                Some(AddressFamily::Inet) => (true, false),
                Some(AddressFamily::Inet6) => (false, true),
                _ => (false, false),
            };

            #[allow(clippy::cast_sign_loss, clippy::option_if_let_else)]
            if let Some(value) = map.get_mut(ifa_name.to_bytes()) {
                if let Some(prev) = value {
                    prev.0 |= current.0;
                    prev.1 |= current.1;
                } else {
                    *value = Some(current);
                }
            } else if insert
                && (ifaddr.ifa_flags
                    & InterfaceFlags::IFF_LOOPBACK.bits() as u32)
                    == 0
            {
                // Insert newly found interface into map
                // except for when the required flag is used
                // or it's a loopback interface
                _ = map.insert(ifa_name.to_bytes(), Some(current));
            }
        }
    };

    match (correct_family.is_some(), correct_name) {
        (true, true) => Some(interface_up),
        _ => None,
    }
}

impl<'a> InterfacesArgument<'a> {
    fn from_args(args: &'a Args) -> (bool, Option<Self>) {
        let require_or_ignore = InterfacesRequireOrIgnoreArgument::parse_args(
            args.interface.as_deref(),
            args.ignore.as_deref(),
        );
        let family_type =
            InterfacesFamilyTypeArgument::from_args(args.ipv4, args.ipv6);

        match (require_or_ignore, family_type) {
            (None, None) => (false, None),
            (require_or_ignore, Some(family_type)) => (
                true,
                Some(InterfacesArgument {
                    require_or_ignore: require_or_ignore.map(
                        |(interfaces, action)| {
                            InterfacesRequireOrIgnoreArgument::new(
                                interfaces, action,
                            )
                        },
                    ),
                    family_type: Some(family_type),
                }),
            ),
            (Some((interfaces, action)), None) => (
                action == InterfacesActionArgument::Require,
                Some(InterfacesArgument {
                    require_or_ignore: Some(
                        InterfacesRequireOrIgnoreArgument::new(
                            interfaces, action,
                        ),
                    ),
                    family_type,
                }),
            ),
        }
    }
}

impl<'a> From<&'a Args> for NetworkArgument<'a> {
    fn from(args: &'a Args) -> Self {
        let (exact, interfaces_argument) = InterfacesArgument::from_args(args);
        Self {
            interfaces_argument,
            exact,
            any: args.any,
        }
    }
}

#[allow(clippy::field_reassign_with_default)]
#[cfg(test)]
mod tests {
    use super::*;

    use core::slice;
    use std::{ffi::CString, ptr};

    use libc::{c_char, sa_family_t};
    use nix::sys::socket::AddressFamily;

    use crate::{
        ifaddrs::{ifaddrs, InterfaceFlags as F},
        sockaddr::sockaddr,
    };

    const FLAGS_LOOPBACK: i32 = F::IFF_UP.bits()
        | F::IFF_LOOPBACK.bits()
        | F::IFF_RUNNING.bits()
        | F::IFF_LOWER_UP.bits();

    const FLAGS_LOWER_LAYWER_DOWN: i32 =
        F::IFF_UP.bits() | F::IFF_BROADCAST.bits() | F::IFF_MULTICAST.bits();

    const FLAGS_UP: i32 = F::IFF_UP.bits()
        | F::IFF_BROADCAST.bits()
        | F::IFF_RUNNING.bits()
        | F::IFF_MULTICAST.bits()
        | F::IFF_LOWER_UP.bits();

    #[test]
    fn online() {
        let n_args = NetworkArgument::default();

        let mut v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
        ];
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
        );
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.pop();
        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Packet),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_implicit_ignore_loopback() {
        let mut args = Args::default();
        args.ipv4 = true;
        let n_args = NetworkArgument::from(&args);

        let v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet),
        ];
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_require_loopback() {
        let mut args = Args::default();
        args.ipv4 = true;
        args.interface = Some(vec!["lo".into()]);
        let n_args = NetworkArgument::from(&args);

        let mock_ifaddr = MockIfaddrs::new()
            .name("lo")
            .flags(FLAGS_LOOPBACK)
            .sockaddr(AddressFamily::Packet);

        let mut v = vec![mock_ifaddr.sockaddr(AddressFamily::Packet)];
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        let mock_ifaddr = MockIfaddrs::new()
            .name("lo")
            .flags(FLAGS_LOOPBACK)
            .sockaddr(AddressFamily::Packet);
        v.push(mock_ifaddr.sockaddr(AddressFamily::Inet));
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_family_type_v4() {
        let mut args = Args::default();
        args.ipv4 = true;
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet)];
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet),
        );
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.pop();
        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet6),
        );
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_family_type_v6() {
        let mut args = Args::default();
        args.ipv6 = true;
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet6)];
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet6),
        );
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.pop();
        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet),
        );
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet6),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_require_single() {
        let mut args = Args::default();
        args.interface = Some(vec!["eth1".into()]);
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
        ];

        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.pop();
        v.push(MockIfaddrs::new().name("eth1").flags(FLAGS_UP));
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_require_missing() {
        let mut args = Args::default();
        args.interface = Some(vec!["eth999999".into()]);
        let n_args = NetworkArgument::from(&args);

        let v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
        ];
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_require_multi() {
        let mut args = Args::default();
        args.interface = Some(vec!["eth1".into(), "eth2".into()]);
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet),
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet),
        ];

        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v[0] = MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_LOWER_LAYWER_DOWN)
            .sockaddr(AddressFamily::Inet);
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet);
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_LOWER_LAYWER_DOWN)
            .sockaddr(AddressFamily::Inet);
        v[2] = MockIfaddrs::new()
            .name("eth2")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet);
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet);
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v[0] = MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_LOWER_LAYWER_DOWN)
            .sockaddr(AddressFamily::Inet);
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_ignore_single() {
        let mut args = Args::default();
        args.ignore = Some(vec!["eth0".into()]);
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![
            MockIfaddrs::new()
                .name("lo")
                .flags(FLAGS_LOOPBACK)
                .sockaddr(AddressFamily::Inet6),
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet6),
        ];

        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet);
        v[2] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_LOWER_LAYWER_DOWN)
            .sockaddr(AddressFamily::Inet6);
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn online_ignore_multi() {
        let mut args = Args::default();
        args.ignore = Some(vec!["eth1".into(), "eth2".to_string()]);
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet6),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet6),
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_LOWER_LAYWER_DOWN)
                .sockaddr(AddressFamily::Inet6),
        ];

        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v[0] = MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet6);
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet6);
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_LOWER_LAYWER_DOWN)
            .sockaddr(AddressFamily::Inet6);
        v[2] = MockIfaddrs::new()
            .name("eth2")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet6);
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v[1] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_UP)
            .sockaddr(AddressFamily::Inet6);
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }
    #[test]
    fn online_any() {
        let mut args = Args::default();
        args.any = true;
        let n_args = NetworkArgument::from(&args);

        let mut v = vec![MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_LOWER_LAYWER_DOWN)
            .sockaddr(AddressFamily::Inet6)];

        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet6),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        args.ipv4 = true;
        let n_args = NetworkArgument::from(&args);
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Packet),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[derive(Default, Debug)]
    struct MockSockaddr {
        sa_family: sa_family_t,
        sa_data: [c_char; 14],
    }

    #[allow(unused)]
    #[derive(Default, Debug)]
    struct MockIfaddrs {
        ifa_name: CString,
        ifa_flags: i32,
        ifa_addr: MockSockaddr,
        ifa_netmask: MockSockaddr,
        ifa_ifu: MockSockaddr,
        ifa_data: Option<Vec<u8>>,
    }

    struct MockIfaddrsIterator<'a> {
        iterator: slice::Iter<'a, MockIfaddrs>,
        raw_scokaddrs: Vec<*mut sockaddr>,
    }

    impl<'a> Iterator for MockIfaddrsIterator<'a> {
        type Item = ifaddrs;

        fn next(&mut self) -> Option<Self::Item> {
            self.iterator.next().map(|mock_ifaddrs| {
                let ifaddrs = mock_ifaddrs
                    .make_ifaddrs(ptr::null::<ifaddrs>().cast_mut());
                self.raw_scokaddrs.push(ifaddrs.ifa_addr);
                self.raw_scokaddrs.push(ifaddrs.ifa_netmask);
                self.raw_scokaddrs.push(ifaddrs.ifa_ifu);
                ifaddrs
            })
        }
    }

    impl<'a> MockIfaddrsIterator<'a> {
        fn new(v: &'a [MockIfaddrs]) -> Self {
            Self {
                iterator: v.iter(),
                raw_scokaddrs: Vec::new(),
            }
        }
    }

    impl MockSockaddr {
        const fn as_sockaddr(&self) -> sockaddr {
            sockaddr {
                sa_family: self.sa_family,
                sa_data: self.sa_data,
            }
        }
    }

    impl<'a> MockIfaddrs {
        fn new() -> Self {
            Self::default()
        }

        fn name<S: ToString + ?Sized>(mut self, name: &S) -> Self {
            self.ifa_name = CString::new(name.to_string()).unwrap();
            self
        }

        const fn flags(mut self, flags: i32) -> Self {
            self.ifa_flags = flags;
            self
        }

        fn sockaddr(mut self, family: AddressFamily) -> Self {
            self.ifa_addr = MockSockaddr {
                sa_family: libc::sa_family_t::try_from(family as i32).unwrap(),
                sa_data: [libc::c_char::default(); 14],
            };
            self
        }

        fn make_ifaddrs(&'a self, next: *mut ifaddrs) -> ifaddrs {
            ifaddrs {
                ifa_next: next,
                ifa_name: self.ifa_name.as_c_str().as_ptr().cast_mut(),
                #[allow(clippy::cast_sign_loss)]
                ifa_flags: self.ifa_flags as u32,
                ifa_addr: Box::into_raw(Box::new(self.ifa_addr.as_sockaddr())),
                ifa_netmask: ptr::null_mut::<sockaddr>(),
                ifa_ifu: ptr::null_mut::<sockaddr>(),
                ifa_data: ptr::null_mut::<libc::c_void>(),
                //ifa_data: self
                //    .ifa_data
                //    .clone()
                //    .map_or(ptr::null_mut::<libc::c_void>(), |v| {
                //        v.as_ptr().cast::<libc::c_void>().cast_mut()
                //    }),
            }
        }
    }

    impl<'a> Drop for MockIfaddrsIterator<'a> {
        fn drop(&mut self) {
            for sockaddr in &self.raw_scokaddrs {
                if !sockaddr.is_null() {
                    let sockaddr: Box<sockaddr> =
                        unsafe { Box::from_raw(*sockaddr) };
                    drop(sockaddr);
                }
            }
        }
    }
}
