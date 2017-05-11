use std;

use dbus_nm;
use dbus_nm::DBusNetworkManager;


#[derive(Debug)]
pub struct Device {
    pub path: String,
    pub interface: String,
    pub device_type: DeviceType,
}

impl Device {
    pub fn from_path(manager: &DBusNetworkManager, path: &String) -> Result<Device, String> {
        let interface = try!(manager.get_device_interface(&path));

        let device_type = try!(manager.get_device_type(&path));

        Ok(Device {
               interface: interface,
               path: path.clone(),
               device_type: device_type,
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
/// use network_manager::dbus_nm;
/// let manager = dbus_nm::new();
/// let devices = device::list(&manager).unwrap();
/// println!("{:?}", devices);
/// ```
pub fn list(manager: &DBusNetworkManager) -> Result<Vec<Device>, String> {
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
    let manager = dbus_nm::new();
    let devices = list(&manager).unwrap();
    assert!(devices.len() > 0);
}

/// Connects a Network Manager device.
///
/// Examples
///
/// ```
/// use network_manager::device;
/// use network_manager::manager;
/// let manager = dbus_nm::new();
/// let devices = device::list(&manager).unwrap();
/// let i = devices.iter().position(|ref d| d.device_type == device::DeviceType::WiFi).unwrap();
/// let device = &devices[i];
/// device::connect(&manager, device, 10).unwrap();
/// ```
pub fn connect(manager: &DBusNetworkManager,
               device: &Device,
               time_out: i32)
               -> Result<DeviceState, String> {
    let state = try!(get_device_state(manager, device));

    match state {
        DeviceState::Activated => Ok(DeviceState::Activated),
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
/// let manager = dbus_nm::new();
/// let devices = device::list(&manager).unwrap();
/// let i = devices.iter().position(|ref d| d.device_type == device::DeviceType::WiFi).unwrap();
/// let device = &devices[i];
/// device::disconnect(&manager, device, 10).unwrap();
/// ```
pub fn disconnect(manager: &DBusNetworkManager,
                  device: &Device,
                  time_out: i32)
                  -> Result<DeviceState, String> {
    let state = try!(get_device_state(manager, device));

    match state {
        DeviceState::Disconnected => Ok(DeviceState::Disconnected),
        _ => {
            try!(manager.disconnect_device(&device.path));

            wait(manager, device, time_out, DeviceState::Disconnected)
        }
    }
}

fn wait(manager: &DBusNetworkManager,
        device: &Device,
        time_out: i32,
        target_state: DeviceState)
        -> Result<DeviceState, String> {
    if time_out == 0 {
        return get_device_state(manager, device);
    }

    let mut total_time = 0;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let state = try!(get_device_state(manager, device));

        total_time += 1;

        if state == target_state || total_time >= time_out {
            return Ok(state);
        }
    }
}


pub fn get_device_state(manager: &DBusNetworkManager,
                        device: &Device)
                        -> Result<DeviceState, String> {
    manager.get_device_state(&device.path)
}

#[test]
fn test_connect_disconnect_functions() {
    let manager = dbus_nm::new();

    let devices = list(&manager).unwrap();

    let i = devices
        .iter()
        .position(|ref d| d.device_type == DeviceType::WiFi)
        .unwrap();
    let device = &devices[i];

    let state = get_device_state(&manager, device).unwrap();

    if state == DeviceState::Activated {
        let state = disconnect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Disconnected, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        let state = connect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Activated, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));
    } else {
        let state = connect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Activated, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        let state = disconnect(&manager, device, 10).unwrap();
        assert_eq!(DeviceState::Disconnected, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));
    }
}
