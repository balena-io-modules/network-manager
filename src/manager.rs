use dbus::Connection as DBusConnection;
use dbus::{BusType, Path, ConnPath, Message, MessageItem};
use dbus::arg::{Dict, Variant, Iter, Array, Get, RefArg};
use dbus::stdintf::OrgFreedesktopDBusProperties;

use enum_primitive::FromPrimitive;

use connection::{ConnectionSettings, ConnectionState};
use device::{DeviceType, DeviceState};
use status::{Connectivity, NetworkManagerState};
use wifi::{NM80211ApSecurityFlags, NM80211ApFlags};

pub const NM_SERVICE_MANAGER: &'static str = "org.freedesktop.NetworkManager";

pub const NM_SERVICE_PATH: &'static str = "/org/freedesktop/NetworkManager";
pub const NM_SETTINGS_PATH: &'static str = "/org/freedesktop/NetworkManager/Settings";

pub const NM_SERVICE_INTERFACE: &'static str = "org.freedesktop.NetworkManager";
pub const NM_SETTINGS_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings";
pub const NM_CONNECTION_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings.\
                                                   Connection";
pub const NM_ACTIVE_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Connection.Active";
pub const NM_DEVICE_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device";
pub const NM_WIRELESS_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device.Wireless";
pub const NM_ACCESS_POINT_INTERFACE: &'static str = "org.freedesktop.NetworkManager.AccessPoint";


pub fn new() -> NetworkManager {
    NetworkManager::new()
}


pub struct NetworkManager {
    connection: DBusConnection,
}

impl NetworkManager {
    pub fn new() -> Self {
        let connection = DBusConnection::get_private(BusType::System).unwrap();

        NetworkManager { connection: connection }
    }

    pub fn get_state(&self) -> Result<NetworkManagerState, String> {
        let response = try!(self.call(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "state"));

        let state_u32: u32 = try!(self.extract(&response));

        Ok(NetworkManagerState::from(state_u32))
    }

    pub fn check_connectivity(&self) -> Result<Connectivity, String> {
        let response = try!(self.call(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "CheckConnectivity"));

        let connectivity_u32: u32 = try!(self.extract(&response));

        Ok(Connectivity::from(connectivity_u32))
    }

    pub fn is_wireless_enabled(&self) -> Result<bool, String> {
        self.property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "WirelessEnabled")
    }

    pub fn is_networking_enabled(&self) -> Result<bool, String> {
        self.property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "NetworkingEnabled")
    }

    pub fn list_connections(&self) -> Result<Vec<String>, String> {
        let response = try!(self.call(NM_SETTINGS_PATH, NM_SETTINGS_INTERFACE, "ListConnections"));

        let array: Array<Path, _> = try!(self.extract(&response));

        Ok(array.map(|e| e.to_string()).collect())
    }

    pub fn get_active_connections(&self) -> Result<Vec<String>, String> {
        self.property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "ActiveConnections")
    }

    pub fn get_active_connection_path(&self, path: &String) -> Option<String> {
        match self.property(path, NM_ACTIVE_INTERFACE, "Connection") {
            Ok(p) => Some(p),
            Err(_) => None,
        }
    }

    pub fn get_connection_state(&self, path: &String) -> Result<ConnectionState, String> {
        let state_i64 = match self.property(path, NM_ACTIVE_INTERFACE, "State") {
            Ok(state_i64) => state_i64,
            Err(_) => return Ok(ConnectionState::Unknown),
        };

        match ConnectionState::from_i64(state_i64) {
            Some(state) => Ok(state),
            None => Err(format!("Undefined connection state for {}", path)),
        }
    }

    pub fn get_connection_settings(&self, path: &String) -> Result<ConnectionSettings, String> {
        let response = try!(self.call(&path, NM_CONNECTION_INTERFACE, "GetSettings"));

        let dict: Dict<&str, Dict<&str, Variant<Iter>, _>, _> = try!(self.extract(&response));

        let mut id = String::new();
        let mut uuid = String::new();
        let mut ssid = String::new();

        for (_, v1) in dict {
            for (k2, v2) in v1 {
                match k2 {
                    "id" => {
                        id = try!(extract::<String>(&v2));
                    }
                    "uuid" => {
                        uuid = try!(extract::<String>(&v2));
                    }
                    "ssid" => {
                        ssid = try!(utf8_variant_to_string(&v2));
                    }
                    _ => {}
                }
            }
        }

        Ok(ConnectionSettings {
               id: id,
               uuid: uuid,
               ssid: ssid,
           })
    }

    pub fn get_connection_devices(&self, path: &String) -> Result<Vec<String>, String> {
        self.property(path, NM_ACTIVE_INTERFACE, "Devices")
    }

    pub fn delete_connection(&self, path: &String) -> Result<(), String> {
        try!(self.call(path, NM_CONNECTION_INTERFACE, "Delete"));

        Ok(())
    }

    pub fn activate_connection(&self, path: &String) -> Result<(), String> {
        try!(self.call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "ActivateConnection",
                                 &[MessageItem::ObjectPath(path.to_string().into()),
                                   MessageItem::ObjectPath("/".into()),
                                   MessageItem::ObjectPath("/".into())]));

        Ok(())
    }

    pub fn deactivate_connection(&self, path: &String) -> Result<(), String> {
        try!(self.call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "DeactivateConnection",
                                 &[MessageItem::ObjectPath(path.to_string().into())]));

        Ok(())
    }

    pub fn get_devices(&self) -> Result<Vec<String>, String> {
        self.property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "Devices")
    }

    pub fn get_device_interface(&self, path: &String) -> Result<String, String> {
        self.property(path, NM_DEVICE_INTERFACE, "Interface")
    }

    pub fn get_device_type(&self, path: &String) -> Result<DeviceType, String> {
        self.property(path, NM_DEVICE_INTERFACE, "DeviceType")
    }

    pub fn get_device_state(&self, path: &String) -> Result<DeviceState, String> {
        self.property(path, NM_DEVICE_INTERFACE, "State")
    }

    pub fn is_device_real(&self, path: &String) -> Result<bool, String> {
        self.property(path, NM_DEVICE_INTERFACE, "Real")
    }

    pub fn activate_device(&self, path: &String) -> Result<(), String> {
        try!(self.call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "ActivateConnection",
                                 &[MessageItem::ObjectPath("/".into()),
                                   MessageItem::ObjectPath(path.to_string().into()),
                                   MessageItem::ObjectPath("/".into())]));

        Ok(())
    }

    pub fn disconnect_device(&self, path: &String) -> Result<(), String> {
        try!(self.call(path, NM_DEVICE_INTERFACE, "Disconnect"));

        Ok(())
    }

    pub fn get_device_access_points(&self, path: &String) -> Result<Vec<String>, String> {
        self.property(path, NM_WIRELESS_INTERFACE, "AccessPoints")
    }

    pub fn get_access_point_ssid(&self, path: &String) -> Option<String> {
        if let Ok(ssid_vec) = self.property(path, NM_ACCESS_POINT_INTERFACE, "Ssid") {
            utf8_vec_u8_to_string(ssid_vec).ok()
        } else {
            None
        }
    }

    pub fn get_access_point_strength(&self, path: &String) -> Result<u32, String> {
        self.property(path, NM_ACCESS_POINT_INTERFACE, "Strength")
    }

    pub fn get_access_point_flags(&self, path: &String) -> Result<NM80211ApFlags, String> {
        self.property(path, NM_ACCESS_POINT_INTERFACE, "Flags")
    }

    pub fn get_access_point_wpa_flags(&self,
                                      path: &String)
                                      -> Result<NM80211ApSecurityFlags, String> {
        self.property(path, NM_ACCESS_POINT_INTERFACE, "WpaFlags")
    }

    pub fn get_access_point_rsn_flags(&self,
                                      path: &String)
                                      -> Result<NM80211ApSecurityFlags, String> {
        self.property(path, NM_ACCESS_POINT_INTERFACE, "RsnFlags")
    }

    fn call(&self, path: &str, interface: &str, method: &str) -> Result<Message, String> {
        self.call_with_args(path, interface, method, &[])
    }

    fn call_with_args(&self,
                      path: &str,
                      interface: &str,
                      method: &str,
                      items: &[MessageItem])
                      -> Result<Message, String> {
        let call_error = |details: &str| {
            Err(format!("D-Bus '{}'::'{}' method call failed on '{}': {}",
                        interface,
                        method,
                        path,
                        details))
        };

        match Message::new_method_call(NM_SERVICE_MANAGER, path, interface, method) {
            Ok(mut message) => {
                if items.len() > 0 {
                    message.append_items(items);
                }

                match self.connection.send_with_reply_and_block(message, 2000) {
                    Ok(response) => Ok(response),
                    Err(err) => {
                        match err.message() {
                            Some(details) => call_error(details),
                            None => call_error("no details"),
                        }
                    }
                }
            }
            Err(details) => call_error(&details),
        }
    }

    fn extract<'a, T>(&self, response: &'a Message) -> Result<T, String>
        where T: Get<'a>
    {
        match response.get1() {
            Some(data) => Ok(data),
            None => Err("D-Bus wrong response type".to_string()),
        }
    }

    fn with_path<'a, P: Into<Path<'a>>>(&'a self, path: P) -> ConnPath<'a, &'a DBusConnection> {
        self.connection.with_path(NM_SERVICE_MANAGER, path, 2000)
    }
}


trait Property<T> {
    fn property(&self, path: &str, interface: &str, name: &str) -> Result<T, String>;
}


impl<T> Property<T> for NetworkManager
    where NetworkManager: VariantTo<T>
{
    fn property(&self, path: &str, interface: &str, name: &str) -> Result<T, String> {
        let property_error = |details: &str| {
            Err(format!("D-Bus get '{}'::'{}' property failed on '{}': {}",
                        interface,
                        name,
                        path,
                        details))
        };

        let path = self.with_path(path);

        match path.get(interface, name) {
            Ok(variant) => {
                match NetworkManager::variant_to(variant) {
                    Some(data) => Ok(data),
                    None => property_error("wrong property type"),
                }
            }
            Err(err) => {
                match err.message() {
                    Some(details) => property_error(details),
                    None => property_error("no details"),
                }
            }
        }
    }
}


trait VariantTo<T> {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<T>;
}


impl VariantTo<String> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<String> {
        variant_to_string(value)
    }
}


impl VariantTo<i64> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<i64> {
        variant_to_i64(value)
    }
}


impl VariantTo<u32> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<u32> {
        variant_to_u32(value)
    }
}


impl VariantTo<bool> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<bool> {
        variant_to_bool(value)
    }
}


impl VariantTo<Vec<String>> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
        variant_to_string_vec(value)
    }
}


impl VariantTo<Vec<u8>> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<Vec<u8>> {
        variant_to_u8_vec(value)
    }
}


impl VariantTo<DeviceType> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<DeviceType> {
        variant_to_device_type(value)
    }
}


impl VariantTo<DeviceState> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<DeviceState> {
        variant_to_device_state(value)
    }
}


impl VariantTo<NM80211ApFlags> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<NM80211ApFlags> {
        variant_to_ap_flags(value)
    }
}


impl VariantTo<NM80211ApSecurityFlags> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<NM80211ApSecurityFlags> {
        variant_to_ap_security_flags(value)
    }
}


fn variant_to_string_vec(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
    let mut result = Vec::new();

    if let Some(list) = value.0.as_iter() {
        for element in list {
            if let Some(string) = element.as_str() {
                result.push(string.to_string());
            } else {
                return None;
            }
        }

        Some(result)
    } else {
        None
    }
}


fn variant_to_u8_vec(value: Variant<Box<RefArg>>) -> Option<Vec<u8>> {
    let mut result = Vec::new();

    if let Some(list) = value.0.as_iter() {
        for element in list {
            if let Some(value) = element.as_i64() {
                result.push(value as u8);
            } else {
                return None;
            }
        }

        Some(result)
    } else {
        None
    }
}


fn variant_to_string(value: Variant<Box<RefArg>>) -> Option<String> {
    if let Some(string) = value.0.as_str() {
        Some(string.to_string())
    } else {
        None
    }
}


fn variant_to_i64(value: Variant<Box<RefArg>>) -> Option<i64> {
    value.0.as_i64()
}


fn variant_to_u32(value: Variant<Box<RefArg>>) -> Option<u32> {
    match value.0.as_i64() {
        Some(integer) => Some(integer as u32),
        None => None,
    }
}


fn variant_to_bool(value: Variant<Box<RefArg>>) -> Option<bool> {
    if let Some(integer) = value.0.as_i64() {
        Some(integer == 0)
    } else {
        None
    }
}


fn variant_to_device_type(value: Variant<Box<RefArg>>) -> Option<DeviceType> {
    if let Some(integer) = value.0.as_i64() {
        Some(DeviceType::from(integer))
    } else {
        None
    }
}


fn variant_to_device_state(value: Variant<Box<RefArg>>) -> Option<DeviceState> {
    if let Some(integer) = value.0.as_i64() {
        Some(DeviceState::from(integer))
    } else {
        None
    }
}


fn variant_to_ap_flags(value: Variant<Box<RefArg>>) -> Option<NM80211ApFlags> {
    if let Some(integer) = value.0.as_i64() {
        Some(NM80211ApFlags::from_bits(integer as u32).unwrap())
    } else {
        None
    }
}


fn variant_to_ap_security_flags(value: Variant<Box<RefArg>>) -> Option<NM80211ApSecurityFlags> {
    if let Some(integer) = value.0.as_i64() {
        Some(NM80211ApSecurityFlags::from_bits(integer as u32).unwrap())
    } else {
        None
    }
}


fn extract<'a, T>(var: &'a Variant<Iter>) -> Result<T, String>
    where T: Get<'a>
{
    match var.0.clone().get::<T>() {
        Some(value) => Ok(value),
        None => Err(format!("D-Bus variant type does not match: {:?}", var)),
    }
}


fn utf8_vec_u8_to_string(var: Vec<u8>) -> Result<String, String> {
    match ::std::str::from_utf8(&var) {
        Ok(string) => Ok(string.to_string()),
        Err(_) => Err(format!("D-Bus variant not a UTF-8 string: {:?}", var)),
    }
}

fn utf8_variant_to_string(var: &Variant<Iter>) -> Result<String, String> {
    let array_option = &var.0.clone().get::<Array<u8, _>>();

    if let Some(array) = *array_option {
        utf8_vec_u8_to_string(array.collect())
    } else {
        Err(format!("D-Bus variant not an array: {:?}", var))
    }
}
