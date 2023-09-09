#[cfg(feature = "clap")]
use clap::{value_parser, Parser};

#[derive(Debug)]
#[cfg_attr(feature = "clap", derive(Parser), command(author, version, about))]
pub struct Args {
    /// Block until at least these interfaces have appeared
    #[cfg_attr(feature = "clap", arg(short, long, conflicts_with = "ignore"))]
    pub interface: Option<Vec<String>>,

    /// Don't take these interfaces into account
    ///
    /// By default only loopback interfaces are ignored.
    #[cfg_attr(feature = "clap", arg(long, conflicts_with = "interface"))]
    pub ignore: Option<Vec<String>>,

    /// Requires at least one IPv4 address
    #[cfg_attr(
        feature = "clap",
        arg(short = '4', long, default_value_t = false)
    )]
    pub ipv4: bool,

    /// Requires at least one IPv6 address
    #[cfg_attr(
        feature = "clap",
        arg(short = '6', long, default_value_t = false)
    )]
    pub ipv6: bool,

    /// Wait until at least one of the interfaces is online
    ///
    /// If this options is specified with `--interface`, then wait until at
    /// least one specified interfaces becomes online.
    #[cfg_attr(feature = "clap", arg(long, default_value_t = false))]
    pub any: bool,

    /// Maximum time to wait for network connectivity in seconds
    ///
    /// Fail the service if the network is not online by the time the timeout
    /// elapses.
    /// A timeout of 0 disables the timeout.
    #[cfg_attr(feature = "clap", arg( long, default_value_t = Self::DEFAULT_TIMOUT))]
    pub timeout: u64,

    /// Time between checks in ms
    ///
    /// Must be between inclusive 10 and 10_000.
    #[cfg_attr(feature = "clap", arg(
        long, default_value_t = Self::DEFAULT_INTERVAL,
        value_parser = value_parser!(u64).range(Self::MIN_INTERVAL..=Self::MAX_INTERVAL)
    ))]
    pub interval: u64,
}

impl Args {
    pub const DEFAULT_INTERVAL: u64 = 500;
    #[cfg(feature = "clap")]
    const MIN_INTERVAL: u64 = 10;
    #[cfg(feature = "clap")]
    const MAX_INTERVAL: u64 = 10_000;

    const DEFAULT_TIMOUT: u64 = 120;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            interface: None,
            ignore: None,
            timeout: Self::DEFAULT_TIMOUT,
            interval: Self::DEFAULT_INTERVAL,
            ipv4: false,
            ipv6: false,
            any: false,
        }
    }

    #[must_use]
    pub fn interface(mut self, interface: Vec<String>) -> Self {
        self.ignore = None;
        self.interface = Some(interface);
        self
    }

    #[must_use]
    pub fn ignore(mut self, ignore: Vec<String>) -> Self {
        self.interface = None;
        self.ignore = Some(ignore);
        self
    }

    #[must_use]
    pub const fn interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    #[must_use]
    pub const fn ipv4(mut self, ipv4: bool) -> Self {
        self.ipv4 = ipv4;
        self
    }

    #[must_use]
    pub const fn ipv6(mut self, ipv6: bool) -> Self {
        self.ipv6 = ipv6;
        self
    }

    #[must_use]
    pub const fn any(mut self, any: bool) -> Self {
        self.any = any;
        self
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}
