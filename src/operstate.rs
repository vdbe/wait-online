/// Systemd opersational states
///
/// (See
/// [`netowrkctl(1)`](https://man7.org/linux/man-pages/man1/networkctl.1.html)
/// for more info)
pub enum OperState {
    /// the device is missing
    Missing,

    /// the device is powered down
    Off,

    /// the device is powered up, but it does not yet have a carrier
    NoCarrier,

    /// the device has a carrier, but is not yet ready for normal traffic
    Dormant,

    /// one of the bonding or bridge slave network interfaces is in off,
    /// no-carrier, or dormant state, and the master interface has no address.
    DegradedCarrier,

    /// the link has a carrier, or for bond or bridge master, all bonding or
    /// bridge slave network interfaces are enslaved to the master
    Carrier,

    /// the link has carrier and addresses valid on the local link configured.
    /// For bond or bridge master this means that not all slave network
    /// interfaces have carrier but at least one does.
    Degraded,

    /// the link has carrier and is enslaved to bond or bridge master network
    /// interface
    Enslaved,

    /// the link has carrier and routable address configured. For bond or
    /// bridge master it is not necessary for all slave network interfaces to
    /// have carrier, but at least one must.
    Routable,
}
