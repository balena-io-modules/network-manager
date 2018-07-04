#[macro_use]
extern crate error_chain;

extern crate clap;
extern crate network_manager;

use clap::{App, Arg};
use std::io::Write;

use network_manager::{AccessPoint, AccessPointCredentials, Device, DeviceType, NetworkManager};

// Network manager version set at compile time
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    let matches = App::new(file!())
        .version(CARGO_PKG_VERSION)
        .arg(
            Arg::with_name("SSID")
                .takes_value(true)
                .required(true)
                .help("Network SSID"),
        )
        .arg(
            Arg::with_name("PASSWORD")
                .takes_value(true)
                .required(true)
                .help("Network password"),
        )
        .get_matches();

    let manager = NetworkManager::new();

    let device = find_device(&manager)?;

    let wifi_device = device.as_wifi_device().unwrap();

    let access_points = wifi_device.get_access_points()?;

    let ap_index = find_access_point(&access_points, matches.value_of("SSID").unwrap())?;

    let credentials = AccessPointCredentials::Wpa {
        passphrase: matches.value_of("PASSWORD").unwrap().to_string(),
    };

    wifi_device.connect(&access_points[ap_index], &credentials)?;

    Ok(())
}

fn find_device(manager: &NetworkManager) -> Result<Device> {
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

fn find_access_point(access_points: &[AccessPoint], ssid: &str) -> Result<usize> {
    if let Some(index) = access_points.iter().position(|ap| same_ssid(ap, ssid)) {
        Ok(index)
    } else {
        bail!(ErrorKind::Runtime("Access point not found".into()))
    }
}

fn same_ssid(ap: &AccessPoint, ssid: &str) -> bool {
    if let Ok(ap_ssid) = ap.ssid().as_str() {
        ap_ssid == ssid
    } else {
        false
    }
}
