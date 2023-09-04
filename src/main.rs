use std::{
    cmp::min,
    io,
    process::ExitCode,
    thread::sleep,
    time::{Duration, Instant},
};
use wait_online::{
    arguments::Args, ifaddrs::getifaddrs, network_online,
    NewtorkOnlineArguments,
};

fn main() -> Result<ExitCode, io::Error> {
    let start = Instant::now();

    let args = Args::get();
    let stop = start + Duration::from_secs(args.timeout);

    let network_online_arguments = NewtorkOnlineArguments::from(&args);

    if args.interval == 0 {
        while !network_online(network_online_arguments, getifaddrs()?) {
            sleep(Duration::from_millis(args.interval));
        }
    } else {
        while !network_online(network_online_arguments, getifaddrs()?) {
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
