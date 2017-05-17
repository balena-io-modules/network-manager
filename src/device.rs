use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use dbus_nm::DBusNetworkManager;

use wifi::{WiFiDevice, new_wifi_device};


#[derive(Clone)]
pub struct Device {
    dbus_manager: Rc<RefCell<DBusNetworkManager>>,
    path: String,
    interface: String,
    device_type: DeviceType,
}

impl Device {
    fn init(dbus_manager: &Rc<RefCell<DBusNetworkManager>>, path: &str) -> Result<Self, String> {
        let interface = try!(dbus_manager.borrow().get_device_interface(path));

        let device_type = try!(dbus_manager.borrow().get_device_type(path));

        Ok(Device {
               dbus_manager: dbus_manager.clone(),
               path: path.to_string(),
               interface: interface,
               device_type: device_type,
           })
    }

    pub fn device_type(&self) -> &DeviceType {
        &self.device_type
    }

    pub fn interface(&self) -> &str {
        &self.interface
    }

    pub fn get_state(&self) -> Result<DeviceState, String> {
        self.dbus_manager.borrow().get_device_state(&self.path)
    }

    pub fn as_wifi_device<'a>(&'a self) -> Option<WiFiDevice<'a>> {
        if self.device_type == DeviceType::WiFi {
            Some(new_wifi_device(&self.dbus_manager, self))
        } else {
            None
        }
    }

    /// Connects a Network Manager device.
    ///
    /// Examples
    ///
    /// ```
    /// use network_manager::{NetworkManager, DeviceType};
    /// let manager = NetworkManager::new();
    /// let devices = manager.get_devices().unwrap();
    /// let i = devices.iter().position(|ref d| *d.device_type() == DeviceType::WiFi).unwrap();
    /// devices[i].connect().unwrap();
    /// ```
    pub fn connect(&self) -> Result<DeviceState, String> {
        let state = try!(self.get_state());

        match state {
            DeviceState::Activated => Ok(DeviceState::Activated),
            _ => {
                try!(self.dbus_manager.borrow().connect_device(&self.path));

                wait(self,
                     DeviceState::Activated,
                     self.dbus_manager.borrow().method_timeout())
            }
        }
    }

    /// Disconnect a Network Manager device.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::{NetworkManager, DeviceType};
    /// let manager = NetworkManager::new();
    /// let devices = manager.get_devices().unwrap();
    /// let i = devices.iter().position(|ref d| *d.device_type() == DeviceType::WiFi).unwrap();
    /// devices[i].disconnect().unwrap();
    /// ```
    pub fn disconnect(&self) -> Result<DeviceState, String> {
        let state = try!(self.get_state());

        match state {
            DeviceState::Disconnected => Ok(DeviceState::Disconnected),
            _ => {
                try!(self.dbus_manager.borrow().disconnect_device(&self.path));

                wait(self,
                     DeviceState::Disconnected,
                     self.dbus_manager.borrow().method_timeout())
            }
        }
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Device {{ path: {:?}, interface: {:?}, device_type: {:?} }}",
               self.path,
               self.interface,
               self.device_type)
    }
}

pub trait PathGetter {
    fn path(&self) -> &str;
}

impl PathGetter for Device {
    fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Clone, Debug, PartialEq)]
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


#[derive(Clone, Debug, PartialEq)]
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


pub fn get_devices(dbus_manager: &Rc<RefCell<DBusNetworkManager>>) -> Result<Vec<Device>, String> {
    let device_paths = try!(dbus_manager.borrow().get_devices());

    let mut result = Vec::new();

    for path in device_paths {
        let device = try!(Device::init(dbus_manager, &path));

        result.push(device);
    }

    Ok(result)
}

pub fn get_device_by_interface(dbus_manager: &Rc<RefCell<DBusNetworkManager>>,
                               interface: &str)
                               -> Result<Device, String> {
    let path = try!(dbus_manager.borrow().get_device_by_interface(interface));

    Device::init(dbus_manager, &path)
}

pub fn get_active_connection_devices(dbus_manager: &Rc<RefCell<DBusNetworkManager>>,
                                     active_path: &str)
                                     -> Result<Vec<Device>, String> {
    let device_paths = try!(dbus_manager
                                .borrow()
                                .get_active_connection_devices(active_path));

    let mut result = Vec::new();

    for path in device_paths {
        let device = try!(Device::init(dbus_manager, &path));

        result.push(device);
    }

    Ok(result)
}

fn wait(device: &Device, target_state: DeviceState, timeout: u64) -> Result<DeviceState, String> {
    if timeout == 0 {
        return device.get_state();
    }

    let mut total_time = 0;

    loop {
        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        let state = try!(device.get_state());

        total_time += 1;

        if state == target_state || total_time >= timeout {
            return Ok(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::NetworkManager;

    use super::*;

    #[test]
    fn test_device_connect_disconnect() {
        let manager = NetworkManager::new();

        let devices = manager.get_devices().unwrap();

        let i = devices
            .iter()
            .position(|ref d| d.device_type == DeviceType::WiFi)
            .unwrap();
        let device = &devices[i];

        let state = device.get_state().unwrap();

        if state == DeviceState::Activated {
            let state = device.disconnect().unwrap();
            assert_eq!(DeviceState::Disconnected, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));

            let state = device.connect().unwrap();
            assert_eq!(DeviceState::Activated, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        } else {
            let state = device.connect().unwrap();
            assert_eq!(DeviceState::Activated, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));

            let state = device.disconnect().unwrap();
            assert_eq!(DeviceState::Disconnected, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        }
    }
}
