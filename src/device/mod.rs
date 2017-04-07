use manager;
use manager::NetworkManager;


#[derive(Debug)]
pub struct Device {
    interface: String,
    path: String,
    device_type: DeviceType,
    state: DeviceState,
    real: bool,
}


#[derive(Debug)]
pub enum DeviceType {
    Unknown,
    Generic,
    Ethernet,
    WiFi,
    Bridge,
}

impl From<i64> for DeviceType {
    fn from(state: i64) -> Self {
        match state {
            0 => DeviceType::Unknown,
            14 => DeviceType::Generic,
            1 => DeviceType::Ethernet,
            2 => DeviceType::WiFi,
            13 => DeviceType::Bridge,
            _ => DeviceType::Unknown,

        }
    }
}


#[derive(Debug)]
pub enum DeviceState {
    Unknown,
    Unmanaged,
    Unavailable,
    Disconnected,
    Activated,
    Deactivating,
    Failed,
}

impl From<i64> for DeviceState {
    fn from(state: i64) -> Self {
        match state {
            0 => DeviceState::Unknown,
            10 => DeviceState::Unmanaged,
            20 => DeviceState::Unavailable,
            30 => DeviceState::Disconnected,
            100 => DeviceState::Activated,
            110 => DeviceState::Deactivating,
            120 => DeviceState::Failed,
            _ => DeviceState::Unknown,

        }
    }
}


/// Get a list of Network Manager devices.
///
/// # Examples
///
/// ```
/// use network_manager::device;
/// use network_manager::manager;
/// let manager = manager::new();
/// let devices = device::list(&manager).unwrap();
/// println!("{:?}", devices);
/// ```
pub fn list(manager: &NetworkManager) -> Result<Vec<Device>, String> {
    let device_paths = try!(manager.get_devices());

    let mut result = Vec::new();

    for device_path in device_paths {
        let interface = try!(manager.get_device_interface(&device_path));

        let device_type = try!(manager.get_device_type(&device_path));

        let state = try!(manager.get_device_state(&device_path));

        let real = try!(manager.is_device_real(&device_path));

        let device = Device {
            interface: interface,
            path: device_path.clone(),
            device_type: device_type,
            state: state,
            real: real,
        };

        result.push(device);
    }

    Ok(result)
}

#[test]
fn test_list_function() {
    let manager = manager::new();
    let devices = list(&manager).unwrap();
    assert!(devices.len() > 0);
}

/// Enables a Network Manager device.
///
/// # Examples
///
/// ```
/// use network_manager::device;
/// use network_manager::manager;
/// let manager = manager::new();
/// let devices = device::list(&manager).unwrap();
/// let device = &devices[0];
/// let state = device::enable(&manager, device, 10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(manager: &NetworkManager, c: &Device, t: i32) -> Result<DeviceState, String> {
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
/// use network_manager::device;
/// use network_manager::manager;
/// let manager = manager::new();
/// let devices = device::list(&manager).unwrap();
/// let device = &devices[0];
/// let state = device::disable(&manager, device, 10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(manager: &NetworkManager, c: &Device, t: i32) -> Result<DeviceState, String> {
    // Disable device

    if t != 0 {
        // Wait until the device state is 'Unavailable' or
        // until the time has elapsed
    }

    Ok(DeviceState::Unavailable)
}
