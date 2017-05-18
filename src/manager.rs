use std::rc::Rc;
use std::cell::RefCell;

use dbus_nm::DBusNetworkManager;

use connection::{Connection, get_connections, get_active_connections};
use device::{Device, get_devices, get_device_by_interface};
use service::{start_service, stop_service, get_service_state, ServiceState, Error};


pub struct NetworkManager {
    dbus_manager: Rc<RefCell<DBusNetworkManager>>,
}

impl NetworkManager {
    pub fn new() -> Self {
        NetworkManager { dbus_manager: Rc::new(RefCell::new(DBusNetworkManager::new())) }
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
    pub fn start_service(timeout: u64) -> Result<ServiceState, Error> {
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
    pub fn stop_service(timeout: u64) -> Result<ServiceState, Error> {
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
    pub fn get_service_state() -> Result<ServiceState, Error> {
        get_service_state()
    }

    pub fn set_method_timeout(&mut self, timeout: u64) {
        self.dbus_manager.borrow_mut().set_method_timeout(timeout);
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
    pub fn get_connections(&self) -> Result<Vec<Connection>, String> {
        get_connections(&self.dbus_manager)
    }

    pub fn get_active_connections(&self) -> Result<Vec<Connection>, String> {
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
    pub fn get_devices(&self) -> Result<Vec<Device>, String> {
        get_devices(&self.dbus_manager)
    }

    pub fn get_device_by_interface(&self, interface: &str) -> Result<Device, String> {
        get_device_by_interface(&self.dbus_manager, interface)
    }

    pub fn get_state(&self) -> Result<NetworkManagerState, String> {
        self.dbus_manager.borrow().get_state()
    }

    pub fn get_connectivity(&self) -> Result<Connectivity, String> {
        self.dbus_manager.borrow().check_connectivity()
    }

    pub fn is_networking_enabled(&self) -> Result<bool, String> {
        self.dbus_manager.borrow().is_networking_enabled()
    }

    pub fn is_wireless_enabled(&self) -> Result<bool, String> {
        self.dbus_manager.borrow().is_wireless_enabled()
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

impl From<i64> for NetworkManagerState {
    fn from(state: i64) -> Self {
        match state {
            0 => NetworkManagerState::Unknown,
            10 => NetworkManagerState::Asleep,
            20 => NetworkManagerState::Disconnected,
            30 => NetworkManagerState::Disconnecting,
            40 => NetworkManagerState::Connecting,
            50 => NetworkManagerState::ConnectedLocal,
            60 => NetworkManagerState::ConnectedSite,
            70 => NetworkManagerState::ConnectedGlobal,
            _ => NetworkManagerState::Unknown,

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

impl From<i64> for Connectivity {
    fn from(state: i64) -> Self {
        match state {
            0 => Connectivity::Unknown,
            1 => Connectivity::None,
            2 => Connectivity::Portal,
            3 => Connectivity::Limited,
            4 => Connectivity::Full,
            _ => Connectivity::Unknown,

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
    fn test_start_stop_service() {
        let s = NetworkManager::get_service_state().unwrap();

        assert!(s == ServiceState::Active || s == ServiceState::Inactive);

        match s {
            ServiceState::Active => {
                NetworkManager::stop_service(10).unwrap();
                assert_eq!(ServiceState::Inactive,
                           NetworkManager::get_service_state().unwrap());

                NetworkManager::start_service(10).unwrap();
                assert_eq!(ServiceState::Active,
                           NetworkManager::get_service_state().unwrap());
            }
            ServiceState::Inactive => {
                NetworkManager::start_service(10).unwrap();
                assert_eq!(ServiceState::Active,
                           NetworkManager::get_service_state().unwrap());

                NetworkManager::stop_service(10).unwrap();
                assert_eq!(ServiceState::Inactive,
                           NetworkManager::get_service_state().unwrap());
            }
            _ => (),
        }
    }
}
