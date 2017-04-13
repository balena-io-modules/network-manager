use std;

use manager;
use manager::NetworkManager;


#[derive(Debug)]
pub struct Device {
    pub interface: String,
    pub path: String,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub real: bool,
}

impl Device {
    pub fn from_path(manager: &NetworkManager, path: &String) -> Result<Device, String> {
        let interface = try!(manager.get_device_interface(&path));

        let device_type = try!(manager.get_device_type(&path));

        let state = try!(manager.get_device_state(&path));

        let real = try!(manager.is_device_real(&path));

        Ok(Device {
               interface: interface,
               path: path.clone(),
               device_type: device_type,
               state: state,
               real: real,
           })
    }
}


#[derive(Debug, PartialEq)]
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


#[derive(Debug, PartialEq)]
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

    for path in device_paths {
        let device = try!(Device::from_path(manager, &path));

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

/// Connects a Network Manager device.
///
/// # Examples
///
/// # ```
/// # use network_manager::device;
/// # use network_manager::manager;
/// # let manager = manager::new();
/// # let mut devices = device::list(&manager).unwrap();
/// # let i = devices.iter().position(|ref d| d.device_type == device::DeviceType::WiFi).unwrap();
/// # let device = &mut devices[i];
/// # device::connect(&manager, device, 10).unwrap();
/// # ```
pub fn connect(manager: &NetworkManager, device: &mut Device, time_out: i32) -> Result<(), String> {
    match device.state {
        DeviceState::Activated => Ok(()),
        _ => {
            try!(manager.activate_device(&device.path));

            wait(manager, device, time_out, DeviceState::Activated)
        }
    }
}

/// Disconnect a Network Manager device.
///
/// # Examples
///
/// ```
/// use network_manager::device;
/// use network_manager::manager;
/// let manager = manager::new();
/// let mut devices = device::list(&manager).unwrap();
/// let i = devices.iter().position(|ref d| d.device_type == device::DeviceType::WiFi).unwrap();
/// let device = &mut devices[i];
/// device::disconnect(&manager, device, 10).unwrap();
/// ```
pub fn disconnect(manager: &NetworkManager,
                  device: &mut Device,
                  time_out: i32)
                  -> Result<(), String> {
    match device.state {
        DeviceState::Disconnected => Ok(()),
        _ => {
            try!(manager.disconnect_device(&device.path));

            wait(manager, device, time_out, DeviceState::Disconnected)
        }
    }
}

fn wait(manager: &NetworkManager,
        device: &mut Device,
        time_out: i32,
        target_state: DeviceState)
        -> Result<(), String> {
    if time_out == 0 {
        return Ok(());
    }

    let mut total_time = 0;

    while total_time < time_out {
        device.state = try!(manager.get_device_state(&device.path));

        if device.state == target_state {
            return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        total_time += 1;
    }

    Err("service timed out".to_string())
}


#[test]
fn test_connect_disconnect_functions() {
    let manager = manager::new();

    let mut devices = list(&manager).unwrap();

    let i = devices
        .iter()
        .position(|ref d| d.device_type == DeviceType::WiFi)
        .unwrap();
    let device = &mut devices[i];

    if device.state == DeviceState::Activated {
        disconnect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Disconnected, device.state);

        connect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Activated, device.state);
    } else {
        connect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Activated, device.state);

        disconnect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Disconnected, device.state);
    }
}
