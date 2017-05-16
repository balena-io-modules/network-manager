use std::rc::Rc;
use std::cell::RefCell;

use enum_primitive::FromPrimitive;

use dbus_nm::DBusNetworkManager;

use connection::{Connection, get_connections, get_active_connections};
use device::{Device, get_devices};
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


enum_from_primitive!{
#[derive(Debug, PartialEq)]
pub enum NetworkManagerState {
    Unknown = 0,
    Asleep = 10,
    Disconnected = 20,
    Disconnecting = 30,
    Connecting = 40,
    ConnectedLocal = 50,
    ConnectedSite = 60,
    ConnectedGlobal = 70,
}
}

impl From<u32> for NetworkManagerState {
    fn from(val: u32) -> NetworkManagerState {
        NetworkManagerState::from_u32(val).expect("Invalid Network Manager State enum value")
    }
}

impl From<NetworkManagerState> for u32 {
    fn from(val: NetworkManagerState) -> u32 {
        val as u32
    }
}


enum_from_primitive!{
#[derive(Debug, PartialEq)]
pub enum Connectivity { // See https://bugzilla.gnome.org/show_bug.cgi?id=776848
    Unknown = 0,
    None = 1,
    Portal = 2,
    Limited = 3,
    Full = 4,
}
}

impl From<u32> for Connectivity {
    fn from(val: u32) -> Connectivity {
        Connectivity::from_u32(val).expect("Invalid Connectivity enum value")
    }
}

impl From<Connectivity> for u32 {
    fn from(val: Connectivity) -> u32 {
        val as u32
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
