extern crate dbus;
extern crate enum_primitive;

use std::str::FromStr;
use enum_primitive::FromPrimitive;

pub const NM_SERVICE_MANAGER: &'static str = "org.freedesktop.NetworkManager";

pub const NM_SERVICE_PATH: &'static str = "/org/freedesktop/NetworkManager";
pub const NM_SETTINGS_PATH: &'static str = "/org/freedesktop/NetworkManager/Settings";

pub const NM_SERVICE_INTERFACE: &'static str = "org.freedesktop.NetworkManager";
pub const NM_SETTINGS_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings";
pub const NM_CONNECTION_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings.\
                                                   Connection";
pub const NM_ACTIVE_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Connection.Active";
pub const NM_DEVICE_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device";

/// Gets the Network Manager status.
///
/// # Examples
///
/// ```no_run
/// use network_manager::general;
/// let status = general::status().unwrap();
/// println!("{:?}", status);
/// ```
pub fn status() -> Result<Status, String> {
    let mut status: Status = Default::default();

    let message = dbus_message!(NM_SERVICE_MANAGER,
                                NM_SERVICE_PATH,
                                NM_SERVICE_INTERFACE,
                                "state");
    let response = dbus_connect!(message);
    let val: u32 = response.get1().unwrap();
    status.state = NetworkManagerState::from(val);

    let message = dbus_message!(NM_SERVICE_MANAGER,
                                NM_SERVICE_PATH,
                                NM_SERVICE_INTERFACE,
                                "CheckConnectivity");
    let response = dbus_connect!(message);
    let val: u32 = response.get1().unwrap();
    status.connectivity = Connectivity::from(val);

    status.wireless_network_enabled = dbus_property!(NM_SERVICE_MANAGER,
                                                     NM_SERVICE_PATH,
                                                     NM_SERVICE_INTERFACE,
                                                     "WirelessEnabled")
            .unwrap()
            .inner()
            .unwrap();

    status.networking_enabled = dbus_property!(NM_SERVICE_MANAGER,
                                               NM_SERVICE_PATH,
                                               NM_SERVICE_INTERFACE,
                                               "NetworkingEnabled")
            .unwrap()
            .inner()
            .unwrap();

    Ok(status)
}

// This should be implemented as a trait in the dbus crate
pub fn dbus_path_to_string(path: dbus::Path) -> String {
    path.as_cstr()
        .to_str()
        .unwrap()
        .to_string()
}

#[derive(Debug)]
pub struct Status {
    state: NetworkManagerState,
    connectivity: Connectivity,
    wireless_network_enabled: bool,
    networking_enabled: bool, // Any type of networking is enabled (Doc: https://goo.gl/P92Xtn)
}

impl Default for Status {
    fn default() -> Status {
        Status {
            state: NetworkManagerState::Unknown,
            connectivity: Connectivity::Unknown,
            wireless_network_enabled: false,
            networking_enabled: false,
        }
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

#[derive(Debug)]
pub enum Security {
    None,
    WEP,
    WPA1,
    WPA2,
}
