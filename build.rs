use std::{env, fs::File, io::Write, path::Path};

fn service(exec: &str) -> String {
    let description = "Wait for Network to be Configured";

    let network_target = "network.target";
    let network_online_target = "network-online.target";
    let shutdown_target = "shutdown.target";

    format!(
        r#"[Unit]
Description={description}
DefaultDependencies=no
Conflicts={shutdown_target}
BindsTo={network_target}
After={network_target}
Before={network_online_target} {shutdown_target}

[Service]
Type=oneshot
ExecStart={exec}
RemainAfterExit=yes

[Install]
WantedBy={network_online_target}
    "#
    )
}

fn main() {
    let prefix = env::var("prefix");
    let prefix = prefix.as_deref().unwrap_or("/usr/local");

    let service_path =
        Path::new("target/network-standalone-wait-online.service");
    let exec = format!("{prefix}/bin/wait-online");

    let service = service(&exec);

    File::create(service_path)
        .expect("failed to create service file")
        .write_all(service.as_bytes())
        .expect("failed to write service file");
}
