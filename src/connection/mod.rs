extern crate dbus;

use errors::*;
use general::{NM_SERVICE_MANAGER, NM_SETTINGS_PATH, NM_SETTINGS_INTERFACE};
use enum_primitive::FromPrimitive;
// use std::str::FromStr;
// use std::time::Duration;
use std::collections::HashMap;
use self::dbus::{Connection, Message, MessageItem, BusType, Path};
use self::dbus::arg::{Dict, Variant};
// use self::futures::Future;
// use self::futures_cpupool::CpuPool;
// use self::tokio_timer::Timer;
// use general::ConnectionState;

/// // Get a list of Network Manager connections sorted by path.
/// //
/// // # Examples
/// //
/// // ```no_run
/// // use network_manager::connection;
/// // let connections = connection::list().unwrap();
/// // println!("{:?}", connections);
/// // ```
/// pub fn list() -> Result<Vec<Connection>, String> {
///     let message = dbus_message!(NM_SERVICE_MANAGER,
///                                 NM_SETTINGS_PATH,
///                                 NM_SETTINGS_INTERFACE,
///                                 "ListConnections");
///     let response = dbus_connect!(message);
///     let paths: dbus::arg::Array<dbus::Path, _> = response.get1().unwrap();
///     let mut connections: Vec<_> = paths.map(|p| get_connection(p).unwrap())
///         .collect();
///     connections.sort_by(|a, b| a.cmp(b));
///
///     Ok(connections)
/// }
///
/// #[test]
/// fn test_list_function() {
///     let connections = list().unwrap();
///     assert!(connections.len() > 0);
///     for (index, val) in connections.iter().enumerate() {
///         assert_ne!(Connection { ..Default::default() }, val.clone());
///         assert_eq!(index as i32, i32::from(val));
///     }
/// }
///
///

/// Creates a Network Manager connection.
///
/// # Examples
///
/// ```
/// ```
pub fn create(ssid: &str, password: &str) -> Result<Settings> {
    let mut connection = HashMap::new();
    connection.insert("id", Variant(MessageItem::from(ssid)));
    connection.insert("type", Variant(MessageItem::from("802-11-wireless")));

    let mut wireless = HashMap::new();
    wireless.insert("ssid",
                    Variant(try!(MessageItem::new_array(ssid.to_string()
                            .into_bytes()
                            .iter()
                            .map(|&c| MessageItem::from(c))
                            .collect())
                        .map_err(Error::Array))));

    let mut wireless_security = HashMap::new();
    wireless_security.insert("auth-alg", Variant(MessageItem::from("open")));
    wireless_security.insert("key-mgmt", Variant(MessageItem::from("wpa-psk")));
    wireless_security.insert("psk", Variant(MessageItem::from(password)));

    let mut settings = HashMap::new();
    settings.insert("connection", Dict::new(connection));
    settings.insert("802-11-wireless", Dict::new(wireless));
    settings.insert("802-11-wireless-security", Dict::new(wireless_security));

    let message = try!(Message::new_method_call(NM_SERVICE_MANAGER,
                                                NM_SETTINGS_PATH,
                                                NM_SETTINGS_INTERFACE,
                                                "AddConnection")
            .map_err(Error::Message))
        .append1(Dict::new(settings));

    let connection = try!(Connection::get_private(BusType::System).map_err(Error::Connection));

    let response = try!(connection.send_with_reply_and_block(message, 2000)
        .map_err(Error::Connection));

    let path = try!(response.get1::<Path>().ok_or(Error::NotFound));

    get(path)
}

/// // Deletes a Network Manager connection.
/// //
/// // # Examples
/// //
/// // ```
/// // let connections = network_manager::connection::list().unwrap();
/// // let connection = &connections[0];
/// // network_manager::connection::delete(connection).unwrap();
/// // ```
/// pub fn delete(connection: Connection) -> Result<(), String> {
///     let message = dbus_message!(NM_SERVICE_MANAGER,
///                                 connection.path,
///                                 NM_CONNECTION_INTERFACE,
///                                 "Delete");
///     dbus_connect!(message);
///
///     Ok(())
/// }

/// // Enables a Network Manager connection.
/// //
/// // # Examples
/// //
/// // ```no_run
/// // use network_manager::connection;
/// // let connections = connection::list().unwrap();
/// // let mut connection = connections[0].clone();
/// // connection::enable(&mut connection, 10).unwrap();
/// // println!("{:?}", connection.state);
/// // ```
/// pub fn enable(connection: &mut Connection, time_out: i32) -> Result<(), String> {
///     update_state(connection).expect("Unable to get connection state");
///     match connection.state {
///         ConnectionState::Activated => Ok(()),
///         ConnectionState::Activating => wait(connection, time_out, ConnectionState::Activated),
///         ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
///         _ => {
///             let mut message = dbus_message!(NM_SERVICE_MANAGER,
///                                             NM_SERVICE_PATH,
///                                             NM_SERVICE_INTERFACE,
///                                             "ActivateConnection");
///             message.append_items(&[
///                            dbus::MessageItem::ObjectPath(connection.path.to_string().into()),
///                            dbus::MessageItem::ObjectPath("/".into()),
///                            dbus::MessageItem::ObjectPath("/".into())]);
///             dbus_connect!(message);
///
///             wait(connection, time_out, ConnectionState::Activated)
///         }
///     }
/// }

/// // Disables a Network Manager connection.
/// //
/// // # Examples
/// //
/// // ```no_run
/// // use network_manager::connection;
/// // let connections = connection::list().unwrap();
/// // let mut connection = connections[0].clone();
/// // connection::disable(&mut connection, 10).unwrap();
/// // println!("{:?}", connection.state);
/// // ```
/// pub fn disable(connection: &mut Connection, time_out: i32) -> Result<(), String> {
///     update_state(connection).expect("Unable to get connection state");
///     match connection.state {
///         ConnectionState::Deactivated => Ok(()),
///         ConnectionState::Deactivating => wait(connection, time_out, ConnectionState::Deactivated),
///         ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
///         _ => {
///             let mut message = dbus_message!(NM_SERVICE_MANAGER,
///                                             NM_SERVICE_PATH,
///                                             NM_SERVICE_INTERFACE,
///                                             "DeactivateConnection");
///             message.append_items(&[dbus::MessageItem::ObjectPath(connection.active_path
///                                        .to_string()
///                                        .into())]);
///             dbus_connect!(message);
///
///             wait(connection, time_out, ConnectionState::Deactivated)
///         }
///     }
/// }
///
/// #[test]
/// fn test_enable_disable_functions() {
///     let connections = list().unwrap();
///
///     // Note - replace "TP-LINK_2.4GHz_9BDD8F" with one of your configured connections to test
///     let mut connection =
///         connections.iter().filter(|c| c.ssid == "TP-LINK_2.4GHz_9BDD8F").nth(0).unwrap().clone();
///
///     assert!(connection.state == ConnectionState::Activated ||
///             connection.state == ConnectionState::Deactivated);
///
///     match connection.state {
///         ConnectionState::Activated => {
///             disable(&mut connection, 10).unwrap();
///             assert_eq!(ConnectionState::Deactivated, connection.state);
///
///             enable(&mut connection, 10).unwrap();
///             assert_eq!(ConnectionState::Activated, connection.state);
///         }
///         ConnectionState::Deactivated => {
///             enable(&mut connection, 10).unwrap();
///             assert_eq!(ConnectionState::Activated, connection.state);
///
///             disable(&mut connection, 10).unwrap();
///             assert_eq!(ConnectionState::Deactivated, connection.state);
///         }
///         _ => (),
///     }
/// }
///
fn get(path: dbus::Path) -> Result<Settings> {
    let mut settings = Settings { path: path, ..Default::default() };

    return Ok(settings);

    // let message = dbus_message!(NM_SERVICE_MANAGER,
    //                             connection.path.clone(),
    //                             NM_CONNECTION_INTERFACE,
    //                             "GetSettings");
    // let response = dbus_connect!(message);
    // let dictionary: dbus::arg::Dict<&str,
    //                                 dbus::arg::Dict<&str, dbus::arg::Variant<dbus::arg::Iter>, _>,
    //                                 _> = response.get1().unwrap();
    //
    // for (_, v1) in dictionary {
    //     for (k2, v2) in v1 {
    //         match k2 {
    //             "id" => {
    //                 connection.id = v2.0.clone().get::<&str>().unwrap().to_string();
    //             }
    //             "uuid" => {
    //                 connection.uuid = v2.0.clone().get::<&str>().unwrap().to_string();
    //             }
    //             "ssid" => {
    //                 connection.ssid = std::str::from_utf8(&v2.0
    //                         .clone()
    //                         .get::<dbus::arg::Array<u8, _>>()
    //                         .unwrap()
    //                         .collect::<Vec<u8>>())
    //                     .unwrap()
    //                     .to_string();
    //             }
    //             _ => (),
    //         }
    //     }
    // }
    //
    // update_state(&mut connection).unwrap();
    //
    // Ok(connection)
}
///
/// fn update_state(connection: &mut Connection) -> Result<(), String> {
///     let active_paths: Vec<String> = dbus_property!(NM_SERVICE_MANAGER,
///                                                    NM_SERVICE_PATH,
///                                                    NM_SERVICE_INTERFACE,
///                                                    "ActiveConnections")
///         .unwrap()
///         .inner::<&Vec<dbus::MessageItem>>()
///         .unwrap()
///         .iter()
///         .map(|p| dbus_path_to_string(p.inner::<&dbus::Path>().unwrap().to_owned()))
///         .collect();
///
///     let settings_paths = active_paths.iter().map(|p| {
///         dbus_path_to_string(dbus_property!(NM_SERVICE_MANAGER,
///                                            p,
///                                            NM_ACTIVE_INTERFACE,
///                                            "Connection")
///             .unwrap()
///             .inner::<&dbus::Path>()
///             .unwrap()
///             .to_owned())
///     });
///
///     connection.active_path = "".to_string();
///     connection.state = ConnectionState::Deactivated;
///
///     for (active_path, settings_path) in active_paths.iter().zip(settings_paths) {
///         if connection.path == settings_path {
///             connection.active_path = active_path.to_owned();
///
///             let result = dbus_property!(NM_SERVICE_MANAGER,
///                                         connection.active_path.clone(),
///                                         NM_ACTIVE_INTERFACE,
///                                         "State");
///             if let Ok(val) = result {
///                 connection.state = ConnectionState::from(val.inner::<u32>().unwrap())
///             }
///
///             break;
///         }
///     }
///
///     Ok(())
/// }
///
/// fn wait(connection: &mut Connection,
///         time_out: i32,
///         target_state: ConnectionState)
///         -> Result<(), String> {
///     if time_out == 0 {
///         return Ok(());
///     }
///
///     let mut total_time = 0;
///     while total_time < time_out {
///         update_state(connection).unwrap();
///         if connection.state == target_state {
///             return Ok(());
///         }
///         std::thread::sleep(std::time::Duration::from_secs(1));
///         total_time += 1;
///     }
///
///     Err("service timed out".to_string())
/// }

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

enum_from_primitive!{
//#[derive(Debug, Clone, Eq, PartialEq)]
    pub enum State {
        Unknown = 0,
        Activating = 1,
        Activated = 2,
        Deactivating = 3,
        Deactivated = 4,
    }
}

impl From<u32> for State {
    fn from(val: u32) -> State {
        State::from_u32(val).map_err(Error::NotFound);
    }
}

impl From<State> for u32 {
    fn from(val: State) -> u32 {
        val as u32
    }
}

// #[derive(Debug, Clone, Eq, PartialEq)]
pub struct Settings {
    pub path: Path,
    pub active_path: Path<'m>,
    pub id: String,
    pub ssid: String,
    pub state: State,
}

impl Default for Connection {
    fn default() -> Connection {
        Connection {
            path: Path::from(""),
            active_path: Path::from(""),
            id: "".to_string(),
            ssid: "".to_string(),
            state: State::Unknown,
        }
    }
}
//
// impl Ord for Connection {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         i32::from(self).cmp(&i32::from(other))
//     }
// }
//
// impl PartialOrd for Connection {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
//
// impl<'a> From<&'a Connection> for i32 {
//     fn from(val: &Connection) -> i32 {
//         val.clone().path.rsplit('/').nth(0).unwrap().parse::<i32>().unwrap()
//     }
// }
