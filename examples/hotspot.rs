extern crate network_manager;

use std::env;
use std::process;

use network_manager::{NetworkManager, Device, DeviceType};


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


fn find_device(manager: &NetworkManager, interface: Option<String>) -> Option<Device> {
    if let Some(interface) = interface {
        let device = manager.get_device_by_interface(&interface).unwrap();

        if *device.device_type() == DeviceType::WiFi {
            Some(device)
        } else {
            None
        }
    } else {
        let devices = manager.get_devices().unwrap();

        let index = devices
            .iter()
            .position(|ref d| *d.device_type() == DeviceType::WiFi);

        if let Some(index) = index {
            Some(devices[index].clone())
        } else {
            None
        }
    }
}


fn main() {
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

    let device = find_device(&manager, interface).unwrap();
    let wifi_device = device.as_wifi_device().unwrap();

    wifi_device.create_hotspot(&ssid as &str, pass_str).unwrap();
}
