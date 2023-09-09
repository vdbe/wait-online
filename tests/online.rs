use std::{ffi::CString, ptr, slice};

use nix::{net::if_::InterfaceFlags, sys::socket::AddressFamily};

use wait_online::{
    arguments::Args,
    libc::{self, c_char, ifaddrs, sa_family_t, sockaddr},
    network_online, NetworkArgument,
};

const FLAGS_LOOPBACK: i32 = InterfaceFlags::IFF_UP.bits()
    | InterfaceFlags::IFF_LOOPBACK.bits()
    | InterfaceFlags::IFF_RUNNING.bits()
    | InterfaceFlags::IFF_LOWER_UP.bits();

const FLAGS_LOWER_LAYWER_DOWN: i32 = InterfaceFlags::IFF_UP.bits()
    | InterfaceFlags::IFF_BROADCAST.bits()
    | InterfaceFlags::IFF_MULTICAST.bits();

const FLAGS_UP: i32 = InterfaceFlags::IFF_UP.bits()
    | InterfaceFlags::IFF_BROADCAST.bits()
    | InterfaceFlags::IFF_RUNNING.bits()
    | InterfaceFlags::IFF_MULTICAST.bits()
    | InterfaceFlags::IFF_LOWER_UP.bits();

mod online {
    use super::*;

    #[test]
    fn basic() {
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
    fn only_loopback() {
        let n_args = NetworkArgument::default();

        let mut v = vec![MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK)];
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        let args = Args::new().ipv4(true);
        assert!(!network_online(
            MockIfaddrsIterator::new(&v),
            (&args).into()
        ));

        v.push(
            MockIfaddrs::new()
                .name("eth0")
                .flags(FLAGS_UP)
                .sockaddr(AddressFamily::Inet),
        );
        assert!(network_online(MockIfaddrsIterator::new(&v), (&args).into()));
    }

    #[test]
    fn implicit_ignore_loopback() {
        let mut args = Args::new();
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
    fn require_loopback() {
        let mut args = Args::new();
        args.ipv4 = true;
        args.interface = Some(vec!["lo".into()]);
        let n_args = NetworkArgument::from(&args);

        let mock_ifaddr = MockIfaddrs::new()
            .name("lo")
            .flags(FLAGS_LOOPBACK)
            .sockaddr(AddressFamily::Packet);

        let mut v = vec![mock_ifaddr.sockaddr(AddressFamily::Packet)];
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));

        v.push(
            MockIfaddrs::new()
                .name("lo")
                .flags(FLAGS_LOOPBACK)
                .sockaddr(AddressFamily::Inet),
        );
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn family_type_v4() {
        let mut args = Args::new();
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
    fn family_type_v6() {
        let mut args = Args::new();
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
    fn require_single() {
        let mut args = Args::new();
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
    fn require_missing() {
        let mut args = Args::new();
        args.interface = Some(vec!["eth999999".into()]);
        let n_args = NetworkArgument::from(&args);

        let v = vec![
            MockIfaddrs::new().name("lo").flags(FLAGS_LOOPBACK),
            MockIfaddrs::new().name("eth0").flags(FLAGS_UP),
        ];
        assert!(!network_online(MockIfaddrsIterator::new(&v), n_args));
    }

    #[test]
    fn require_multi() {
        let mut args = Args::new();
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
    fn ignore_single() {
        let mut args = Args::new();
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
    fn ignore_multi() {
        let mut args = Args::new();
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
    fn any() {
        let mut args = Args::new();
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
            let ifaddrs =
                mock_ifaddrs.make_ifaddrs(ptr::null::<ifaddrs>().cast_mut());
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
        self.ifa_name = CString::new(name.to_string()).expect("");
        self
    }

    const fn flags(mut self, flags: i32) -> Self {
        self.ifa_flags = flags;
        self
    }

    fn sockaddr(mut self, family: AddressFamily) -> Self {
        self.ifa_addr = MockSockaddr {
            sa_family: libc::sa_family_t::try_from(family as i32).expect(""),
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
