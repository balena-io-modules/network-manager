use dbus::{Connection, ConnPath, BusType};
use dbus::stdintf::OrgFreedesktopDBusProperties;

use general::{NM_DEVICE_INTERFACE, NM_SERVICE_INTERFACE, NM_SERVICE_PATH};
use dbus_helper::{variant_to_string_list, manager_path, property_as_string, property_as_i64,
                  property_as_bool};


#[derive(Debug)]
pub struct Device {
    interface: String,
    path: String,
    device_type: DeviceType,
    state: DeviceState,
    real: bool,
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


/// Get a list of Network Manager devices.
///
/// # Examples
///
/// ```
/// let devices = network_manager::device::list().unwrap();
/// println!("{:?}", devices);
/// ```
pub fn list() -> Result<Vec<Device>, String> {
    let connection = Connection::get_private(BusType::System).unwrap();

    let path = manager_path(&connection, NM_SERVICE_PATH);

    let devices = path.get(NM_SERVICE_INTERFACE, "Devices").unwrap();

    let device_paths = variant_to_string_list(devices).unwrap();

    let mut result = Vec::new();

    for device_path in device_paths {
        let path = manager_path(&connection, &device_path);

        let interface = device_string(&path, "Interface").unwrap();

        let device_type = DeviceType::from(device_i64(&path, "DeviceType").unwrap());

        let state = DeviceState::from(device_i64(&path, "State").unwrap());

        let real = device_bool(&path, "Real").unwrap();

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
    let devices = list().unwrap();
    assert!(devices.len() > 0);
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

#[inline]
fn device_string(path: &ConnPath<&Connection>, property: &str) -> Option<String> {
    property_as_string(path, NM_DEVICE_INTERFACE, property)
}

#[inline]
fn device_i64(path: &ConnPath<&Connection>, property: &str) -> Option<i64> {
    property_as_i64(path, NM_DEVICE_INTERFACE, property)
}

#[inline]
fn device_bool(path: &ConnPath<&Connection>, property: &str) -> Option<bool> {
    property_as_bool(path, NM_DEVICE_INTERFACE, property)
}
