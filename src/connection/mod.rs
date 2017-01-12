extern crate dbus;

use std::str;
use general::*;

/// Get a list of Network Manager connections.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// println!("{:?}", connections);
/// ```
pub fn list() -> Result<Vec<Connection>, String> {
    let message = dbus_message!(NM_SERVICE_MANAGER,
                                NM_SETTINGS_PATH,
                                NM_SETTINGS_INTERFACE,
                                "ListConnections");
    let response = dbus_connect!(message).unwrap();
    let paths: dbus::arg::Array<dbus::Path, _> = response.get1().unwrap();
    let connections = paths.map(|p| get(p).unwrap()).collect::<Vec<Connection>>();

    Ok(connections)
}

/// Creates a Network Manager connection.
///
/// # Examples
///
/// ```
/// let connection = network_manager::connection::create(
///     "resin_io",
///     network_manager::general::Interface::WiFi,
///     network_manager::general::Security::WPA2,
///     "super_secret_passphase"
///     ).unwrap();
/// println!("{:?}", connection);
/// ```
pub fn create(s: &str, i: Interface, sc: Security, p: &str) -> Result<Connection, String> {
    // Create a connection
    // Get the connection
    // Return the connection

    let connection1 = Connection {
        path: "/org/freedesktop/NetworkManager/ActiveConnection/187".to_string(),
        id: "resin_io".to_string(),
        uuid: "3c8e6e8b-b895-4b07-97a5-bbc192c3b436".to_string(),
        ssid: "resin_io".to_string(), /* device: "wlp4s0".to_string(),
                                       * interface: Interface::WiFi,
                                       * security: Security::WPA2,
                                       * state: ConnectionState::Activated, */
    };

    Ok(connection1)
}

/// Deletes a Network Manager connection.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// let connection = &connections[0];
/// network_manager::connection::delete(connection).unwrap();
/// ```
pub fn delete(c: &Connection) -> Result<(), String> {
    // Delete connection

    Ok(())
}

/// Enables a Network Manager connection.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// let connection = &connections[0];
/// let state = network_manager::connection::enable(connection, 10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(c: &Connection, t: i32) -> Result<ConnectionState, String> {
    // Enable connection

    if t != 0 {
        // Wait until the connection state is 'Activated' or
        // until the time has elapsed
    }

    Ok(ConnectionState::Activated)
}

/// Disables a Network Manager connection.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// let connection = &connections[0];
/// let state = network_manager::connection::disable(connection, 10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(c: &Connection, t: i32) -> Result<ConnectionState, String> {
    // Disable connection

    if t != 0 {
        // Wait until the connection state is 'Deactivated' or
        // until the time has elapsed
    }

    Ok(ConnectionState::Deactivated)
}

/// Gets the state of a Network Manager connection.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// let connection = &connections[0];
/// let state = network_manager::connection::state(connection).unwrap();
/// println!("{:?}", state);
/// ```
pub fn state(connection: &Connection) -> Result<ConnectionState, String> {
    // Get a vector containing active connection paths for all the active connections
    let active_paths = dbus_property!(NM_SERVICE_MANAGER,
                                      NM_SERVICE_PATH,
                                      NM_SERVICE_INTERFACE,
                                      "ActiveConnections")
        .inner::<&Vec<dbus::MessageItem>>()
        .unwrap()
        .iter()
        .map(|p| dbus_path_to_string(p.inner::<&dbus::Path>().unwrap().to_owned()))
        .collect::<Vec<_>>();
    // How can we avoid this ^ collect - it gets turned back into an iterator below.

    // Get a vector containing settings paths for all active connections
    let settings_paths = active_paths.iter()
        .map(|p| {
            dbus_path_to_string(dbus_property!(NM_SERVICE_MANAGER,
                                               p,
                                               NM_ACTIVE_INTERFACE,
                                               "Connection")
                .inner::<&dbus::Path>()
                .unwrap()
                .to_owned())
        });

    // Pre-set the state as it won't get changed unless the passed in connection.path is present
    let mut state = ConnectionState::Deactivated;

    // Loop over the active paths and settings paths
    // If the passed in connection.path is equal to the settings path the state is
    // retrieved using the active connection path
    for (active_path, settings_path) in active_paths.iter().zip(settings_paths) {
        if settings_path == connection.path {
            state = ConnectionState::from(dbus_property!(NM_SERVICE_MANAGER,
                                                         active_path,
                                                         NM_ACTIVE_INTERFACE,
                                                         "State")
                .inner::<u32>()
                .unwrap());
            break;
        }
    }

    Ok(state)
}

fn get(path: dbus::Path) -> Result<Connection, String> {
    let mut connection = Connection { path: dbus_path_to_string(path), ..Default::default() };

    let message = dbus_message!(NM_SERVICE_MANAGER,
                                connection.path.clone(),
                                NM_CONNECTION_INTERFACE,
                                "GetSettings");
    let response = dbus_connect!(message).unwrap();
    let dictionary: dbus::arg::Dict<&str,
                                    dbus::arg::Dict<&str, dbus::arg::Variant<dbus::arg::Iter>, _>,
                                    _> = response.get1().unwrap();

    for (_, v1) in dictionary {
        for (k2, v2) in v1 {
            match k2 {
                "id" => {
                    connection.id = v2.0.clone().get::<&str>().unwrap().to_string();
                }
                "uuid" => {
                    connection.uuid = v2.0.clone().get::<&str>().unwrap().to_string();
                }
                "ssid" => {
                    connection.ssid = str::from_utf8(&v2.0
                            .clone()
                            .get::<dbus::arg::Array<u8, _>>()
                            .unwrap()
                            .collect::<Vec<u8>>())
                        .unwrap()
                        .to_string();
                }
                _ => (),
            }
        }
    }

    Ok(connection)
}

#[derive(Default, Debug)]
pub struct Connection {
    path: String,
    id: String,
    uuid: String,
    ssid: String, /* device: String,
                   * interface: Interface,
                   * security: Security,
                   * state: ConnectionState, */
}
