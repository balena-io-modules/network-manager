extern crate dbus;
extern crate enum_primitive;

use enum_primitive::FromPrimitive;

use manager::NetworkManager;

/// Gets the Network Manager status.
///
/// # Examples
///
/// ```no_run
/// use network_manager::status;
/// use network_manager::manager;
/// let manager = manager::new();
/// let status = status::status(&manager).unwrap();
/// println!("{:?}", status);
/// ```
pub fn status(manager: &NetworkManager) -> Result<Status, String> {
    let mut status: Status = Default::default();

    status.state = try!(manager.get_state());
    status.connectivity = try!(manager.check_connectivity());
    status.wireless_network_enabled = try!(manager.is_wireless_enabled());
    status.networking_enabled = try!(manager.is_networking_enabled());

    Ok(status)
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
