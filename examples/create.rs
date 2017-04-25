extern crate network_manager;

use std::env;
use std::process;

use network_manager::manager;
use network_manager::wifi;
use network_manager::device;
use network_manager::connection;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("USAGE: create SSID PASSWORD");
        process::exit(1);
    }

    let manager = manager::new();

    let mut devices = device::list(&manager).unwrap();
    let device_index = devices
        .iter()
        .position(|ref d| d.device_type == device::DeviceType::WiFi)
        .unwrap();
    let device_ref = &mut devices[device_index];

    let access_points = wifi::scan(&manager, device_ref).unwrap();

    let ap_index = access_points
        .iter()
        .position(|ref ap| ap.ssid == args[1])
        .unwrap();

    connection::create(&manager, device_ref, &access_points[ap_index], &args[2], 10).unwrap();
}
