use std::str::FromStr;
extern crate dbus;
extern crate enum_primitive;
use enum_primitive::FromPrimitive;

const NM_SERVICE: &'static str = "org.freedesktop.NetworkManager";
const NM_PATH: &'static str = "/org/freedesktop/NetworkManager";
const NM_INTERFACE: &'static str = "org.freedesktop.NetworkManager";


/// Gets the Network Manager status.
///
/// # Examples
///
/// ```
/// let status = network_manager::general::status().unwrap();
/// println!("{:?}", status);
/// ```
pub fn status() -> Result<Status, String> {
    // Get network manager status

    let mut status = Status {
        state: NetworkManagerState::Unknown,
        connectivity: Connectivity::Unknown,
        wireless_enabled: false,
        networking_enabled: false,
    };

    let message = dbus_message!(NM_SERVICE, NM_PATH, NM_INTERFACE, "state");
    let response = dbus_connect!(message).unwrap();
    let val: u32 = response.get1().unwrap();
    status.state = NetworkManagerState::from(val);

    let message = dbus_message!(NM_SERVICE, NM_PATH, NM_INTERFACE, "CheckConnectivity");
    let response = dbus_connect!(message).unwrap();
    let val: u32 = response.get1().unwrap();
    status.connectivity = Connectivity::from(val);

    let connection = dbus::Connection::get_private(dbus::BusType::System).unwrap();
    status.networking_enabled =
        dbus::Props::new(&connection, NM_SERVICE, NM_PATH, NM_INTERFACE, 2000)
            .get("NetworkingEnabled")
            .unwrap()
            .inner()
            .unwrap();
    status.wireless_enabled =
        dbus::Props::new(&connection, NM_SERVICE, NM_PATH, NM_INTERFACE, 2000)
            .get("WirelessEnabled")
            .unwrap()
            .inner()
            .unwrap();

    Ok(status)
}



impl From<u32> for NetworkManagerState {
    fn from(val: u32) -> NetworkManagerState {
        NetworkManagerState::from_u32(val).expect("passed Value does not match an enum value!")
    }
}
impl From<NetworkManagerState> for u32 {
    fn from(val: NetworkManagerState) -> u32 {
        val as u32
    }
}

impl From<u32> for Connectivity {
    fn from(val: u32) -> Connectivity {
        Connectivity::from_u32(val).expect("passed Value does not match an enum value!")
    }
}
impl From<Connectivity> for u32 {
    fn from(val: Connectivity) -> u32 {
        val as u32
    }
}

#[derive(Debug)]
pub struct Status {
    state: NetworkManagerState,
    connectivity: Connectivity,
    wireless_enabled: bool,
    networking_enabled: bool,
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

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ServiceState {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}

impl FromStr for ServiceState {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(ServiceState::Active),
            "reloading" => Ok(ServiceState::Reloading),
            "inactive" => Ok(ServiceState::Inactive),
            "failed" => Ok(ServiceState::Failed),
            "activating" => Ok(ServiceState::Activating),
            "deactivating" => Ok(ServiceState::Deactivating),
            _ => Err("invalid service state value"),
        }
    }
}

#[derive(Debug)]
pub enum ConnectionState {
    Unknown,
    Activating,
    Activated,
    Deactivating,
    Deactivated,
}

#[derive(Debug)]
pub enum DeviceState {
    Unknown,
    Unmanaged,
    Unavailable,
    Disconnected,
    Activated,
    Deactivating,
    Failed,
}

enum_from_primitive!{
#[derive(Debug, PartialEq)]
pub enum Connectivity {
    Unknown = 1,
    None = 2,
    Portal = 3,
    Limited = 4,
    Full = 5,
}
}

#[derive(Debug)]
pub enum Security {
    None,
    WEP,
    WPA1,
    WPA2,
}

#[derive(Debug)]
pub enum Interface {
    Unknown,
    Generic,
    Ethernet,
    WiFi,
    Bridge,
}
