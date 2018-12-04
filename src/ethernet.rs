use std::rc::Rc;
use std::net::Ipv4Addr;

use errors::*;
use dbus_nm::DBusNetworkManager;

use connection::{Connection, ConnectionState, set_ethernet_address};
use device::{Device, PathGetter};

pub struct EthernetDevice<'a> {
    dbus_manager: Rc<DBusNetworkManager>,
    device: &'a Device,
}

impl<'a> EthernetDevice<'a> {
    pub fn set_ethernet_address(
        &self,
        address: Ipv4Addr,
        address_netmask_bit_count: u8,
        gateway: Ipv4Addr,
        dns_addr_1: Ipv4Addr,
        dns_addr_2: Ipv4Addr,
        dns_search: &str,
        method: &str,
        connection_name: &str,
    ) -> Result<(Connection, ConnectionState)>
    {
        set_ethernet_address(
            &self.dbus_manager,
            self.device.path(),
            self.device.interface(),
            address,
            address_netmask_bit_count,
            gateway,
            dns_addr_1,
            dns_addr_2,
            dns_search,
            method,
            connection_name,
        )
    }
}

pub fn new_ethernet_device<'a>(
    dbus_manager: &Rc<DBusNetworkManager>,
    device: &'a Device,
) -> EthernetDevice<'a> {
    EthernetDevice {
        dbus_manager: Rc::clone(dbus_manager),
        device: device,
    }
}
