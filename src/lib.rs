#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported");

use arguments::Args;
use ifaddrs::is_interface_up;
use std::ffi;

mod errno;
#[macro_use]
mod macros;
mod bitflags;

pub mod arguments;
pub mod ifaddrs;

#[derive(Debug, Clone, Copy, PartialEq)]
enum NetworkOnlineArgumentsInterfacesAction {
    Ignore,
    Require,
}

#[derive(Debug, Clone, Copy)]
struct NetworkOnlineArgumentsInterfaces<'a> {
    interfaces: &'a [String],
    action: NetworkOnlineArgumentsInterfacesAction,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NewtorkOnlineArguments<'a> {
    interfaces: Option<NetworkOnlineArgumentsInterfaces<'a>>,
}

/// Checks if network if online given the requirements provided by
/// `network_online_arguments`
pub fn network_online<I>(
    network_online_arguments: NewtorkOnlineArguments,
    mut ifaddrs: I,
) -> bool
where
    I: Iterator<Item = libc::ifaddrs>,
{
    if let Some(NetworkOnlineArgumentsInterfaces { interfaces, action }) =
        network_online_arguments.interfaces
    {
        ifaddrs.all(|ifaddr| {
            // up   => true
            // down => _not_ in interfaces => require => true
            //                             => ignore  => false
            //         _in_ interfaces     => require => false
            //                             => ignore  => true
            //
            is_interface_up(ifaddr) || {
                let ifa_name = unsafe { ffi::CStr::from_ptr(ifaddr.ifa_name) };
                let ifa_name = ifa_name.to_bytes();
                let in_interface = interfaces
                    .iter()
                    .any(|interface| interface.as_bytes() == ifa_name);
                (action == NetworkOnlineArgumentsInterfacesAction::Require)
                    ^ in_interface
            }
        })
    } else {
        ifaddrs.all(is_interface_up)
    }
}

impl<'a> NetworkOnlineArgumentsInterfaces<'a> {
    const fn new(
        interfaces: &'a [String],
        action: NetworkOnlineArgumentsInterfacesAction,
    ) -> Self {
        Self { interfaces, action }
    }
}

impl<'a> From<&'a Args> for NewtorkOnlineArguments<'a> {
    fn from(args: &'a Args) -> Self {
        type Noia<'a> = NetworkOnlineArgumentsInterfaces<'a>;
        use NetworkOnlineArgumentsInterfacesAction::{Ignore, Require};
        let interfaces: Option<NetworkOnlineArgumentsInterfaces<'a>> =
            match (&args.interface, &args.ignore) {
                (None, None) => None,
                (Some(interfaces), None) => {
                    Some(Noia::new(interfaces, Require))
                }
                (None, Some(interfaces)) => Some(Noia::new(interfaces, Ignore)),
                _ => unreachable!(
                "`interfaces` and `ignore` can never be set at the same time"
            ),
            };
        Self { interfaces }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::slice;
    use std::{ffi::CString, ptr};

    use libc::{c_char, c_void, ifaddrs, sa_family_t, sockaddr};

    use crate::ifaddrs::InterfaceFlags as F;

    const FLAGS_LOOPBACK: i32 =
        F::IFF_UP | F::IFF_LOOPBACK | F::IFF_RUNNING | F::IFF_LOWER_UP;

    const FLAGS_LOWER_LAYWER_DOWN: i32 =
        F::IFF_UP | F::IFF_BROADCAST | F::IFF_MULTICAST;

    const FLAGS_UP: i32 = F::IFF_UP
        | F::IFF_BROADCAST
        | F::IFF_RUNNING
        | F::IFF_MULTICAST
        | F::IFF_LOWER_UP;

    #[test]
    fn online() {
        let online_args = NewtorkOnlineArguments::default();

        let mut v = vec![
            MockIfaddrs::new().flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().flags(FLAGS_UP),
        ];
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));

        v.push(MockIfaddrs::new().flags(FLAGS_LOWER_LAYWER_DOWN));
        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));
    }

    #[test]
    fn online_require_single() {
        let interfaces = ["eth1".to_string()];
        let online_args = NewtorkOnlineArguments {
            interfaces: Some(NetworkOnlineArgumentsInterfaces {
                interfaces: &interfaces,
                action: NetworkOnlineArgumentsInterfacesAction::Require,
            }),
        };

        let mut v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
        ];

        let i = MockIfaddrsIterator::new(&v);
        assert!(!network_online(online_args, i));

        v.push(MockIfaddrs::new().name("eth1").flags(FLAGS_UP));
        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));

        v.remove(2);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));
    }
    #[test]
    fn online_require_multi() {
        let interfaces = ["eth1".to_string(), "eth2".to_string()];
        let online_args = NewtorkOnlineArguments {
            interfaces: Some(NetworkOnlineArgumentsInterfaces {
                interfaces: &interfaces,
                action: NetworkOnlineArgumentsInterfacesAction::Require,
            }),
        };

        let mut v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
        ];

        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[1] = MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_LOWER_LAYWER_DOWN);
        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[2] = MockIfaddrs::new().name("eth1").flags(FLAGS_UP);
        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[2] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_LOWER_LAYWER_DOWN);
        v[3] = MockIfaddrs::new().name("eth1").flags(FLAGS_UP);
        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[2] = MockIfaddrs::new().name("eth1").flags(FLAGS_UP);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[1] = MockIfaddrs::new()
            .name("eth0")
            .flags(FLAGS_LOWER_LAYWER_DOWN);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));
    }

    #[test]
    fn online_ignore_single() {
        let interfaces = ["eth0".to_string()];
        let online_args = NewtorkOnlineArguments {
            interfaces: Some(NetworkOnlineArgumentsInterfaces {
                interfaces: &interfaces,
                action: NetworkOnlineArgumentsInterfacesAction::Ignore,
            }),
        };

        let v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
            MockIfaddrs::new().name("eth1").flags(FLAGS_UP),
        ];

        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));

        let v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
        ];

        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));
    }

    #[test]
    fn online_ignore_multi() {
        let interfaces = ["eth1".to_string(), "eth2".to_string()];
        let online_args = NewtorkOnlineArguments {
            interfaces: Some(NetworkOnlineArgumentsInterfaces {
                interfaces: &interfaces,
                action: NetworkOnlineArgumentsInterfacesAction::Ignore,
            }),
        };

        let mut v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
            MockIfaddrs::new()
                .name("eth1")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
            MockIfaddrs::new()
                .name("eth2")
                .flags(FLAGS_LOWER_LAYWER_DOWN),
        ];

        assert!(!network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[1] = MockIfaddrs::new().name("eth0").flags(FLAGS_UP);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[2] = MockIfaddrs::new().name("eth1").flags(FLAGS_UP);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[2] = MockIfaddrs::new()
            .name("eth1")
            .flags(FLAGS_LOWER_LAYWER_DOWN);
        v[3] = MockIfaddrs::new().name("eth2").flags(FLAGS_UP);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));

        v[2] = MockIfaddrs::new().name("eth1").flags(FLAGS_UP);
        assert!(network_online(online_args, MockIfaddrsIterator::new(&v)));
    }

    #[derive(Default, Clone, Debug)]
    struct MockSockaddr {
        sa_family: sa_family_t,
        sa_data: [c_char; 14],
    }

    #[derive(Default, Clone, Debug)]
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
    }

    impl<'a> Iterator for MockIfaddrsIterator<'a> {
        type Item = ifaddrs;

        fn next(&mut self) -> Option<Self::Item> {
            self.iterator.next().map(|mock_ifaddrs| {
                mock_ifaddrs
                    .make_ifaddrs(ptr::null::<libc::ifaddrs>().cast_mut())
            })
        }
    }

    impl<'a> MockIfaddrsIterator<'a> {
        fn new(v: &'a [MockIfaddrs]) -> Self {
            Self { iterator: v.iter() }
        }
    }

    impl MockSockaddr {
        const fn into_sockaddr(self) -> sockaddr {
            sockaddr {
                sa_family: self.sa_family,
                sa_data: self.sa_data,
            }
        }
    }

    impl MockIfaddrs {
        fn new() -> Self {
            MockIfaddrs::default()
        }

        fn name<S: ToString>(mut self, name: S) -> Self {
            self.ifa_name = CString::new(name.to_string()).unwrap();
            self
        }

        fn flags(mut self, flags: i32) -> Self {
            self.ifa_flags = flags;
            self
        }

        fn make_ifaddrs(&self, next: *mut ifaddrs) -> libc::ifaddrs {
            ifaddrs {
                ifa_next: next,
                ifa_name: self.ifa_name.as_c_str().as_ptr().cast_mut(),
                #[allow(clippy::cast_sign_loss)]
                ifa_flags: self.ifa_flags as u32,
                ifa_addr: &mut self.ifa_addr.clone().into_sockaddr(),
                ifa_netmask: &mut self.ifa_netmask.clone().into_sockaddr(),
                ifa_ifu: &mut self.ifa_ifu.clone().into_sockaddr(),
                ifa_data: self
                    .ifa_data
                    .clone()
                    .map_or(ptr::null::<c_void>().cast_mut(), |v| {
                        v.as_ptr().cast::<c_void>().cast_mut()
                    }),
            }
        }
    }
}
