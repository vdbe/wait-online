use clap::{value_parser, Parser};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Interface(s) to wait for
    #[arg(short, long, conflicts_with = "ignore")]
    pub interface: Option<Vec<String>>,

    #[arg(long, conflicts_with = "interface")]
    pub ignore: Option<Vec<String>>,

    /// Max time before failing in seconds
    #[arg( long, default_value_t = Self::DEFAULT_TIMOUT)]
    pub timeout: u64,

    /// Time between checks in ms
    #[arg(
        long, default_value_t = Self::DEFAULT_INTERVAL,
        value_parser = value_parser!(u64).range(Self::MIN_INTERVAL..=Self::MAX_INTERVAL)
    )]
    pub interval: u64,
}
impl Args {
    pub const DEFAULT_INTERVAL: u64 = 500;
    const MIN_INTERVAL: u64 = 10;
    const MAX_INTERVAL: u64 = 10_000;

    const DEFAULT_TIMOUT: u64 = 120;

    #[must_use]
    #[inline]
    pub fn get() -> Self {
        Self::parse()
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            interface: None,
            ignore: None,
            timeout: Self::DEFAULT_TIMOUT,
            interval: Self::DEFAULT_INTERVAL,
        }
    }
}
