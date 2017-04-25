extern crate network_manager;

use std::env;
use std::process;

use network_manager::manager;
use network_manager::device;
use network_manager::connection;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("USAGE: hotspot SSID PASSWORD");
        process::exit(1);
    }

    let manager = manager::new();

    let mut devices = device::list(&manager).unwrap();
    let device_index = devices
        .iter()
        .position(|ref d| d.device_type == device::DeviceType::WiFi)
        .unwrap();
    let device_ref = &mut devices[device_index];

    connection::create_hotspot(&manager, device_ref, &args[1], &args[2], 10).unwrap();
}
