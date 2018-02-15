#[macro_use]
extern crate error_chain;

extern crate network_manager;

use std::env;
use std::process;
use std::io::Write;

use network_manager::{Device, DeviceType, NetworkManager};

mod errors {
    use network_manager;

    error_chain! {
        links {
            NetworkManager(network_manager::errors::Error, network_manager::errors::ErrorKind);
        }

        errors {
            Runtime(info: String) {
                description("Runtime error")
                display("{}", info)
            }
        }
    }
}

use errors::*;

struct Options {
    interface: Option<String>,
    ssid: String,
    password: Option<String>,
}

fn main() {
    if let Err(ref e) = run() {
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "  caused by: {}", e).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let Options {
        interface,
        ssid,
        password,
    } = parse_options();

    let pass_str = match password {
        Some(ref s) => Some(s as &str),
        None => None,
    };

    let manager = NetworkManager::new();

    let device = find_device(&manager, interface)?;
    let wifi_device = device.as_wifi_device().unwrap();

    wifi_device.create_hotspot(&ssid as &str, pass_str, None)?;

    Ok(())
}

fn parse_options() -> Options {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage_and_exit();
    }

    let (ssid_pos, interface) = if args[1] == "-i" {
        if args.len() < 4 {
            print_usage_and_exit();
        }

        (3, Some(args[2].clone()))
    } else {
        (1, None)
    };

    let ssid = args[ssid_pos].clone();

    let password = if args.len() < ssid_pos + 2 {
        None
    } else {
        Some(args[ssid_pos + 1].clone())
    };

    Options {
        interface: interface,
        ssid: ssid,
        password: password,
    }
}

fn print_usage_and_exit() {
    println!("USAGE: hotspot [-i INTERFACE] SSID [PASSWORD]");
    process::exit(1);
}

fn find_device(manager: &NetworkManager, interface: Option<String>) -> Result<Device> {
    if let Some(interface) = interface {
        let device = manager.get_device_by_interface(&interface)?;

        if *device.device_type() == DeviceType::WiFi {
            Ok(device)
        } else {
            bail!(ErrorKind::Runtime(format!(
                "{} is not a WiFi device",
                interface
            )))
        }
    } else {
        let devices = manager.get_devices()?;

        let index = devices
            .iter()
            .position(|d| *d.device_type() == DeviceType::WiFi);

        if let Some(index) = index {
            Ok(devices[index].clone())
        } else {
            bail!(ErrorKind::Runtime("Cannot find a WiFi device".into()))
        }
    }
}
