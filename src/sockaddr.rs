// Re-export in case we need a wrapper later
pub use crate::libc::sockaddr;
pub use nix::sys::socket::AddressFamily;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterfacesFamilyTypeArgument {
    pub(crate) ipv4: bool,
    pub(crate) ipv6: bool,
}

impl InterfacesFamilyTypeArgument {
    #[must_use]
    pub(crate) const fn from_args(ipv4: bool, ipv6: bool) -> Option<Self> {
        match (ipv4, ipv6) {
            (false, false) => None,
            (ipv4, ipv6) => Some(Self { ipv4, ipv6 }),
        }
    }
}

/// # Safety
///
/// A valid `libc::sockaddr` ptr must be provided,
/// this ptr can be null
#[inline]
pub(crate) unsafe fn get_addres_family(
    ifa_addr: *mut sockaddr,
) -> Option<AddressFamily> {
    (!ifa_addr.is_null())
        .then(|| AddressFamily::from_i32(i32::from((*ifa_addr).sa_family)))
        .flatten()
}

/// Checks if waiting is required for given `ifa_addr`
///
/// # Safety
///
/// A valid `libc::sockaddr` ptr must be provided,
/// this ptr can be null
pub unsafe fn check_family_type(
    ifa_addr: *mut sockaddr,
    family_argument: InterfacesFamilyTypeArgument,
) -> bool {
    // Check if this type of address if required
    let family = unsafe { get_addres_family(ifa_addr) };
    match family {
        Some(AddressFamily::Inet) => family_argument.ipv4,
        Some(AddressFamily::Inet6) => family_argument.ipv6,
        _ => true,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use crate::libc;

    #[test]
    fn family_type_arg() {
        assert!(InterfacesFamilyTypeArgument::from_args(false, false).is_none());
        assert_eq!(
            InterfacesFamilyTypeArgument::from_args(true, false),
            Some(InterfacesFamilyTypeArgument {
                ipv4: true,
                ipv6: false
            })
        );
        assert_eq!(
            InterfacesFamilyTypeArgument::from_args(false, true),
            Some(InterfacesFamilyTypeArgument {
                ipv4: false,
                ipv6: true
            })
        );
        assert_eq!(
            InterfacesFamilyTypeArgument::from_args(true, true),
            Some(InterfacesFamilyTypeArgument {
                ipv4: true,
                ipv6: true
            })
        );
    }
    #[test]
    fn check_family_type_ipv4() {
        let combinations = vec![
            (
                AddressFamily::Inet,
                InterfacesFamilyTypeArgument::from_args(true, true),
                true,
            ),
            (
                AddressFamily::Inet,
                InterfacesFamilyTypeArgument::from_args(false, true),
                false,
            ),
            (
                AddressFamily::Inet,
                InterfacesFamilyTypeArgument::from_args(true, false),
                true,
            ),
        ];

        check_combinations(&combinations);
    }

    #[test]
    fn check_family_type_ipv6() {
        let combinations = vec![
            (
                AddressFamily::Inet6,
                InterfacesFamilyTypeArgument::from_args(true, true),
                true,
            ),
            (
                AddressFamily::Inet6,
                InterfacesFamilyTypeArgument::from_args(false, true),
                true,
            ),
            (
                AddressFamily::Inet6,
                InterfacesFamilyTypeArgument::from_args(true, false),
                false,
            ),
        ];

        check_combinations(&combinations);
    }

    #[test]
    fn check_family_type_other() {
        let combinations = vec![
            (
                AddressFamily::Unix,
                InterfacesFamilyTypeArgument::from_args(true, true),
                true,
            ),
            (
                AddressFamily::Unix,
                InterfacesFamilyTypeArgument::from_args(false, true),
                true,
            ),
            (
                AddressFamily::Unix,
                InterfacesFamilyTypeArgument::from_args(true, false),
                true,
            ),
        ];

        check_combinations(&combinations);
    }

    fn check_combinations(
        combinations: &[(
            AddressFamily,
            Option<InterfacesFamilyTypeArgument>,
            bool,
        )],
    ) {
        for combination in combinations {
            let mut sockaddr = create_sockaddr(combination.0);
            let sockaddr_ptr: *mut sockaddr = &mut sockaddr;
            let ret = unsafe {
                check_family_type(sockaddr_ptr, combination.1.unwrap())
            };
            assert_eq!(ret, combination.2);
        }
    }

    fn create_sockaddr(family: AddressFamily) -> sockaddr {
        sockaddr {
            sa_family: libc::sa_family_t::try_from(family as i32).unwrap(),
            sa_data: [libc::c_char::default(); 14],
        }
    }
}
