use dbus::Connection as DBusConnection;
use dbus::{BusType, Path, ConnPath, Message};
use dbus::arg::{Dict, Variant, Iter, Array, Get, RefArg};
use dbus::stdintf::OrgFreedesktopDBusProperties;

use enum_primitive::FromPrimitive;

use connection::ConnectionSettings;
use general::*;


pub struct NetworkManager {
    connection: DBusConnection,
}


impl NetworkManager {
    pub fn new() -> Self {
        let connection = DBusConnection::get_private(BusType::System).unwrap();

        NetworkManager { connection: connection }

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

    fn call(&self, path: &str, interface: &str, method: &str) -> Result<Message, String> {
        let call_error = |details: &str| {
            Err(format!("D-Bus '{}'::'{}' method call failed on '{}': {}",
                        interface,
                        method,
                        path,
                        details))
        };

        match Message::new_method_call(NM_SERVICE_MANAGER, path, interface, method) {
            Ok(message) => {
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


impl VariantTo<Vec<String>> for NetworkManager {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
        variant_to_string_list(value)
    }
}


fn variant_to_string_list(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
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


fn extract<'a, T>(var: &'a Variant<Iter>) -> Result<T, String>
    where T: Get<'a>
{
    match var.0.clone().get::<T>() {
        Some(value) => Ok(value),
        None => Err(format!("D-Bus variant type does not match: {:?}", var)),
    }
}


fn utf8_variant_to_string(var: &Variant<Iter>) -> Result<String, String> {
    let array_option = &var.0.clone().get::<Array<u8, _>>();

    if let Some(array) = *array_option {
        let utf8_vec = array.collect::<Vec<u8>>();

        match ::std::str::from_utf8(&utf8_vec) {
            Ok(string) => Ok(string.to_string()),
            Err(_) => Err(format!("D-Bus variant not a UTF-8 string: {:?}", var)),
        }
    } else {
        Err(format!("D-Bus variant not an array: {:?}", var))
    }
}
