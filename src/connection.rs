use std::rc::Rc;
use std::fmt;
use std::mem;
use std::str;
use std::ops::Deref;

use ascii::AsAsciiStr;

use dbus_nm::DBusNetworkManager;

use wifi::Security;
use device::{Device, get_active_connection_devices};


#[derive(Clone)]
pub struct Connection {
    dbus_manager: Rc<DBusNetworkManager>,
    path: String,
    settings: ConnectionSettings,
}

impl Connection {
    fn init(dbus_manager: &Rc<DBusNetworkManager>, path: &str) -> Result<Self, String> {
        let settings = dbus_manager.get_connection_settings(path)?;

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
        let active_path_option = get_connection_active_path(&self.dbus_manager, &self.path)?;

        if let Some(active_path) = active_path_option {
            let state = self.dbus_manager.get_connection_state(&active_path)?;

            Ok(state)
        } else {
            Ok(ConnectionState::Deactivated)
        }
    }

    pub fn delete(&self) -> Result<(), String> {
        self.dbus_manager.delete_connection(&self.path)
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
        let state = self.get_state()?;

        match state {
            ConnectionState::Activated => Ok(ConnectionState::Activated),
            ConnectionState::Activating => {
                wait(self,
                     ConnectionState::Activated,
                     self.dbus_manager.method_timeout())
            }
            ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
            _ => {
                self.dbus_manager.activate_connection(&self.path)?;

                wait(self,
                     ConnectionState::Activated,
                     self.dbus_manager.method_timeout())
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
        let state = self.get_state()?;

        match state {
            ConnectionState::Deactivated => Ok(ConnectionState::Deactivated),
            ConnectionState::Deactivating => {
                wait(self,
                     ConnectionState::Deactivated,
                     self.dbus_manager.method_timeout())
            }
            ConnectionState::Unknown => Err("Unable to get connection state".to_string()),
            _ => {
                let active_path_option = get_connection_active_path(&self.dbus_manager,
                                                                    &self.path)?;

                if let Some(active_path) = active_path_option {
                    self.dbus_manager.deactivate_connection(&active_path)?;

                    wait(self,
                         ConnectionState::Deactivated,
                         self.dbus_manager.method_timeout())
                } else {
                    Ok(ConnectionState::Deactivated)
                }
            }
        }
    }

    pub fn get_devices(&self) -> Result<Vec<Device>, String> {
        let active_path_option = get_connection_active_path(&self.dbus_manager, &self.path)?;

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
    pub ssid: Ssid,
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


pub fn get_connections(dbus_manager: &Rc<DBusNetworkManager>) -> Result<Vec<Connection>, String> {
    let paths = dbus_manager.list_connections()?;

    let mut connections = Vec::with_capacity(paths.len());

    for path in &paths {
        connections.push(Connection::init(dbus_manager, path)?)
    }

    connections.sort();

    Ok(connections)
}


pub fn get_active_connections(dbus_manager: &Rc<DBusNetworkManager>)
                              -> Result<Vec<Connection>, String> {
    let active_paths = dbus_manager.get_active_connections()?;

    let mut connections = Vec::with_capacity(active_paths.len());

    for active_path in active_paths {
        if let Some(path) = dbus_manager.get_active_connection_path(&active_path) {
            connections.push(Connection::init(dbus_manager, &path)?)
        }
    }

    connections.sort();

    Ok(connections)
}


pub fn connect_to_access_point<P>(dbus_manager: &Rc<DBusNetworkManager>,
                                  device_path: &str,
                                  access_point_path: &str,
                                  ssid: &SsidSlice,
                                  security: &Security,
                                  password: &P)
                                  -> Result<(Connection, ConnectionState), String>
    where P: AsAsciiStr + ?Sized
{
    let (path, _) =
        dbus_manager
            .connect_to_access_point(device_path, access_point_path, ssid, security, password)?;

    let connection = Connection::init(dbus_manager, &path)?;

    let state = wait(&connection,
                     ConnectionState::Activated,
                     dbus_manager.method_timeout())?;

    Ok((connection, state))
}

pub fn create_hotspot<S, P>(dbus_manager: &Rc<DBusNetworkManager>,
                            device_path: &str,
                            interface: &str,
                            ssid: &S,
                            password: Option<&P>)
                            -> Result<(Connection, ConnectionState), String>
    where S: AsSsidSlice + ?Sized,
          P: AsAsciiStr + ?Sized
{
    let (path, _) = dbus_manager
        .create_hotspot(device_path, interface, ssid, password)?;

    let connection = Connection::init(dbus_manager, &path)?;

    let state = wait(&connection,
                     ConnectionState::Activated,
                     dbus_manager.method_timeout())?;

    Ok((connection, state))
}

fn get_connection_active_path(dbus_manager: &DBusNetworkManager,
                              connection_path: &str)
                              -> Result<Option<String>, String> {
    let active_paths = dbus_manager.get_active_connections()?;

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

        let state = connection.get_state()?;

        total_time += 1;

        if state == target_state || total_time >= timeout {
            return Ok(state);
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SsidSlice {
    slice: [u8],
}

pub trait AsSsidSlice {
    fn as_ssid_slice(&self) -> Result<&SsidSlice, String>;
}

impl SsidSlice {
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.slice)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { mem::transmute(&self.slice) }
    }
}

impl AsSsidSlice for [u8] {
    fn as_ssid_slice(&self) -> Result<&SsidSlice, String> {
        if self.len() > 32 {
            Err(format!("SSID length should not exceed 32: {} len", self.len()))
        } else {
            Ok(unsafe { mem::transmute(self) })
        }
    }
}

impl AsSsidSlice for str {
    fn as_ssid_slice(&self) -> Result<&SsidSlice, String> {
        self.as_bytes().as_ssid_slice()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ssid {
    vec: Vec<u8>,
}

impl Ssid {
    pub fn new() -> Self {
        Ssid { vec: Vec::new() }
    }

    pub fn from_bytes<B>(bytes: B) -> Result<Self, String>
        where B: Into<Vec<u8>> + AsRef<[u8]>
    {
        match bytes.as_ref().as_ssid_slice() {
            Ok(_) => Ok(unsafe { Ssid::from_bytes_unchecked(bytes) }),
            Err(e) => Err(e),
        }
    }

    unsafe fn from_bytes_unchecked<B>(bytes: B) -> Self
        where B: Into<Vec<u8>>
    {
        Ssid { vec: mem::transmute(bytes.into()) }
    }
}

pub trait IntoSsid: Sized {
    fn into_ssid(self) -> Result<Ssid, String>;
}

impl Deref for Ssid {
    type Target = SsidSlice;

    #[inline]
    fn deref(&self) -> &SsidSlice {
        unsafe { mem::transmute(&self.vec[..]) }
    }
}

#[cfg(test)]
mod tests {
    use super::super::NetworkManager;
    use super::*;

    #[test]
    fn test_ssid_as_str() {
        let ssid_u8 = vec![0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let ssid = Ssid::from_bytes(ssid_u8.clone()).unwrap();
        let ssid_slice = &ssid as &SsidSlice;
        let ssid_str = ssid_slice.as_str().unwrap();
        assert_eq!("hello", ssid_str);
    }

    #[test]
    fn test_ssid() {
        let ssid_from_string = Ssid::from_bytes("hello".to_string()).unwrap();
        let ssid_u8 = vec![0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let ssid_from_u8 = Ssid::from_bytes(ssid_u8).unwrap();
        assert_eq!(ssid_from_string, ssid_from_u8);
    }

    #[test]
    fn test_ssid_slice() {
        let slice_from_str = "hello".as_ssid_slice().unwrap();
        let ssid_u8 = [0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let slice_from_u8 = (&ssid_u8 as &[u8]).as_ssid_slice().unwrap();
        assert_eq!(slice_from_str, slice_from_u8);
    }

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
                    .filter(|c| c.settings().ssid.as_str().unwrap() == ssid)
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
