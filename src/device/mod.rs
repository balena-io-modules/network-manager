use general::{Interface, DeviceState};

/// Get a list of Network Manager devices.
///
/// # Examples
///
/// ```
/// let devices = network_manager::device::list().unwrap();
/// println!("{:?}", devices);
/// ```
pub fn list() -> Result<Vec<Device>, String> {
    // Get list of devices

    let device1 = Device {
        name: "resin_io".to_owned(),
        device: "wlp4s0".to_owned(),
        path: "/org/freedesktop/NetworkManager/ActiveDevice/187".to_owned(),
        interface: Interface::WiFi,
        state: DeviceState::Activated,
    };

    let device2 = Device {
        name: "docker0".to_owned(),
        device: "docker0".to_owned(),
        path: "/org/freedesktop/NetworkManager/ActiveDevice/180".to_owned(),
        interface: Interface::Bridge,
        state: DeviceState::Activated,
    };

    Ok(vec![device1, device2])
}

/// Enables a Network Manager device.
///
/// # Examples
///
/// ```
/// let devices = network_manager::device::list().unwrap();
/// let device = &devices[0];
/// let state = network_manager::device::enable(device, 10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(c: &Device, t: i32) -> Result<DeviceState, String> {
    // Enable device

    if t != 0 {
        // Wait until the device state is 'Activated' or
        // until the time has elapsed
    }

    Ok(DeviceState::Activated)
}

/// Disables a Network Manager device.
///
/// # Examples
///
/// ```
/// let devices = network_manager::device::list().unwrap();
/// let device = &devices[0];
/// let state = network_manager::device::disable(device, 10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(c: &Device, t: i32) -> Result<DeviceState, String> {
    // Disable device

    if t != 0 {
        // Wait until the device state is 'Unavailable' or
        // until the time has elapsed
    }

    Ok(DeviceState::Unavailable)
}

#[derive(Debug)]
pub struct Device {
    name: String,
    device: String,
    path: String,
    interface: Interface,
    state: DeviceState,
}
