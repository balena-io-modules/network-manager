extern crate dbus;

use std;
use general::*;

/// Get a list of Network Manager connections sorted by path.
///
/// # Examples
///
/// ```no_run
/// use network_manager::connection;
/// let connections = connection::list().unwrap();
/// println!("{:?}", connections);
/// ```
pub fn list() -> Result<Vec<Connection>, String> {
    let message = dbus_message!(NM_SERVICE_MANAGER,
                                NM_SETTINGS_PATH,
                                NM_SETTINGS_INTERFACE,
                                "ListConnections");
    let response = dbus_connect!(message);
    let paths: dbus::arg::Array<dbus::Path, _> = response.get1().unwrap();
    let mut connections: Vec<_> = paths.map(|p| get_connection(p).unwrap())
        .collect();
    connections.sort_by(|a, b| a.cmp(b));

    Ok(connections)
}

#[test]
fn test_list_function() {
    let connections = list().unwrap();
    assert!(connections.len() > 0);
    for (index, val) in connections.iter().enumerate() {
        assert_ne!(Connection { ..Default::default() }, val.clone());
        assert_eq!(index as i32, i32::from(val));
    }
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
    //
    // dbus::arg::Dict<&str,dbus::arg::Dict<&str, dbus::arg::Variant<dbus::arg::Iter>,
    // connection = {
    //         '802-11-wireless': {
    //             ssid: _.invokeMap(ssid, 'charCodeAt')
    //         },
    //         connection: {
    //             id: ssid,
    //             type: '802-11-wireless',
    //         },
    //         '802-11-wireless-security': {
    //             'auth-alg': 'open',
    //             'key-mgmt': 'wpa-psk',
    //             'psk': passphrase,
    //         }
    //     }


    let mut settings = std::collections::HashMap::new();

    settings.insert("connection",
                std::collections::HashMap::<&str, dbus::arg::Variant<_>>::new());
    settings.insert("802-11-wireless",
                std::collections::HashMap::<&str, dbus::arg::Variant<_>>::new());
    settings.insert("802-11-wireless-security",
                std::collections::HashMap::<&str, dbus::arg::Variant<_>>::new());

    settings.get_mut("connection").unwrap().insert("id", dbus::arg::Variant("resin_io"));
    settings.get_mut("connection").unwrap().insert("type", dbus::arg::Variant("802-11-wireless"));

    // settings.get_mut("802-11-wireless").unwrap().insert("ssid", dbus::arg::Variant(vec![10, 20, 30]));

    // let test = dbus::arg::Dict::new(settings);
    println!("{:?}", settings);

    // let mut message = dbus_message!(NM_SERVICE_MANAGER,
    //                                 NM_SETTINGS_PATH,
    //                                 NM_SETTINGS_INTERFACE,
    //                                 "AddConnection");
    // message.append_items(&[
    //
    //
    //                dbus::MessageItem::ObjectPath(connection.path.to_string().into()),
    //                dbus::MessageItem::ObjectPath("/".into()),
    //                dbus::MessageItem::ObjectPath("/".into())]);
    // dbus_connect!(message);

    let connection1 = Connection {
        path: "/org/freedesktop/NetworkManager/ActiveConnection/187".to_string(),
        id: "resin_io".to_string(),
        uuid: "3c8e6e8b-b895-4b07-97a5-bbc192c3b436".to_string(),
        ssid: "resin_io".to_string(),
        active_path: "test".to_string(),
        state: ConnectionState::Deactivated, /* device: "wlp4s0".to_string(),
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
pub fn delete(connection: Connection) -> Result<(), String> {
    let message = dbus_message!(NM_SERVICE_MANAGER,
                                connection.path,
                                NM_CONNECTION_INTERFACE,
                                "Delete");
    dbus_connect!(message);

    Ok(())
}

/// Enables a Network Manager connection.
///
/// # Examples
///
/// ```no_run
/// use network_manager::connection;
/// let connections = connection::list().unwrap();
/// let mut connection = connections[0].clone();
/// connection::enable(&mut connection, 10).unwrap();
/// println!("{:?}", connection.state);
/// ```
pub fn enable(connection: &mut Connection, time_out: i32) -> Result<(), String> {
    update_state(connection).expect("Unable to get connection state");
    match connection.state {
        ConnectionState::Activated => Ok(()),
        ConnectionState::Activating => wait(connection, time_out, ConnectionState::Activated),
        ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
        _ => {
            let mut message = dbus_message!(NM_SERVICE_MANAGER,
                                            NM_SERVICE_PATH,
                                            NM_SERVICE_INTERFACE,
                                            "ActivateConnection");
            message.append_items(&[
                           dbus::MessageItem::ObjectPath(connection.path.to_string().into()),
                           dbus::MessageItem::ObjectPath("/".into()),
                           dbus::MessageItem::ObjectPath("/".into())]);
            dbus_connect!(message);

            wait(connection, time_out, ConnectionState::Activated)
        }
    }
}

/// Disables a Network Manager connection.
///
/// # Examples
///
/// ```no_run
/// use network_manager::connection;
/// let connections = connection::list().unwrap();
/// let mut connection = connections[0].clone();
/// connection::disable(&mut connection, 10).unwrap();
/// println!("{:?}", connection.state);
/// ```
pub fn disable(connection: &mut Connection, time_out: i32) -> Result<(), String> {
    update_state(connection).expect("Unable to get connection state");
    match connection.state {
        ConnectionState::Deactivated => Ok(()),
        ConnectionState::Deactivating => wait(connection, time_out, ConnectionState::Deactivated),
        ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
        _ => {
            let mut message = dbus_message!(NM_SERVICE_MANAGER,
                                            NM_SERVICE_PATH,
                                            NM_SERVICE_INTERFACE,
                                            "DeactivateConnection");
            message.append_items(&[dbus::MessageItem::ObjectPath(connection.active_path
                                       .to_string()
                                       .into())]);
            dbus_connect!(message);

            wait(connection, time_out, ConnectionState::Deactivated)
        }
    }
}

#[test]
fn test_enable_disable_functions() {
    let connections = list().unwrap();

    // Note - replace "TP-LINK_2.4GHz_9BDD8F" with one of your configured connections to test
    let mut connection =
        connections.iter().filter(|c| c.ssid == "TP-LINK_2.4GHz_9BDD8F").nth(0).unwrap().clone();

    assert!(connection.state == ConnectionState::Activated ||
            connection.state == ConnectionState::Deactivated);

    match connection.state {
        ConnectionState::Activated => {
            disable(&mut connection, 10).unwrap();
            assert_eq!(ConnectionState::Deactivated, connection.state);

            enable(&mut connection, 10).unwrap();
            assert_eq!(ConnectionState::Activated, connection.state);
        }
        ConnectionState::Deactivated => {
            enable(&mut connection, 10).unwrap();
            assert_eq!(ConnectionState::Activated, connection.state);

            disable(&mut connection, 10).unwrap();
            assert_eq!(ConnectionState::Deactivated, connection.state);
        }
        _ => (),
    }
}

fn get_connection(path: dbus::Path) -> Result<Connection, String> {
    let mut connection = Connection { path: dbus_path_to_string(path), ..Default::default() };

    let message = dbus_message!(NM_SERVICE_MANAGER,
                                connection.path.clone(),
                                NM_CONNECTION_INTERFACE,
                                "GetSettings");
    let response = dbus_connect!(message);
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
                    connection.ssid = std::str::from_utf8(&v2.0
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

    update_state(&mut connection).unwrap();

    Ok(connection)
}

fn update_state(connection: &mut Connection) -> Result<(), String> {
    let active_paths: Vec<String> = dbus_property!(NM_SERVICE_MANAGER,
                                                   NM_SERVICE_PATH,
                                                   NM_SERVICE_INTERFACE,
                                                   "ActiveConnections")
        .unwrap()
        .inner::<&Vec<dbus::MessageItem>>()
        .unwrap()
        .iter()
        .map(|p| dbus_path_to_string(p.inner::<&dbus::Path>().unwrap().to_owned()))
        .collect();

    let settings_paths = active_paths.iter().map(|p| {
        dbus_path_to_string(dbus_property!(NM_SERVICE_MANAGER,
                                           p,
                                           NM_ACTIVE_INTERFACE,
                                           "Connection")
            .unwrap()
            .inner::<&dbus::Path>()
            .unwrap()
            .to_owned())
    });

    connection.active_path = "".to_string();
    connection.state = ConnectionState::Deactivated;

    for (active_path, settings_path) in active_paths.iter().zip(settings_paths) {
        if connection.path == settings_path {
            connection.active_path = active_path.to_owned();

            let result = dbus_property!(NM_SERVICE_MANAGER,
                                        connection.active_path.clone(),
                                        NM_ACTIVE_INTERFACE,
                                        "State");
            if let Ok(val) = result {
                connection.state = ConnectionState::from(val.inner::<u32>().unwrap())
            }

            break;
        }
    }

    Ok(())
}

fn wait(connection: &mut Connection,
        time_out: i32,
        target_state: ConnectionState)
        -> Result<(), String> {
    if time_out == 0 {
        return Ok(());
    }

    let mut total_time = 0;
    while total_time < time_out {
        update_state(connection).unwrap();
        if connection.state == target_state {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        total_time += 1;
    }

    Err("service timed out".to_string())
}

// // Contains fields needed for wireless connections
// pub struct Settings {
//     pub settings_path: dbus::Path,
//     pub active_path: dbus::Path,
//     pub connection: Connection,
//     pub wireless: Wireless,
//     pub wireless_security: WirelessSecurity,
//     pub ipv4: Ip,
//     pub ipv6: IP,
// }
//
// pub struct Connection {
//     pub id: String,
//     pub permissions: dbus::arg::Array,
//     pub secondaries: dbus::arg::Array,
//     pub timestamp: u64,
//     // Called `type` in the Network Manager spec, renamed to
//     // `interface` because `type` is a reserved word.
//     pub interface: String,
//     pub uuid: String,
// }
//
// // Called `802-11-wireless` in the Network Manager spec, renamed
// // to `Wireless` because type names cannot contain numbers/dashes
// pub struct Wireless {
//     pub mac_address: dbus::arg::Array,
//     pub mac_address_blacklist: dbus::arg::Array,
//     pub mode: String,
//     pub security: String,
//     pub seen_bssids: dbus::arg::Array,
//     pub ssid: dbus::arg::Array,
// }
//
// // Called `802-11-wireless-security` in the Network Manager spec,
// // renamed to `WirelessSecurity` because type names cannot contain numbers/dashes
// pub struct WirelessSecurity {
//     pub auth_alg: String,
//     pub group: dbus::arg::Array,
//     pub key_mgmt: String,
//     pub pairwise: dbus::arg::Array,
//     pub proto: dbus::arg::Array,
// }
//
// pub struct Ip {
//     pub address_data: dbus::arg::Array,
//     pub addresses: dbus::arg::Array,
//     pub dns: dbus::arg::Array,
//     pub dns_search: dbus::arg::Array,
//     pub method: String,
//     pub route_data: dbus::arg::Array,
//     pub routes: dbus::arg::Array,
// }

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Connection {
    pub path: String,
    pub active_path: String,
    pub id: String,
    pub uuid: String,
    pub ssid: String,
    pub state: ConnectionState, /* device: String,
                                 * interface: Interface,
                                 * security: Security, */
}

impl Default for Connection {
    fn default() -> Connection {
        Connection {
            path: "".to_string(),
            active_path: "".to_string(),
            id: "".to_string(),
            uuid: "".to_string(),
            ssid: "".to_string(),
            state: ConnectionState::Unknown,
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
        val.clone().path.rsplit('/').nth(0).unwrap().parse::<i32>().unwrap()
    }
}
