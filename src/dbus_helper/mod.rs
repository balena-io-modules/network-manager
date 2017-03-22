use dbus::{Connection, Path, ConnPath};
use dbus::arg::{Variant, RefArg};
use dbus::stdintf::OrgFreedesktopDBusProperties;

use general::NM_SERVICE_MANAGER;


macro_rules! dbus_message {
    ($service:expr, $path:expr, $interface:expr, $function:expr) => {{
       dbus::Message::new_method_call($service, $path, $interface, $function).
           unwrap_or_else(|e| panic!("dbus_message error: {}", e))
    }}
}

macro_rules! dbus_property {
   ($service:expr, $path:expr, $interface:expr, $property:expr) => {{
        let connection = dbus::Connection::get_private(dbus::BusType::System)
            .unwrap_or_else(|e| panic!("dbus_property error: {}", e));

        dbus::Props::new(&connection, $service, $path, $interface, 2000).
            get($property)
    }}
}

macro_rules! dbus_connect {
    ($message:expr) => {{
        dbus::Connection::get_private(dbus::BusType::System)
            .unwrap_or_else(|e| panic!("dbus_connect error: {}", e)).
            send_with_reply_and_block($message, 2000)
            .unwrap_or_else(|e| panic!("dbus_connect error: {}", e))
    }}
}

#[inline]
pub fn variant_to_string_list(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
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

#[inline]
pub fn property_as_string(
    path: &ConnPath<&Connection>,
    interface: &str,
    property: &str
) -> Option<String> {
    if let Ok(variant) = path.get(interface, property) {
        if let Some(data) = variant.as_str() {
            Some(data.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

#[inline]
pub fn property_as_i64(
    path: &ConnPath<&Connection>,
    interface: &str,
    property: &str
) -> Option<i64> {
    if let Ok(variant) = path.get(interface, property) {
        if let Some(data) = variant.as_i64() {
            Some(data)
        } else {
            None
        }
    } else {
        None
    }
}

#[inline]
pub fn property_as_bool(
    path: &ConnPath<&Connection>,
    interface: &str,
    property: &str
) -> Option<bool> {
    if let Ok(variant) = path.get(interface, property) {
        if let Some(data) = variant.as_i64() {
            Some(data != 0)
        } else {
            None
        }
    } else {
        None
    }
}

#[inline]
pub fn manager_path<'a, P: Into<Path<'a>>>(
    connection: &'a Connection,
    path: P
) -> ConnPath<'a, &'a Connection> {
    connection.with_path(
            NM_SERVICE_MANAGER,
            path,
            2000
    )
}
