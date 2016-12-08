use general::{Interface, Security, ConnectionState};

/// Get a list of Network Manager connections.
///
/// # Examples
///
/// ```
/// let connections = network_manager::connection::list().unwrap();
/// println!("{:?}", connections);
/// ```
pub fn list() -> Result<Vec<Connection>, String> {
    // Get list of connections

    let connection1 = Connection {
        name: "resin_io".to_owned(),
        ssid: "resin_io".to_owned(),
        uuid: "3c8e6e8b-b895-4b07-97a5-bbc192c3b436".to_owned(),
        device: "wlp4s0".to_owned(),
        path: "/org/freedesktop/NetworkManager/ActiveConnection/187".to_owned(),
        interface: Interface::WiFi,
        security: Security::WPA2,
        state: ConnectionState::Activated,
    };

    let connection2 = Connection {
        name: "docker0".to_owned(),
        ssid: String::new(),
        uuid: "3c8e6e8b-b895-4b07-97a5-bbc192c3b436".to_owned(),
        device: "docker0".to_owned(),
        path: "/org/freedesktop/NetworkManager/ActiveConnection/180".to_owned(),
        interface: Interface::Bridge,
        security: Security::None,
        state: ConnectionState::Deactivated,
    };

    Ok(vec![connection1, connection2])
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

    let new_connection = Connection {
        name: "resin_io".to_owned(),
        ssid: "resin_io".to_owned(),
        uuid: "3c8e6e8b-b895-4b07-97a5-bbc192c3b436".to_owned(),
        device: "wlp4s0".to_owned(),
        path: "/org/freedesktop/NetworkManager/ActiveConnection/187".to_owned(),
        interface: Interface::WiFi,
        security: Security::WPA2,
        state: ConnectionState::Activated,
    };

    Ok(new_connection)
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

#[derive(Debug)]
pub struct Connection {
    name: String,
    ssid: String,
    uuid: String,
    device: String,
    path: String,
    interface: Interface,
    security: Security,
    state: ConnectionState,
}
