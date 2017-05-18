use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use ascii::AsAsciiStr;

use dbus_nm::DBusNetworkManager;

use wifi::Security;
use device::{Device, get_active_connection_devices};


#[derive(Clone)]
pub struct Connection {
    dbus_manager: Rc<RefCell<DBusNetworkManager>>,
    path: String,
    settings: ConnectionSettings,
}

impl Connection {
    fn init(dbus_manager: &Rc<RefCell<DBusNetworkManager>>, path: &str) -> Result<Self, String> {
        let settings = try!(dbus_manager.borrow().get_connection_settings(path));

        Ok(Connection {
               dbus_manager: dbus_manager.clone(),
               path: path.to_string(),
               settings: settings,
           })
    }

    pub fn settings(&self) -> &ConnectionSettings {
        &self.settings
    }

    pub fn get_state(&self) -> Result<ConnectionState, String> {
        let active_path_option = try!(get_connection_active_path(&self.dbus_manager.borrow(),
                                                                 &self.path));

        if let Some(active_path) = active_path_option {
            let state = try!(self.dbus_manager
                                 .borrow()
                                 .get_connection_state(&active_path));

            Ok(state)
        } else {
            Ok(ConnectionState::Deactivated)
        }
    }

    pub fn delete(&self) -> Result<(), String> {
        self.dbus_manager.borrow().delete_connection(&self.path)
    }

    /// Activate a Network Manager connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use network_manager::NetworkManager;
    /// let manager = NetworkManager::new();
    /// let connections = manager.get_connections().unwrap();
    /// connections[0].activate().unwrap();
    /// ```
    pub fn activate(&self) -> Result<ConnectionState, String> {
        let state = try!(self.get_state());

        match state {
            ConnectionState::Activated => Ok(ConnectionState::Activated),
            ConnectionState::Activating => {
                wait(self,
                     ConnectionState::Activated,
                     self.dbus_manager.borrow().method_timeout())
            }
            ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
            _ => {
                try!(self.dbus_manager.borrow().activate_connection(&self.path));

                wait(self,
                     ConnectionState::Activated,
                     self.dbus_manager.borrow().method_timeout())
            }
        }
    }

    /// Deactivates a Network Manager connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use network_manager::NetworkManager;
    /// let manager = NetworkManager::new();
    /// let connections = manager.get_connections().unwrap();
    /// connections[0].deactivate().unwrap();
    /// ```
    pub fn deactivate(&self) -> Result<ConnectionState, String> {
        let state = try!(self.get_state());

        match state {
            ConnectionState::Deactivated => Ok(ConnectionState::Deactivated),
            ConnectionState::Deactivating => {
                wait(self,
                     ConnectionState::Deactivated,
                     self.dbus_manager.borrow().method_timeout())
            }
            ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
            _ => {
                let active_path_option =
                    try!(get_connection_active_path(&self.dbus_manager.borrow(), &self.path));

                if let Some(active_path) = active_path_option {
                    try!(self.dbus_manager
                             .borrow()
                             .deactivate_connection(&active_path));

                    wait(self,
                         ConnectionState::Deactivated,
                         self.dbus_manager.borrow().method_timeout())
                } else {
                    Ok(ConnectionState::Deactivated)
                }
            }
        }
    }

    pub fn get_devices(&self) -> Result<Vec<Device>, String> {
        let active_path_option = try!(get_connection_active_path(&self.dbus_manager.borrow(),
                                                                 &self.path));

        if let Some(active_path) = active_path_option {
            get_active_connection_devices(&self.dbus_manager, &active_path)
        } else {
            Ok(vec![])
        }
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        i32::from(self).cmp(&i32::from(other))
    }
}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Connection) -> bool {
        i32::from(self) == i32::from(other)
    }
}

impl Eq for Connection {}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Connection {{ path: {:?}, settings: {:?} }}",
               self.path,
               self.settings)
    }
}

impl<'a> From<&'a Connection> for i32 {
    fn from(val: &Connection) -> i32 {
        val.clone()
            .path
            .rsplit('/')
            .nth(0)
            .unwrap()
            .parse::<i32>()
            .unwrap()
    }
}


#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ConnectionSettings {
    pub id: String,
    pub uuid: String,
    pub ssid: String,
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConnectionState {
    Unknown = 0,
    Activating = 1,
    Activated = 2,
    Deactivating = 3,
    Deactivated = 4,
}

impl From<i64> for ConnectionState {
    fn from(state: i64) -> Self {
        match state {
            0 => ConnectionState::Unknown,
            1 => ConnectionState::Activating,
            2 => ConnectionState::Activated,
            3 => ConnectionState::Deactivating,
            4 => ConnectionState::Deactivated,
            _ => ConnectionState::Unknown,
        }
    }
}


pub fn get_connections(dbus_manager: &Rc<RefCell<DBusNetworkManager>>)
                       -> Result<Vec<Connection>, String> {
    let paths = try!(dbus_manager.borrow().list_connections());

    let mut connections = Vec::with_capacity(paths.len());

    for path in &paths {
        connections.push(try!(Connection::init(dbus_manager, path)))
    }

    connections.sort();

    Ok(connections)
}


pub fn get_active_connections(dbus_manager: &Rc<RefCell<DBusNetworkManager>>)
                              -> Result<Vec<Connection>, String> {
    let active_paths = try!(dbus_manager.borrow().get_active_connections());

    let mut connections = Vec::with_capacity(active_paths.len());

    for active_path in active_paths {
        if let Some(path) = dbus_manager
               .borrow()
               .get_active_connection_path(&active_path) {
            connections.push(try!(Connection::init(dbus_manager, &path)))
        }
    }

    connections.sort();

    Ok(connections)
}


pub fn connect_to_access_point<T>(dbus_manager: &Rc<RefCell<DBusNetworkManager>>,
                                  device_path: &str,
                                  access_point_path: &str,
                                  ssid: &str,
                                  security: &Security,
                                  password: &T)
                                  -> Result<(Connection, ConnectionState), String>
    where T: AsAsciiStr + ?Sized
{
    let (path, _) = try!(dbus_manager
                             .borrow()
                             .add_and_activate_connection(device_path,
                                                          access_point_path,
                                                          ssid,
                                                          security,
                                                          password));

    let connection = try!(Connection::init(dbus_manager, &path));

    let state = try!(wait(&connection,
                          ConnectionState::Activated,
                          dbus_manager.borrow().method_timeout()));

    Ok((connection, state))
}

pub fn create_hotspot<T>(dbus_manager: &Rc<RefCell<DBusNetworkManager>>,
                         device_path: &str,
                         interface: &str,
                         ssid: &str,
                         password: Option<&T>)
                         -> Result<(Connection, ConnectionState), String>
    where T: AsAsciiStr + ?Sized
{
    let (path, _) = try!(dbus_manager
                             .borrow()
                             .create_hotspot(device_path, interface, ssid, password));

    let connection = try!(Connection::init(dbus_manager, &path));

    let state = try!(wait(&connection,
                          ConnectionState::Activated,
                          dbus_manager.borrow().method_timeout()));

    Ok((connection, state))
}

fn get_connection_active_path(dbus_manager: &DBusNetworkManager,
                              connection_path: &str)
                              -> Result<Option<String>, String> {
    let active_paths = try!(dbus_manager.get_active_connections());

    for active_path in active_paths {
        if let Some(settings_path) = dbus_manager.get_active_connection_path(&active_path) {
            if connection_path == settings_path {
                return Ok(Some(active_path));
            }
        }
    }

    Ok(None)
}

fn wait(connection: &Connection,
        target_state: ConnectionState,
        timeout: u64)
        -> Result<ConnectionState, String> {
    if timeout == 0 {
        return connection.get_state();
    }

    let mut total_time = 0;

    loop {
        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        let state = try!(connection.get_state());

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
    fn test_connection_enable_disable() {
        let manager = NetworkManager::new();

        let connections = manager.get_connections().unwrap();

        // set environment variable $TEST_WIFI_SSID with the wifi's SSID that you want to test
        // e.g.  export TEST_WIFI_SSID="Resin.io Wifi"
        let wifi_env_var = "TEST_WIFI_SSID";
        let connection = match ::std::env::var(wifi_env_var) {
            Ok(ssid) => {
                connections
                    .iter()
                    .filter(|c| c.settings().ssid == ssid)
                    .nth(0)
                    .unwrap()
                    .clone()
            }
            Err(e) => {
                panic!("couldn't retrieve environment variable {}: {}",
                       wifi_env_var,
                       e)
            }
        };

        let state = connection.get_state().unwrap();

        if state == ConnectionState::Activated {
            let state = connection.deactivate().unwrap();
            assert_eq!(ConnectionState::Deactivated, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));

            let state = connection.activate().unwrap();
            assert_eq!(ConnectionState::Activated, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        } else {
            let state = connection.activate().unwrap();
            assert_eq!(ConnectionState::Activated, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));

            let state = connection.deactivate().unwrap();
            assert_eq!(ConnectionState::Deactivated, state);

            ::std::thread::sleep(::std::time::Duration::from_secs(5));
        }
    }
}
