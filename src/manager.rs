use std::rc::Rc;

use errors::*;
use dbus_nm::DBusNetworkManager;

use connection::{get_active_connections, get_connections, Connection};
use device::{get_device_by_interface, get_devices, Device};
use service::{get_service_state, start_service, stop_service, ServiceState};

pub struct NetworkManager {
    dbus_manager: Rc<DBusNetworkManager>,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager {
            dbus_manager: Rc::new(DBusNetworkManager::new(None)),
        }
    }

    pub fn with_method_timeout(timeout: u64) -> Self {
        NetworkManager {
            dbus_manager: Rc::new(DBusNetworkManager::new(Some(timeout))),
        }
    }

    /// Starts the Network Manager service.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::NetworkManager;
    /// let state = NetworkManager::start_service(10).unwrap();
    /// println!("{:?}", state);
    /// ```
    pub fn start_service(timeout: u64) -> Result<ServiceState> {
        start_service(timeout)
    }

    /// Stops the Network Manager service.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::NetworkManager;
    /// let state = NetworkManager::stop_service(10).unwrap();
    /// println!("{:?}", state);
    /// ```
    pub fn stop_service(timeout: u64) -> Result<ServiceState> {
        stop_service(timeout)
    }

    /// Gets the state of the Network Manager service.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::NetworkManager;
    /// let state = NetworkManager::get_service_state().unwrap();
    /// println!("{:?}", state);
    /// ```
    pub fn get_service_state() -> Result<ServiceState> {
        get_service_state()
    }

    /// Get a list of Network Manager connections sorted by path.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::NetworkManager;
    /// let manager = NetworkManager::new();
    /// let connections = manager.get_connections().unwrap();
    /// println!("{:?}", connections);
    /// ```
    pub fn get_connections(&self) -> Result<Vec<Connection>> {
        get_connections(&self.dbus_manager)
    }

    pub fn get_active_connections(&self) -> Result<Vec<Connection>> {
        get_active_connections(&self.dbus_manager)
    }

    /// Get a list of Network Manager devices.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::NetworkManager;
    /// let manager = NetworkManager::new();
    /// let devices = manager.get_devices().unwrap();
    /// println!("{:?}", devices);
    /// ```
    pub fn get_devices(&self) -> Result<Vec<Device>> {
        get_devices(&self.dbus_manager)
    }

    pub fn get_device_by_interface(&self, interface: &str) -> Result<Device> {
        get_device_by_interface(&self.dbus_manager, interface)
    }

    pub fn get_state(&self) -> Result<NetworkManagerState> {
        self.dbus_manager.get_state()
    }

    pub fn get_connectivity(&self) -> Result<Connectivity> {
        self.dbus_manager.check_connectivity()
    }

    pub fn is_networking_enabled(&self) -> Result<bool> {
        self.dbus_manager.is_networking_enabled()
    }

    pub fn is_wireless_enabled(&self) -> Result<bool> {
        self.dbus_manager.is_wireless_enabled()
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkManagerState {
    Unknown,
    Asleep,
    Disconnected,
    Disconnecting,
    Connecting,
    ConnectedLocal,
    ConnectedSite,
    ConnectedGlobal,
}

impl From<u32> for NetworkManagerState {
    fn from(state: u32) -> Self {
        match state {
            0 => NetworkManagerState::Unknown,
            10 => NetworkManagerState::Asleep,
            20 => NetworkManagerState::Disconnected,
            30 => NetworkManagerState::Disconnecting,
            40 => NetworkManagerState::Connecting,
            50 => NetworkManagerState::ConnectedLocal,
            60 => NetworkManagerState::ConnectedSite,
            70 => NetworkManagerState::ConnectedGlobal,
            _ => {
                warn!("Undefined Network Manager state: {}", state);
                NetworkManagerState::Unknown
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Connectivity {
    Unknown,
    None,
    Portal,
    Limited,
    Full,
}

impl From<u32> for Connectivity {
    fn from(state: u32) -> Self {
        match state {
            0 => Connectivity::Unknown,
            1 => Connectivity::None,
            2 => Connectivity::Portal,
            3 => Connectivity::Limited,
            4 => Connectivity::Full,
            _ => {
                warn!("Undefined connectivity state: {}", state);
                Connectivity::Unknown
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_connections() {
        let manager = NetworkManager::new();
        let connections = manager.get_connections().unwrap();
        assert!(connections.len() > 0);
    }

    #[test]
    fn test_get_devices() {
        let manager = NetworkManager::new();
        let devices = manager.get_devices().unwrap();
        assert!(devices.len() > 0);
    }

    #[test]
    fn test_get_connectivity() {
        let manager = NetworkManager::new();
        let connectivity = manager.get_connectivity().unwrap();
        assert_eq!(connectivity, Connectivity::Full);
    }

    #[test]
    fn test_start_stop_service() {
        let s = NetworkManager::get_service_state().unwrap();

        assert!(s == ServiceState::Active || s == ServiceState::Inactive);

        match s {
            ServiceState::Active => {
                NetworkManager::stop_service(10).unwrap();
                assert_eq!(
                    ServiceState::Inactive,
                    NetworkManager::get_service_state().unwrap()
                );

                NetworkManager::start_service(10).unwrap();
                assert_eq!(
                    ServiceState::Active,
                    NetworkManager::get_service_state().unwrap()
                );
            },
            ServiceState::Inactive => {
                NetworkManager::start_service(10).unwrap();
                assert_eq!(
                    ServiceState::Active,
                    NetworkManager::get_service_state().unwrap()
                );

                NetworkManager::stop_service(10).unwrap();
                assert_eq!(
                    ServiceState::Inactive,
                    NetworkManager::get_service_state().unwrap()
                );
            },
            _ => (),
        }
    }
}
