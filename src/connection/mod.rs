extern crate dbus;

use std::str;
use general::{Interface, Security, ConnectionState};

const NM_SERVICE: &'static str = "org.freedesktop.NetworkManager";
const NM_SETTINGS_PATH: &'static str = "/org/freedesktop/NetworkManager/Settings";
const NM_SETTINGS_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings";
const NM_CONNECTION_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings.Connection";

/// Get a list of Network Manager connections.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// println!("{:?}", connections);
/// ```
pub fn list() -> Result<Vec<Connection>, String> {
    let message = dbus_message!(NM_SERVICE,
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

fn get(path: dbus::Path) -> Result<Connection, String> {
    let mut connection =
        Connection { path: path.as_cstr().to_str().unwrap().to_string(), ..Default::default() };

    let message = dbus_message!(NM_SERVICE,
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
