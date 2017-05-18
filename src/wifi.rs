use std::rc::Rc;
use std::cell::RefCell;

use ascii::AsAsciiStr;

use dbus_nm::DBusNetworkManager;

use connection::{Connection, ConnectionState, connect_to_access_point, create_hotspot};
use device::{Device, PathGetter};


pub struct WiFiDevice<'a> {
    dbus_manager: Rc<RefCell<DBusNetworkManager>>,
    device: &'a Device,
}

impl<'a> WiFiDevice<'a> {
    /// Get the list of access points visible to this device.
    ///
    /// # Examples
    ///
    /// ```
    /// use network_manager::{NetworkManager, DeviceType};
    /// let manager = NetworkManager::new();
    /// let devices = manager.get_devices().unwrap();
    /// let i = devices.iter().position(|ref d| *d.device_type() == DeviceType::WiFi).unwrap();
    /// let device = devices[i].as_wifi_device().unwrap();
    /// let access_points = device.get_access_points().unwrap();
    /// println!("{:?}", access_points);
    /// ```
    pub fn get_access_points(&self) -> Result<Vec<AccessPoint>, String> {
        let mut access_points = Vec::new();

        let paths = try!(self.dbus_manager
                             .borrow()
                             .get_device_access_points(self.device.path()));

        for path in paths {
            if let Some(access_point) = try!(get_access_point(&self.dbus_manager.borrow(), &path)) {
                access_points.push(access_point);
            }
        }

        access_points.sort_by_key(|ap| ap.strength);
        access_points.reverse();

        Ok(access_points)
    }

    pub fn connect<T>(&self,
                      access_point: &AccessPoint,
                      password: &T)
                      -> Result<(Connection, ConnectionState), String>
        where T: AsAsciiStr + ?Sized
    {
        connect_to_access_point(&self.dbus_manager,
                                self.device.path(),
                                &access_point.path,
                                &access_point.ssid,
                                &access_point.security,
                                password)
    }

    pub fn create_hotspot<T>(&self,
                             ssid: &str,
                             password: Option<&T>)
                             -> Result<(Connection, ConnectionState), String>
        where T: AsAsciiStr + ?Sized
    {
        create_hotspot(&self.dbus_manager,
                       self.device.path(),
                       self.device.interface(),
                       ssid,
                       password)
    }
}


#[derive(Debug)]
pub struct AccessPoint {
    path: String,
    ssid: String,
    strength: u32,
    security: Security,
}

impl AccessPoint {
    pub fn ssid(&self) -> &str {
        &self.ssid
    }
}


bitflags! {
    pub flags Security: u32 {
        const NONE         = 0b0000000,
        const WEP          = 0b0000001,
        const WPA          = 0b0000010,
        const WPA2         = 0b0000100,
        const ENTERPRISE   = 0b0001000,
    }
}


bitflags! {
    pub flags NM80211ApFlags: u32 {
        // access point has no special capabilities
        const AP_FLAGS_NONE           = 0x00000000,
        // access point requires authentication and
        const AP_FLAGS_PRIVACY        = 0x00000001,
    }
}


bitflags! {
    pub flags NM80211ApSecurityFlags: u32 {
         // the access point has no special security requirements
        const AP_SEC_NONE                    = 0x00000000,
        // 40/64-bit WEP is supported for pairwise/unicast encryption
        const AP_SEC_PAIR_WEP40              = 0x00000001,
        // 104/128-bit WEP is supported for pairwise/unicast encryption
        const AP_SEC_PAIR_WEP104             = 0x00000002,
        // TKIP is supported for pairwise/unicast encryption
        const AP_SEC_PAIR_TKIP               = 0x00000004,
        // AES/CCMP is supported for pairwise/unicast encryption
        const AP_SEC_PAIR_CCMP               = 0x00000008,
        // 40/64-bit WEP is supported for group/broadcast encryption
        const AP_SEC_GROUP_WEP40             = 0x00000010,
        // 104/128-bit WEP is supported for group/broadcast encryption
        const AP_SEC_GROUP_WEP104            = 0x00000020,
        // TKIP is supported for group/broadcast encryption
        const AP_SEC_GROUP_TKIP              = 0x00000040,
        // AES/CCMP is supported for group/broadcast encryption
        const AP_SEC_GROUP_CCMP              = 0x00000080,
        // WPA/RSN Pre-Shared Key encryption is supported
        const AP_SEC_KEY_MGMT_PSK            = 0x00000100,
        // 802.1x authentication and key management is supported
        const AP_SEC_KEY_MGMT_802_1X         = 0x00000200,
    }
}


pub fn new_wifi_device<'a>(dbus_manager: &Rc<RefCell<DBusNetworkManager>>,
                           device: &'a Device)
                           -> WiFiDevice<'a> {
    WiFiDevice {
        dbus_manager: dbus_manager.clone(),
        device: device,
    }
}


fn get_access_point(manager: &DBusNetworkManager,
                    path: &String)
                    -> Result<Option<AccessPoint>, String> {
    if let Some(ssid) = manager.get_access_point_ssid(path) {
        let strength = try!(manager.get_access_point_strength(path));

        let security = try!(get_access_point_security(manager, path));

        let access_point = AccessPoint {
            path: path.clone(),
            ssid: ssid,
            strength: strength,
            security: security,
        };

        Ok(Some(access_point))
    } else {
        Ok(None)
    }
}


fn get_access_point_security(manager: &DBusNetworkManager,
                             path: &String)
                             -> Result<Security, String> {
    let flags = try!(manager.get_access_point_flags(path));

    let wpa_flags = try!(manager.get_access_point_wpa_flags(path));

    let rsn_flags = try!(manager.get_access_point_rsn_flags(path));

    let mut security = NONE;

    if flags.contains(AP_FLAGS_PRIVACY) && wpa_flags == AP_SEC_NONE && rsn_flags == AP_SEC_NONE {
        security |= WEP;
    }

    if wpa_flags != AP_SEC_NONE {
        security |= WPA;
    }

    if rsn_flags != AP_SEC_NONE {
        security |= WPA2;
    }

    if wpa_flags.contains(AP_SEC_KEY_MGMT_802_1X) || rsn_flags.contains(AP_SEC_KEY_MGMT_802_1X) {
        security |= ENTERPRISE;
    }

    Ok(security)
}
