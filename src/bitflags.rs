libc_bitflags!(
    /// Standard interface flags, used by
    /// [`libc::ifaddrs::ifa_name`]
    pub enum InterfaceFlags: libc::c_int {
        /// Interface is running. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_UP;
        /// Valid broadcast address set. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_BROADCAST;
        /// Internal debugging flag. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_DEBUG;
        /// Interface is a loopback interface. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_LOOPBACK;
        /// Interface is a point-to-point link. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_POINTOPOINT;
        /// Avoid use of trailers. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_NOTRAILERS;
        /// Resources allocated. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_RUNNING;
        /// No arp protocol, L2 destination address not set. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_NOARP;
        /// Interface is in promiscuous mode. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_PROMISC;
        /// Receive all multicast packets. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_ALLMULTI;
        /// Master of a load balancing bundle. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_MASTER;
        /// Slave of a load balancing bundle. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_SLAVE;
        /// Supports multicast. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_MULTICAST;
        /// Is able to select media type via ifmap. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_PORTSEL;
        /// Auto media selection active. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_AUTOMEDIA;
        /// The addresses are lost when the interface goes down. Sysfs. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_DYNAMIC;
        /// Driver signals L1 up. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_LOWER_UP;
        /// Driver signals dormant. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_DORMANT;
        /// Echo sent packets. Volatile. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_ECHO;
    }
);
