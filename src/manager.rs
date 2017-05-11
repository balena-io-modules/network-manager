use std::collections::HashMap;

use dbus::Connection as DBusConnection;
use dbus::{BusType, Path, ConnPath, Message};
use dbus::arg::{Dict, Variant, Iter, Array, Get, RefArg};
use dbus::stdintf::OrgFreedesktopDBusProperties;
use dbus::Error;

use enum_primitive::FromPrimitive;

use connection::{ConnectionSettings, ConnectionState};
use device::{DeviceType, DeviceState};
use status::{Connectivity, NetworkManagerState};
use wifi::{NM80211ApSecurityFlags, NM80211ApFlags, Security, WEP, NONE};


type SettingsMap = HashMap<String, Variant<Box<RefArg>>>;

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

pub const NM_WEP_KEY_TYPE_PASSPHRASE: u32 = 2;

pub const TIMEOUT: i32 = 10_000;
pub const RETRIES_ALLOWED: usize = 50;


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
        self.property(path, NM_ACTIVE_INTERFACE, "Connection").ok()
    }

    pub fn get_connection_state(&self, path: &String) -> Result<ConnectionState, String> {
        let state_i64 = match self.property(path, NM_ACTIVE_INTERFACE, "State") {
            Ok(state_i64) => state_i64,
            Err(_) => return Ok(ConnectionState::Unknown),
        };

        ConnectionState::from_i64(state_i64)
            .ok_or(format!("Undefined connection state for {}", path))
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
                                 vec![&try!(Path::new(path.as_str())) as &RefArg,
                                      &try!(Path::new("/")) as &RefArg,
                                      &try!(Path::new("/")) as &RefArg]));

        Ok(())
    }

    pub fn deactivate_connection(&self, path: &String) -> Result<(), String> {
        try!(self.call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "DeactivateConnection",
                                 vec![&try!(Path::new(path.as_str())) as &RefArg]));

        Ok(())
    }

    pub fn add_and_activate_connection(&self,
                                       device_path: &String,
                                       ap_path: &String,
                                       ssid: &String,
                                       security: &Security,
                                       password: &str)
                                       -> Result<(String, String), String> {
        let mut settings: HashMap<String, SettingsMap> = HashMap::new();

        let mut wireless: SettingsMap = HashMap::new();
        add_val(&mut wireless, "ssid", string_to_utf8_vec_u8(&ssid.clone()));
        settings.insert("802-11-wireless".to_string(), wireless);

        if *security != NONE {
            let mut security_settings: SettingsMap = HashMap::new();

            if security.contains(WEP) {
                add_val(&mut security_settings,
                        "wep-key-type",
                        NM_WEP_KEY_TYPE_PASSPHRASE);
                add_str(&mut security_settings, "wep-key0", password);
            } else {
                add_str(&mut security_settings, "key-mgmt", "wpa-psk");
                add_str(&mut security_settings, "psk", password);
            };

            settings.insert("802-11-wireless-security".to_string(), security_settings);
        }

        let response =
            try!(self.call_with_args(NM_SERVICE_PATH,
                                     NM_SERVICE_INTERFACE,
                                     "AddAndActivateConnection",
                                     vec![&settings as &RefArg,
                                          &try!(Path::new(device_path.clone())) as &RefArg,
                                          &try!(Path::new(ap_path.clone())) as &RefArg]));


        let (conn_path, active_connection): (Path, Path) = try!(self.extract_two(&response));

        Ok((try!(path_to_string(&conn_path)), try!(path_to_string(&active_connection))))
    }

    pub fn create_hotspot(&self,
                          device_path: &String,
                          interface: &String,
                          ssid: &str,
                          password: Option<String>)
                          -> Result<(String, String), String> {
        let mut wireless: SettingsMap = HashMap::new();
        add_val(&mut wireless,
                "ssid",
                string_to_utf8_vec_u8(&ssid.to_string()));
        add_str(&mut wireless, "band", "bg");
        add_val(&mut wireless, "hidden", false);
        add_str(&mut wireless, "mode", "ap");

        let mut connection: SettingsMap = HashMap::new();
        add_val(&mut connection, "autoconnect", false);
        add_str(&mut connection, "id", ssid);
        add_str(&mut connection, "interface-name", interface);
        add_str(&mut connection, "type", "802-11-wireless");

        let mut ipv4: SettingsMap = HashMap::new();
        add_str(&mut ipv4, "method", "shared");

        let mut settings: HashMap<String, SettingsMap> = HashMap::new();

        if let Some(password) = password {
            add_str(&mut wireless, "security", "802-11-wireless-security");

            let mut security: SettingsMap = HashMap::new();
            add_str(&mut security, "key-mgmt", "wpa-psk");
            add_str(&mut security, "psk", &password);

            settings.insert("802-11-wireless-security".to_string(), security);
        }

        settings.insert("802-11-wireless".to_string(), wireless);
        settings.insert("connection".to_string(), connection);
        settings.insert("ipv4".to_string(), ipv4);

        let response = try!(self.call_with_args(NM_SERVICE_PATH,
                                                NM_SERVICE_INTERFACE,
                                                "AddAndActivateConnection",
                                                vec![&settings as &RefArg,
                                                     &try!(Path::new(device_path.clone())) as
                                                     &RefArg,
                                                     &try!(Path::new("/")) as &RefArg]));


        let (conn_path, active_connection): (Path, Path) = try!(self.extract_two(&response));

        Ok((try!(path_to_string(&conn_path)), try!(path_to_string(&active_connection))))
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
                                 vec![&try!(Path::new("/")) as &RefArg,
                                      &try!(Path::new(path.as_str())) as &RefArg,
                                      &try!(Path::new("/")) as &RefArg]));

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
        self.call_with_args(path, interface, method, vec![])
    }

    fn call_with_args(&self,
                      path: &str,
                      interface: &str,
                      method: &str,
                      args: Vec<&RefArg>)
                      -> Result<Message, String> {
        let call_error = |details: &str| {
            Err(format!("D-Bus '{}'::'{}' method call failed on '{}': {}",
                        interface,
                        method,
                        path,
                        details))
        };

        match self.call_with_args_retry(path, interface, method, args) {
            Ok(response) => Ok(response),
            Err(error) => call_error(&error),
        }
    }

    fn call_with_args_retry(&self,
                            path: &str,
                            interface: &str,
                            method: &str,
                            args: Vec<&RefArg>)
                            -> Result<Message, String> {
        let mut retries = 0;

        loop {
            if let Ok(result) = self.create_and_send_message(path, interface, method, &args) {
                return result;
            }

            retries += 1;

            if retries == RETRIES_ALLOWED {
                return Err(format!("method failed after {} retries", RETRIES_ALLOWED));
            }

            ::std::thread::sleep(::std::time::Duration::from_secs(1));
        }
    }

    fn create_and_send_message(&self,
                               path: &str,
                               interface: &str,
                               method: &str,
                               args: &Vec<&RefArg>)
                               -> Result<Result<Message, String>, String> {
        match Message::new_method_call(NM_SERVICE_MANAGER, path, interface, method) {
            Ok(mut message) => {
                if args.len() > 0 {
                    message = message.append_ref(args);
                }

                self.send_message_checked(message)
            }
            Err(details) => Ok(Err(details)),
        }
    }

    fn send_message_checked(&self, message: Message) -> Result<Result<Message, String>, String> {
        match self.connection.send_with_reply_and_block(message, TIMEOUT) {
            Ok(response) => Ok(Ok(response)),
            Err(err) => {
                let message = get_error_message(&err).to_string();
                if err.name() == Some("org.freedesktop.NetworkManager.UnknownConnection") {
                    return Err(message);
                } else {
                    Ok(Err(message))
                }
            }
        }
    }

    fn extract<'a, T>(&self, response: &'a Message) -> Result<T, String>
        where T: Get<'a>
    {
        response
            .get1()
            .ok_or("D-Bus wrong response type".to_string())
    }

    fn extract_two<'a, T1, T2>(&self, response: &'a Message) -> Result<(T1, T2), String>
        where T1: Get<'a>,
              T2: Get<'a>
    {
        let (first, second) = response.get2();

        if let Some(first) = first {
            if let Some(second) = second {
                return Ok((first, second));
            }
        }

        Err("D-Bus wrong response type".to_string())
    }

    fn with_path<'a, P: Into<Path<'a>>>(&'a self, path: P) -> ConnPath<'a, &'a DBusConnection> {
        self.connection.with_path(NM_SERVICE_MANAGER, path, TIMEOUT)
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
    value.0.as_str().and_then(|v| Some(v.to_string()))
}


fn variant_to_i64(value: Variant<Box<RefArg>>) -> Option<i64> {
    value.0.as_i64()
}


fn variant_to_u32(value: Variant<Box<RefArg>>) -> Option<u32> {
    value.0.as_i64().and_then(|v| Some(v as u32))
}


fn variant_to_bool(value: Variant<Box<RefArg>>) -> Option<bool> {
    value.0.as_i64().and_then(|v| Some(v == 0))
}


fn variant_to_device_type(value: Variant<Box<RefArg>>) -> Option<DeviceType> {
    value.0.as_i64().and_then(|v| Some(DeviceType::from(v)))
}


fn variant_to_device_state(value: Variant<Box<RefArg>>) -> Option<DeviceState> {
    value.0.as_i64().and_then(|v| Some(DeviceState::from(v)))
}


fn variant_to_ap_flags(value: Variant<Box<RefArg>>) -> Option<NM80211ApFlags> {
    value
        .0
        .as_i64()
        .and_then(|v| NM80211ApFlags::from_bits(v as u32))
}


fn variant_to_ap_security_flags(value: Variant<Box<RefArg>>) -> Option<NM80211ApSecurityFlags> {
    value
        .0
        .as_i64()
        .and_then(|v| NM80211ApSecurityFlags::from_bits(v as u32))
}


fn extract<'a, T>(var: &'a Variant<Iter>) -> Result<T, String>
    where T: Get<'a>
{
    var.0
        .clone()
        .get::<T>()
        .ok_or(format!("D-Bus variant type does not match: {:?}", var))
}


fn utf8_vec_u8_to_string(var: Vec<u8>) -> Result<String, String> {
    String::from_utf8(var).or(Err(format!("D-Bus variant not a UTF-8 string")))
}

fn utf8_variant_to_string(var: &Variant<Iter>) -> Result<String, String> {
    let array_option = &var.0.clone().get::<Array<u8, _>>();

    if let Some(array) = *array_option {
        utf8_vec_u8_to_string(array.collect())
    } else {
        Err(format!("D-Bus variant not an array: {:?}", var))
    }
}

fn string_to_utf8_vec_u8(var: &String) -> Vec<u8> {
    var.as_bytes().to_vec()
}

fn path_to_string(path: &Path) -> Result<String, String> {
    if let Ok(slice) = path.as_cstr().to_str() {
        Ok(slice.to_string())
    } else {
        Err(format!("Path not a UTF-8 string: {:?}", path))
    }
}

fn add_val<T>(map: &mut SettingsMap, key: &str, value: T)
    where T: RefArg + 'static
{
    map.insert(key.to_string(), Variant(Box::new(value)));
}

fn add_str(map: &mut SettingsMap, key: &str, value: &str) {
    map.insert(key.to_string(), Variant(Box::new(value.to_string())));
}

fn get_error_message(err: &Error) -> &str {
    match err.message() {
        Some(details) => details,
        None => "Undefined error message",
    }
}
