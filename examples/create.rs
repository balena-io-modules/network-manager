extern crate network_manager;

use std::env;
use std::process;

use network_manager::{NetworkManager, DeviceType};


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("USAGE: create SSID PASSWORD");
        process::exit(1);
    }

    let manager = NetworkManager::new();

    let devices = manager.get_devices().unwrap();
    let device_index = devices
        .iter()
        .position(|ref d| *d.device_type() == DeviceType::WiFi)
        .unwrap();
    let wifi_device = devices[device_index].as_wifi_device().unwrap();

    let access_points = wifi_device.get_access_points().unwrap();

    let ap_index = access_points
        .iter()
        .position(|ref ap| ap.ssid() == args[1])
        .unwrap();

    wifi_device
        .connect(&access_points[ap_index], &args[2])
        .unwrap();
}
