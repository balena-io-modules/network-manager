extern crate dbus;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_timer;

use std::str::FromStr;
use std::time::Duration;
use self::dbus::{BusType, Connection, ConnectionItem, Interface, Member, Message, Path, Props};
use self::dbus::arg::{Dict, Iter, Variant};
use self::futures::Future;
use self::futures_cpupool::CpuPool;
use self::tokio_timer::Timer;

use errors::*;

pub const SD_SERVICE_MANAGER: &str = "org.freedesktop.systemd1";
pub const SD_SERVICE_PATH: &str = "/org/freedesktop/systemd1";
pub const SD_MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";
pub const SD_UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";

pub fn start_service(timeout: u64) -> Result<ServiceState> {
    let state = get_service_state()?;
    match state {
        ServiceState::Active => Ok(state),
        ServiceState::Activating => handler(timeout, ServiceState::Active),
        ServiceState::Failed => bail!(ErrorKind::Service),
        _ => {
            let message = Message::new_method_call(
                SD_SERVICE_MANAGER,
                SD_SERVICE_PATH,
                SD_MANAGER_INTERFACE,
                "StartUnit",
            ).map_err(|_| ErrorKind::Service)?
                .append2("NetworkManager.service", "fail");

            let connection =
                Connection::get_private(BusType::System).map_err(|_| ErrorKind::Service)?;

            connection
                .send_with_reply_and_block(message, 2000)
                .map_err(|_| ErrorKind::Service)?;

            handler(timeout, ServiceState::Active)
        },
    }
}

pub fn stop_service(timeout: u64) -> Result<ServiceState> {
    let state = get_service_state()?;
    match state {
        ServiceState::Inactive => Ok(state),
        ServiceState::Deactivating => handler(timeout, ServiceState::Inactive),
        ServiceState::Failed => bail!(ErrorKind::Service),
        _ => {
            let message = Message::new_method_call(
                SD_SERVICE_MANAGER,
                SD_SERVICE_PATH,
                SD_MANAGER_INTERFACE,
                "StopUnit",
            ).map_err(|_| ErrorKind::Service)?
                .append2("NetworkManager.service", "fail");

            let connection =
                Connection::get_private(BusType::System).map_err(|_| ErrorKind::Service)?;

            connection
                .send_with_reply_and_block(message, 2000)
                .map_err(|_| ErrorKind::Service)?;

            handler(timeout, ServiceState::Inactive)
        },
    }
}

pub fn get_service_state() -> Result<ServiceState> {
    let message = Message::new_method_call(
        SD_SERVICE_MANAGER,
        SD_SERVICE_PATH,
        SD_MANAGER_INTERFACE,
        "GetUnit",
    ).map_err(|_| ErrorKind::Service)?
        .append1("NetworkManager.service");

    let connection = Connection::get_private(BusType::System).map_err(|_| ErrorKind::Service)?;

    let response = connection
        .send_with_reply_and_block(message, 2000)
        .map_err(|_| ErrorKind::Service)?;

    let path = response.get1::<Path>().ok_or(ErrorKind::Service)?;

    let response = Props::new(
        &connection,
        SD_SERVICE_MANAGER,
        path,
        SD_UNIT_INTERFACE,
        2000,
    ).get("ActiveState")
        .map_err(|_| ErrorKind::Service)?;

    response
        .inner::<&str>()
        .ok()
        .ok_or(ErrorKind::Service)?
        .parse()
}

fn handler(timeout: u64, target_state: ServiceState) -> Result<ServiceState> {
    if timeout == 0 {
        return get_service_state();
    }

    let timer = Timer::default()
        .sleep(Duration::from_secs(timeout))
        .then(|_| bail!(ErrorKind::Service));

    let process = CpuPool::new_num_cpus().spawn_fn(|| {
        let connection = Connection::get_private(BusType::System).map_err(|_| ErrorKind::Service)?;
        connection
            .add_match(
                "type='signal', sender='org.freedesktop.systemd1', \
                 interface='org.freedesktop.DBus.Properties', \
                 member='PropertiesChanged', \
                 path='/org/freedesktop/systemd1/unit/NetworkManager_2eservice'",
            )
            .map_err(|_| ErrorKind::Service)?;

        if get_service_state()? == target_state {
            return Ok(target_state);
        }

        for item in connection.iter(0) {
            let response = if let ConnectionItem::Signal(ref signal) = item {
                signal
            } else {
                continue;
            };

            if response.interface().ok_or(ErrorKind::Service)?
                != Interface::from("org.freedesktop.DBus.Properties")
                || response.member().ok_or(ErrorKind::Service)? != Member::from("PropertiesChanged")
                || response.path().ok_or(ErrorKind::Service)?
                    != Path::from("/org/freedesktop/systemd1/unit/NetworkManager_2eservice")
            {
                continue;
            }

            let (interface, dictionary) = response.get2::<&str, Dict<&str, Variant<Iter>, _>>();

            if interface.ok_or(ErrorKind::Service)? != "org.freedesktop.systemd1.Unit" {
                continue;
            }

            for (k, mut v) in dictionary.ok_or(ErrorKind::Service)? {
                if k == "ActiveState" {
                    let response = v.0.get::<&str>().ok_or(ErrorKind::Service)?;
                    let state: ServiceState = response.parse()?;
                    if state == target_state {
                        return Ok(target_state);
                    }
                }
            }
        }
        bail!(ErrorKind::Service)
    });

    match timer.select(process).map(|(result, _)| result).wait() {
        Ok(val) => Ok(val),
        Err(val) => Err(val.0),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ServiceState {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}

impl FromStr for ServiceState {
    type Err = Error;
    fn from_str(s: &str) -> Result<ServiceState> {
        match s {
            "active" => Ok(ServiceState::Active),
            "reloading" => Ok(ServiceState::Reloading),
            "inactive" => Ok(ServiceState::Inactive),
            "failed" => Ok(ServiceState::Failed),
            "activating" => Ok(ServiceState::Activating),
            "deactivating" => Ok(ServiceState::Deactivating),
            _ => bail!(ErrorKind::Service),
        }
    }
}
