use std::{
    cmp::min,
    io,
    process::ExitCode,
    thread::sleep,
    time::{Duration, Instant},
};

use clap::Parser;

use wait_online::{
    arguments::Args, ifaddrs::getifaddrs, network_online, NetworkArgument,
};

fn main() -> Result<ExitCode, io::Error> {
    let start = Instant::now();

    let args = Args::parse();
    let stop = start + Duration::from_secs(args.timeout);

    let network_argument = NetworkArgument::from(&args);

    if args.interval == 0 {
        while !network_online(getifaddrs()?, network_argument) {
            sleep(Duration::from_millis(args.interval));
        }
    } else {
        while !network_online(getifaddrs()?, network_argument) {
            // Check if for timeout
            // If not sleep for interval or untill timeout whichever is faster
            let time_to_timeout = stop.checked_duration_since(Instant::now());
            if let Some(time_to_timeout) = time_to_timeout {
                sleep(min(
                    Duration::from_millis(args.interval),
                    time_to_timeout,
                ));
            } else {
                // Timeout
                return Ok(ExitCode::FAILURE);
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}
