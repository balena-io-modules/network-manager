use std::collections::HashMap;

use dbus::Path;
use dbus::arg::{Dict, Variant, Iter, Array, RefArg};

use dbus_api::{DBusApi, extract, utf8_vec_u8_to_string, utf8_variant_to_string,
               string_to_utf8_vec_u8, path_to_string, VariantTo};
use manager::{Connectivity, NetworkManagerState};
use connection::{ConnectionSettings, ConnectionState};
use device::{DeviceType, DeviceState};
use wifi::{NM80211ApSecurityFlags, NM80211ApFlags, Security, WEP, NONE};


type SettingsMap = HashMap<String, Variant<Box<RefArg>>>;

const NM_SERVICE_MANAGER: &'static str = "org.freedesktop.NetworkManager";

const NM_SERVICE_PATH: &'static str = "/org/freedesktop/NetworkManager";
const NM_SETTINGS_PATH: &'static str = "/org/freedesktop/NetworkManager/Settings";

const NM_SERVICE_INTERFACE: &'static str = "org.freedesktop.NetworkManager";
const NM_SETTINGS_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings";
const NM_CONNECTION_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Settings.\
                                                   Connection";
const NM_ACTIVE_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Connection.Active";
const NM_DEVICE_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device";
const NM_WIRELESS_INTERFACE: &'static str = "org.freedesktop.NetworkManager.Device.Wireless";
const NM_ACCESS_POINT_INTERFACE: &'static str = "org.freedesktop.NetworkManager.AccessPoint";

const NM_WEP_KEY_TYPE_PASSPHRASE: u32 = 2;

const UNKNOWN_CONNECTION: &'static str = "org.freedesktop.NetworkManager.UnknownConnection";
const METHOD_RETRY_ERROR_NAMES: &'static [&'static str; 1] = &[UNKNOWN_CONNECTION];


pub struct DBusNetworkManager {
    dbus: DBusApi,
}

impl DBusNetworkManager {
    pub fn new() -> Self {
        DBusNetworkManager { dbus: DBusApi::new(NM_SERVICE_MANAGER, METHOD_RETRY_ERROR_NAMES) }
    }

    pub fn method_timeout(&self) -> u64 {
        self.dbus.method_timeout()
    }

    pub fn set_method_timeout(&mut self, timeout: u64) {
        self.dbus.set_method_timeout(timeout);
    }

    pub fn get_state(&self) -> Result<NetworkManagerState, String> {
        let response = try!(self.dbus
                                .call(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "state"));

        let state: i64 = try!(self.dbus.extract(&response));

        Ok(NetworkManagerState::from(state))
    }

    pub fn check_connectivity(&self) -> Result<Connectivity, String> {
        let response =
            try!(self.dbus
                     .call(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "CheckConnectivity"));

        let connectivity: i64 = try!(self.dbus.extract(&response));

        Ok(Connectivity::from(connectivity))
    }

    pub fn is_wireless_enabled(&self) -> Result<bool, String> {
        self.dbus
            .property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "WirelessEnabled")
    }

    pub fn is_networking_enabled(&self) -> Result<bool, String> {
        self.dbus
            .property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "NetworkingEnabled")
    }

    pub fn list_connections(&self) -> Result<Vec<String>, String> {
        let response =
            try!(self.dbus
                     .call(NM_SETTINGS_PATH, NM_SETTINGS_INTERFACE, "ListConnections"));

        let array: Array<Path, _> = try!(self.dbus.extract(&response));

        Ok(array.map(|e| e.to_string()).collect())
    }

    pub fn get_active_connections(&self) -> Result<Vec<String>, String> {
        self.dbus
            .property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "ActiveConnections")
    }

    pub fn get_active_connection_path(&self, path: &str) -> Option<String> {
        self.dbus
            .property(path, NM_ACTIVE_INTERFACE, "Connection")
            .ok()
    }

    pub fn get_connection_state(&self, path: &str) -> Result<ConnectionState, String> {
        let state: i64 = match self.dbus.property(path, NM_ACTIVE_INTERFACE, "State") {
            Ok(state) => state,
            Err(_) => return Ok(ConnectionState::Unknown),
        };

        Ok(ConnectionState::from(state))
    }

    pub fn get_connection_settings(&self, path: &str) -> Result<ConnectionSettings, String> {
        let response = try!(self.dbus.call(path, NM_CONNECTION_INTERFACE, "GetSettings"));

        let dict: Dict<&str, Dict<&str, Variant<Iter>, _>, _> = try!(self.dbus.extract(&response));

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

    pub fn get_active_connection_devices(&self, path: &str) -> Result<Vec<String>, String> {
        self.dbus.property(path, NM_ACTIVE_INTERFACE, "Devices")
    }

    pub fn delete_connection(&self, path: &str) -> Result<(), String> {
        try!(self.dbus.call(path, NM_CONNECTION_INTERFACE, "Delete"));

        Ok(())
    }

    pub fn activate_connection(&self, path: &str) -> Result<(), String> {
        try!(self.dbus
                 .call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "ActivateConnection",
                                 &[&try!(Path::new(path)) as &RefArg,
                                   &try!(Path::new("/")) as &RefArg,
                                   &try!(Path::new("/")) as &RefArg]));

        Ok(())
    }

    pub fn deactivate_connection(&self, path: &str) -> Result<(), String> {
        try!(self.dbus
                 .call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "DeactivateConnection",
                                 &[&try!(Path::new(path)) as &RefArg]));

        Ok(())
    }

    pub fn add_and_activate_connection(&self,
                                       device_path: &str,
                                       ap_path: &str,
                                       ssid: &str,
                                       security: &Security,
                                       password: &str)
                                       -> Result<(String, String), String> {
        let mut settings: HashMap<String, SettingsMap> = HashMap::new();

        let mut wireless: SettingsMap = HashMap::new();
        add_val(&mut wireless,
                "ssid",
                string_to_utf8_vec_u8(&ssid.to_string()));
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
            try!(self.dbus
                     .call_with_args(NM_SERVICE_PATH,
                                     NM_SERVICE_INTERFACE,
                                     "AddAndActivateConnection",
                                     &[&settings as &RefArg,
                                       &try!(Path::new(device_path.to_string())) as &RefArg,
                                       &try!(Path::new(ap_path.to_string())) as &RefArg]));


        let (conn_path, active_connection): (Path, Path) = try!(self.dbus.extract_two(&response));

        Ok((try!(path_to_string(&conn_path)), try!(path_to_string(&active_connection))))
    }

    pub fn create_hotspot(&self,
                          device_path: &str,
                          interface: &str,
                          ssid: &str,
                          password: Option<&str>)
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
            add_str(&mut security, "psk", password);

            settings.insert("802-11-wireless-security".to_string(), security);
        }

        settings.insert("802-11-wireless".to_string(), wireless);
        settings.insert("connection".to_string(), connection);
        settings.insert("ipv4".to_string(), ipv4);

        let response = try!(self.dbus
                                .call_with_args(NM_SERVICE_PATH,
                                                NM_SERVICE_INTERFACE,
                                                "AddAndActivateConnection",
                                                &[&settings as &RefArg,
                                                  &try!(Path::new(device_path.clone())) as
                                                  &RefArg,
                                                  &try!(Path::new("/")) as &RefArg]));


        let (conn_path, active_connection): (Path, Path) = try!(self.dbus.extract_two(&response));

        Ok((try!(path_to_string(&conn_path)), try!(path_to_string(&active_connection))))
    }

    pub fn get_devices(&self) -> Result<Vec<String>, String> {
        self.dbus
            .property(NM_SERVICE_PATH, NM_SERVICE_INTERFACE, "Devices")
    }

    pub fn get_device_by_interface(&self, interface: &str) -> Result<String, String> {
        let response = try!(self.dbus
                                .call_with_args(NM_SERVICE_PATH,
                                                NM_SERVICE_INTERFACE,
                                                "GetDeviceByIpIface",
                                                &[&interface.to_string() as &RefArg]));

        let path: Path = try!(self.dbus.extract(&response));

        path_to_string(&path)
    }

    pub fn get_device_interface(&self, path: &str) -> Result<String, String> {
        self.dbus.property(path, NM_DEVICE_INTERFACE, "Interface")
    }

    pub fn get_device_type(&self, path: &str) -> Result<DeviceType, String> {
        self.dbus.property(path, NM_DEVICE_INTERFACE, "DeviceType")
    }

    pub fn get_device_state(&self, path: &str) -> Result<DeviceState, String> {
        self.dbus.property(path, NM_DEVICE_INTERFACE, "State")
    }

    pub fn connect_device(&self, path: &str) -> Result<(), String> {
        try!(self.dbus
                 .call_with_args(NM_SERVICE_PATH,
                                 NM_SERVICE_INTERFACE,
                                 "ActivateConnection",
                                 &[&try!(Path::new("/")) as &RefArg,
                                   &try!(Path::new(path)) as &RefArg,
                                   &try!(Path::new("/")) as &RefArg]));

        Ok(())
    }

    pub fn disconnect_device(&self, path: &str) -> Result<(), String> {
        try!(self.dbus.call(path, NM_DEVICE_INTERFACE, "Disconnect"));

        Ok(())
    }

    pub fn get_device_access_points(&self, path: &str) -> Result<Vec<String>, String> {
        self.dbus
            .property(path, NM_WIRELESS_INTERFACE, "AccessPoints")
    }

    pub fn get_access_point_ssid(&self, path: &str) -> Option<String> {
        if let Ok(ssid_vec) = self.dbus.property(path, NM_ACCESS_POINT_INTERFACE, "Ssid") {
            utf8_vec_u8_to_string(ssid_vec).ok()
        } else {
            None
        }
    }

    pub fn get_access_point_strength(&self, path: &str) -> Result<u32, String> {
        self.dbus
            .property(path, NM_ACCESS_POINT_INTERFACE, "Strength")
    }

    pub fn get_access_point_flags(&self, path: &str) -> Result<NM80211ApFlags, String> {
        self.dbus.property(path, NM_ACCESS_POINT_INTERFACE, "Flags")
    }

    pub fn get_access_point_wpa_flags(&self, path: &str) -> Result<NM80211ApSecurityFlags, String> {
        self.dbus
            .property(path, NM_ACCESS_POINT_INTERFACE, "WpaFlags")
    }

    pub fn get_access_point_rsn_flags(&self, path: &str) -> Result<NM80211ApSecurityFlags, String> {
        self.dbus
            .property(path, NM_ACCESS_POINT_INTERFACE, "RsnFlags")
    }
}


impl VariantTo<DeviceType> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<DeviceType> {
        variant_to_device_type(value)
    }
}


impl VariantTo<DeviceState> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<DeviceState> {
        variant_to_device_state(value)
    }
}


impl VariantTo<NM80211ApFlags> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<NM80211ApFlags> {
        variant_to_ap_flags(value)
    }
}


impl VariantTo<NM80211ApSecurityFlags> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<NM80211ApSecurityFlags> {
        variant_to_ap_security_flags(value)
    }
}


fn variant_to_device_type(value: Variant<Box<RefArg>>) -> Option<DeviceType> {
    value.0.as_i64().map(DeviceType::from)
}


fn variant_to_device_state(value: Variant<Box<RefArg>>) -> Option<DeviceState> {
    value.0.as_i64().map(DeviceState::from)
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


pub fn add_val<K, V>(map: &mut SettingsMap, key: K, value: V)
    where K: Into<String>,
          V: RefArg + 'static
{
    map.insert(key.into(), Variant(Box::new(value)));
}

pub fn add_str<K, V>(map: &mut SettingsMap, key: K, value: V)
    where K: Into<String>,
          V: Into<String>
{
    map.insert(key.into(), Variant(Box::new(value.into())));
}
