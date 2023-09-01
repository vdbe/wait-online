use std::{io, thread::sleep, time::Duration};

use wait_online::all_interfaces_up;

fn main() -> Result<(), io::Error> {
    while !all_interfaces_up()? {
        sleep(Duration::from_millis(500));
    }

    Ok(())
}
