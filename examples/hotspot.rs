extern crate network_manager;

use std::env;
use std::process;

use network_manager::manager;
use network_manager::device;
use network_manager::connection;


struct Options {
    interface: Option<String>,
    ssid: String,
    password: Option<String>,
}


fn print_usage_and_exit() {
    println!("USAGE: hotspot [-i INTERFACE] SSID [PASSWORD]");
    process::exit(1);
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


fn find_device(devices: &Vec<device::Device>, interface: Option<String>) -> Option<usize> {
    if let Some(interface) = interface {
        devices
            .iter()
            .position(|ref d| d.device_type == device::DeviceType::WiFi && d.interface == interface)
    } else {
        devices
            .iter()
            .position(|ref d| d.device_type == device::DeviceType::WiFi)
    }
}


fn main() {
    let Options {
        interface,
        ssid,
        password,
    } = parse_options();

    let manager = manager::new();

    let mut devices = device::list(&manager).unwrap();

    let device_index = find_device(&devices, interface).unwrap();
    let device_ref = &mut devices[device_index];

    connection::create_hotspot(&manager, device_ref, &ssid, password, 10).unwrap();
}
