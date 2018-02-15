use std::rc::Rc;
use std::fmt;

use errors::*;
use dbus_nm::DBusNetworkManager;

use wifi::{new_wifi_device, WiFiDevice};

#[derive(Clone)]
pub struct Device {
    dbus_manager: Rc<DBusNetworkManager>,
    path: String,
    interface: String,
    device_type: DeviceType,
}

impl Device {
    fn init(dbus_manager: &Rc<DBusNetworkManager>, path: &str) -> Result<Self> {
        let interface = dbus_manager.get_device_interface(path)?;

        let device_type = dbus_manager.get_device_type(path)?;

        Ok(Device {
            dbus_manager: Rc::clone(dbus_manager),
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

    pub fn get_state(&self) -> Result<DeviceState> {
        self.dbus_manager.get_device_state(&self.path)
    }

    pub fn as_wifi_device(&self) -> Option<WiFiDevice> {
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
    pub fn connect(&self) -> Result<DeviceState> {
        let state = self.get_state()?;

        match state {
            DeviceState::Activated => Ok(DeviceState::Activated),
            _ => {
                self.dbus_manager.connect_device(&self.path)?;

                wait(
                    self,
                    &DeviceState::Activated,
                    self.dbus_manager.method_timeout(),
                )
            },
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
    pub fn disconnect(&self) -> Result<DeviceState> {
        let state = self.get_state()?;

        match state {
            DeviceState::Disconnected => Ok(DeviceState::Disconnected),
            _ => {
                self.dbus_manager.disconnect_device(&self.path)?;

                wait(
                    self,
                    &DeviceState::Disconnected,
                    self.dbus_manager.method_timeout(),
                )
            },
        }
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Device {{ path: {:?}, interface: {:?}, device_type: {:?} }}",
            self.path, self.interface, self.device_type
        )
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
    Ethernet,
    WiFi,
    Unused1,
    Unused2,
    Bt,
    OlpcMesh,
    Wimax,
    Modem,
    Infiniband,
    Bond,
    Vlan,
    Adsl,
    Bridge,
    Generic,
    Team,
    Tun,
    IpTunnel,
    Macvlan,
    Vxlan,
    Veth,
    Macsec,
    Dummy,
}

impl From<i64> for DeviceType {
    fn from(device_type: i64) -> Self {
        match device_type {
            0 => DeviceType::Unknown,
            1 => DeviceType::Ethernet,
            2 => DeviceType::WiFi,
            3 => DeviceType::Unused1,
            4 => DeviceType::Unused2,
            5 => DeviceType::Bt,
            6 => DeviceType::OlpcMesh,
            7 => DeviceType::Wimax,
            8 => DeviceType::Modem,
            9 => DeviceType::Infiniband,
            10 => DeviceType::Bond,
            11 => DeviceType::Vlan,
            12 => DeviceType::Adsl,
            13 => DeviceType::Bridge,
            14 => DeviceType::Generic,
            15 => DeviceType::Team,
            16 => DeviceType::Tun,
            17 => DeviceType::IpTunnel,
            18 => DeviceType::Macvlan,
            19 => DeviceType::Vxlan,
            20 => DeviceType::Veth,
            21 => DeviceType::Macsec,
            22 => DeviceType::Dummy,
            _ => {
                warn!("Undefined device type: {}", device_type);
                DeviceType::Unknown
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceState {
    Unknown,
    Unmanaged,
    Unavailable,
    Disconnected,
    Prepare,
    Config,
    NeedAuth,
    IpConfig,
    IpCheck,
    Secondaries,
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
            40 => DeviceState::Prepare,
            50 => DeviceState::Config,
            60 => DeviceState::NeedAuth,
            70 => DeviceState::IpConfig,
            80 => DeviceState::IpCheck,
            90 => DeviceState::Secondaries,
            100 => DeviceState::Activated,
            110 => DeviceState::Deactivating,
            120 => DeviceState::Failed,
            _ => {
                warn!("Undefined device state: {}", state);
                DeviceState::Unknown
            },
        }
    }
}

pub fn get_devices(dbus_manager: &Rc<DBusNetworkManager>) -> Result<Vec<Device>> {
    let device_paths = dbus_manager.get_devices()?;

    let mut result = Vec::with_capacity(device_paths.len());

    for path in device_paths {
        let device = Device::init(dbus_manager, &path)?;

        result.push(device);
    }

    Ok(result)
}

pub fn get_device_by_interface(
    dbus_manager: &Rc<DBusNetworkManager>,
    interface: &str,
) -> Result<Device> {
    let path = dbus_manager.get_device_by_interface(interface)?;

    Device::init(dbus_manager, &path)
}

pub fn get_active_connection_devices(
    dbus_manager: &Rc<DBusNetworkManager>,
    active_path: &str,
) -> Result<Vec<Device>> {
    let device_paths = dbus_manager.get_active_connection_devices(active_path)?;

    let mut result = Vec::with_capacity(device_paths.len());

    for path in device_paths {
        let device = Device::init(dbus_manager, &path)?;

        result.push(device);
    }

    Ok(result)
}

fn wait(device: &Device, target_state: &DeviceState, timeout: u64) -> Result<DeviceState> {
    if timeout == 0 {
        return device.get_state();
    }

    debug!("Waiting for device state: {:?}", target_state);

    let mut total_time = 0;

    loop {
        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        let state = device.get_state()?;

        total_time += 1;

        if state == *target_state {
            debug!(
                "Device target state reached: {:?} / {}s elapsed",
                state, total_time
            );

            return Ok(state);
        } else if total_time >= timeout {
            debug!(
                "Timeout reached in waiting for device state ({:?}): {:?} / {}s elapsed",
                target_state, state, total_time
            );

            return Ok(state);
        }

        debug!(
            "Still waiting for device state ({:?}): {:?} / {}s elapsed",
            target_state, state, total_time
        );
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
