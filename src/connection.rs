extern crate dbus;

use std;
use std::env;

use enum_primitive::FromPrimitive;

use device::{Device, DeviceType};
use wifi::{AccessPoint, Security};
use dbus_nm;
use dbus_nm::DBusNetworkManager;

/// Get a list of Network Manager connections sorted by path.
///
/// # Examples
///
/// ```no_run
/// use network_manager::connection;
/// use network_manager::dbus_nm;
/// let manager = dbus_nm::new();
/// let connections = connection::list(&manager).unwrap();
/// println!("{:?}", connections);
/// ```
pub fn list(manager: &DBusNetworkManager) -> Result<Vec<Connection>, String> {
    let paths = try!(manager.list_connections());

    let mut connections = Vec::new();

    for path in &paths {
        connections.push(try!(get_connection(&manager, path)))
    }

    connections.sort();

    Ok(connections)
}

#[test]
fn test_list_function() {
    let manager = dbus_nm::new();

    let connections = list(&manager).unwrap();
    assert!(connections.len() > 0);

    for (index, val) in connections.iter().enumerate() {
        assert_ne!(Connection { ..Default::default() }, val.clone());
    }
}

/// Creates a Network Manager connection.
pub fn create(manager: &DBusNetworkManager,
              device: &Device,
              access_point: &AccessPoint,
              password: &str,
              time_out: i32)
              -> Result<Connection, String> {
    let (path, _) = try!(manager.add_and_activate_connection(&device.path,
                                                             &access_point.path,
                                                             &access_point.ssid,
                                                             &access_point.security,
                                                             password));

    let connection = try!(get_connection(manager, &path));

    try!(wait(manager, &connection, time_out, ConnectionState::Activated));

    Ok(connection)
}

/// Starts a Wi-Fi hotspot.
pub fn create_hotspot(manager: &DBusNetworkManager,
                      device: &Device,
                      ssid: &str,
                      password: Option<String>,
                      time_out: i32)
                      -> Result<Connection, String> {
    if device.device_type != DeviceType::WiFi {
        return Err("Not a WiFi device".to_string());
    }

    let (path, _) = try!(manager.create_hotspot(&device.path, &device.interface, ssid, password));

    let connection = try!(get_connection(manager, &path));

    try!(wait(manager, &connection, time_out, ConnectionState::Activated));

    Ok(connection)
}

/// Deletes a Network Manager connection.
///
/// # Examples
///
/// ```
/// use network_manager::connection;
/// use network_manager::dbus_nm;
/// let manager = dbus_nm::new();
/// let mut connections = connection::list(&manager).unwrap();
/// connection::delete(&manager, &connections.pop().unwrap()).unwrap();
/// ```
pub fn delete(manager: &DBusNetworkManager, connection: &Connection) -> Result<(), String> {
    manager.delete_connection(&connection.path)
}

/// Enables a Network Manager connection.
///
/// # Examples
///
/// ```no_run
/// use network_manager::connection;
/// use network_manager::dbus_nm;
/// let manager = dbus_nm::new();
/// let connections = connection::list(&manager).unwrap();
/// connection::enable(&manager, &connections[0], 10).unwrap();
/// ```
pub fn enable(manager: &DBusNetworkManager,
              connection: &Connection,
              time_out: i32)
              -> Result<ConnectionState, String> {
    let state = try!(get_connection_state(manager, connection));

    match state {
        ConnectionState::Activated => Ok(ConnectionState::Activated),
        ConnectionState::Activating => {
            wait(manager, connection, time_out, ConnectionState::Activated)
        }
        ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
        _ => {
            try!(manager.activate_connection(&connection.path));

            wait(manager, connection, time_out, ConnectionState::Activated)
        }
    }
}

/// Disables a Network Manager connection.
///
/// # Examples
///
/// ```no_run
/// use network_manager::connection;
/// use network_manager::dbus_nm;
/// let manager = dbus_nm::new();
/// let connections = connection::list(&manager).unwrap();
/// connection::disable(&manager, &connections[0], 10).unwrap();
/// ```
pub fn disable(manager: &DBusNetworkManager,
               connection: &Connection,
               time_out: i32)
               -> Result<ConnectionState, String> {
    let state = try!(get_connection_state(manager, connection));

    match state {
        ConnectionState::Deactivated => Ok(ConnectionState::Deactivated),
        ConnectionState::Deactivating => {
            wait(manager, connection, time_out, ConnectionState::Deactivated)
        }
        ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
        _ => {
            let active_path_option = try!(get_connection_active_path(manager, connection));

            if let Some(active_path) = active_path_option {
                try!(manager.deactivate_connection(&active_path));

                wait(manager, connection, time_out, ConnectionState::Deactivated)
            } else {
                Ok(ConnectionState::Deactivated)
            }
        }
    }
}

#[test]
fn test_enable_disable_functions() {
    let manager = dbus_nm::new();

    let connections = list(&manager).unwrap();

    // set enviorment variable $TEST_WIFI_SSID with the wifi's SSID that you want to test
    // e.g.  export TEST_WIFI_SSID="Resin.io Wifi"
    let wifiEnvVar = "TEST_WIFI_SSID";
    let connection = match env::var(wifiEnvVar) {
        Ok(ssid) => {
            connections
                .iter()
                .filter(|c| c.settings.ssid == ssid)
                .nth(0)
                .unwrap()
                .clone()
        }
        Err(e) => {
            panic!("couldn't retrieve enviorment variable {}: {}",
                   wifiEnvVar,
                   e)
        }
    };

    let state = get_connection_state(&manager, &connection).unwrap();

    if state == ConnectionState::Activated {
        let state = disable(&manager, &connection, 10).unwrap();
        assert_eq!(ConnectionState::Deactivated, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        let state = enable(&manager, &connection, 10).unwrap();
        assert_eq!(ConnectionState::Activated, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));
    } else {
        let state = enable(&manager, &connection, 10).unwrap();
        assert_eq!(ConnectionState::Activated, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        let state = disable(&manager, &connection, 10).unwrap();
        assert_eq!(ConnectionState::Deactivated, state);

        ::std::thread::sleep(::std::time::Duration::from_secs(5));
    }
}

fn get_connection(manager: &DBusNetworkManager, path: &String) -> Result<Connection, String> {
    let settings = try!(manager.get_connection_settings(path));

    Ok(Connection {
           path: path.clone(),
           settings: settings,
       })
}

pub fn get_connection_state(manager: &DBusNetworkManager,
                            connection: &Connection)
                            -> Result<ConnectionState, String> {
    let active_path_option = try!(get_connection_active_path(manager, connection));

    if let Some(active_path) = active_path_option {
        let state = try!(manager.get_connection_state(&active_path));

        Ok(state)
    } else {
        Ok(ConnectionState::Deactivated)
    }
}

fn get_connection_active_path(manager: &DBusNetworkManager,
                              connection: &Connection)
                              -> Result<Option<String>, String> {
    let active_paths = try!(manager.get_active_connections());

    for active_path in active_paths {
        if let Some(settings_path) = manager.get_active_connection_path(&active_path) {
            if connection.path == settings_path {
                return Ok(Some(active_path));
            }
        }
    }

    Ok(None)
}

fn wait(manager: &DBusNetworkManager,
        connection: &Connection,
        time_out: i32,
        target_state: ConnectionState)
        -> Result<ConnectionState, String> {
    if time_out == 0 {
        return get_connection_state(manager, connection);
    }

    let mut total_time = 0;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let state = try!(get_connection_state(manager, connection));

        total_time += 1;

        if state == target_state || total_time >= time_out {
            return Ok(state);
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Connection {
    pub path: String,
    pub settings: ConnectionSettings,
}

impl Default for Connection {
    fn default() -> Connection {
        Connection {
            path: "".to_string(),
            settings: ConnectionSettings::default(),
        }
    }
}

impl Ord for Connection {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        i32::from(self).cmp(&i32::from(other))
    }
}

impl PartialOrd for Connection {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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


enum_from_primitive!{
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConnectionState {
    Unknown = 0,
    Activating = 1,
    Activated = 2,
    Deactivating = 3,
    Deactivated = 4,
}
}

impl From<u32> for ConnectionState {
    fn from(val: u32) -> ConnectionState {
        ConnectionState::from_u32(val).expect("Invalid ConnectionState enum value")
    }
}

impl From<ConnectionState> for u32 {
    fn from(val: ConnectionState) -> u32 {
        val as u32
    }
}
