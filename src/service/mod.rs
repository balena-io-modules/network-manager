extern crate dbus;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_timer;

use std::str::FromStr;
use std::time::Duration;
use self::dbus::{Connection, ConnectionItem, Message, Props, BusType, Path, Interface, Member};
use self::dbus::arg::{Dict, Iter, Variant};
use self::futures::Future;
use self::futures_cpupool::CpuPool;
use self::tokio_timer::Timer;

pub const SD_SERVICE_MANAGER: &'static str = "org.freedesktop.systemd1";
pub const SD_SERVICE_PATH: &'static str = "/org/freedesktop/systemd1";
pub const SD_MANAGER_INTERFACE: &'static str = "org.freedesktop.systemd1.Manager";
pub const SD_UNIT_INTERFACE: &'static str = "org.freedesktop.systemd1.Unit";

/// Enables the Network Manager service.
///
/// # Examples
///
/// ```
/// use network_manager::service;
/// let state = service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(time_out: u64) -> Result<State, Error> {
    let state = try!(state());
    match state {
        State::Active => Ok(state),
        State::Activating => handler(time_out, State::Active),
        State::Failed => Err(Error::Failed),
        _ => {
            let message = try!(Message::new_method_call(SD_SERVICE_MANAGER,
                                                        SD_SERVICE_PATH,
                                                        SD_MANAGER_INTERFACE,
                                                        "StartUnit")
                    .map_err(Error::Message))
                .append2("NetworkManager.service", "fail");

            let connection = try!(Connection::get_private(BusType::System)
                .map_err(Error::Connection));

            try!(connection.send_with_reply_and_block(message, 2000).map_err(Error::Connection));

            handler(time_out, State::Active)
        }
    }
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```
/// use network_manager::service;
/// let state = service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(time_out: u64) -> Result<State, String> {
    // match state().expect("Unable to get service state") {
    //     ServiceState::Inactive => Ok(ServiceState::Inactive),
    //     ServiceState::Deactivating => wait(time_out, ServiceState::Inactive),
    //     ServiceState::Failed => Err("Service has failed".to_string()),
    //     _ => {
    //         let mut message = dbus_message!(SD_SERVICE_MANAGER,
    //                                         SD_SERVICE_PATH,
    //                                         SD_MANAGER_INTERFACE,
    //                                         "StopUnit");
    //         message.append_items(&["NetworkManager.service".into(), "fail".into()]);
    //         dbus_connect!(message);
    //
    //         wait(time_out, ServiceState::Inactive)
    //     }
    // }
    Ok(State::Active)
}

// #[test]
// fn test_enable_disable_functions() {
//     let s = state().unwrap();
//
//     assert!(s == ServiceState::Active || s == ServiceState::Inactive);
//
//     match s {
//         ServiceState::Active => {
//             disable(10).unwrap();
//             assert_eq!(ServiceState::Inactive, state().unwrap());
//
//             enable(10).unwrap();
//             assert_eq!(ServiceState::Active, state().unwrap());
//         }
//         ServiceState::Inactive => {
//             enable(10).unwrap();
//             assert_eq!(ServiceState::Active, state().unwrap());
//
//             disable(10).unwrap();
//             assert_eq!(ServiceState::Inactive, state().unwrap());
//         }
//         _ => (),
//     }
// }
//

/// Gets the state of the Network Manager service.
///
/// # Examples
///
/// ```
/// use network_manager::service;
/// let state = service::state().unwrap();
/// println!("{:?}", state);
/// ```
pub fn state() -> Result<State, Error> {
    let message = try!(Message::new_method_call(SD_SERVICE_MANAGER,
                                                SD_SERVICE_PATH,
                                                SD_MANAGER_INTERFACE,
                                                "GetUnit")
            .map_err(Error::Message))
        .append1("NetworkManager.service");

    let connection = try!(Connection::get_private(BusType::System).map_err(Error::Connection));

    let response = try!(connection.send_with_reply_and_block(message, 2000)
        .map_err(Error::Connection));

    let path = try!(response.get1::<Path>().ok_or(Error::NotFound));

    let response = try!(Props::new(&connection,
                                   SD_SERVICE_MANAGER,
                                   path,
                                   SD_UNIT_INTERFACE,
                                   2000)
        .get("ActiveState")
        .map_err(Error::Props));

    try!(response.inner::<&str>().ok().ok_or(Error::NotFound)).parse()
}

fn handler(time_out: u64, target_state: State) -> Result<State, Error> {
    if time_out == 0 {
        return state();
    }

    let timer = Timer::default().sleep(Duration::from_secs(time_out)).then(|_| Err(Error::TimedOut));

    let pool = CpuPool::new_num_cpus();
    let process = pool.spawn_fn(|| {
        let connection = try!(Connection::get_private(BusType::System).map_err(Error::Connection));
        try!(connection.add_match("type='signal', sender='org.freedesktop.systemd1', \
                        interface='org.freedesktop.DBus.Properties', \
                        member='PropertiesChanged', \
                        path='/org/freedesktop/systemd1/unit/NetworkManager_2eservice'")
            .map_err(Error::Connection));

        for item in connection.iter(0) {
            let response = if let ConnectionItem::Signal(ref signal) = item {
                signal
            } else {
                continue;
            };

            if try!(response.interface().ok_or(Error::NotFound)) !=
               Interface::from("org.freedesktop.DBus.Properties") {
                continue;
            }

            if try!(response.member().ok_or(Error::NotFound)) != Member::from("PropertiesChanged") {
                continue;
            }

            if try!(response.path().ok_or(Error::NotFound)) !=
               Path::from("/org/freedesktop/systemd1/unit/NetworkManager_2eservice") {
                continue;
            }

            let (interface, dictionary) = response.get2::<&str, Dict<&str, Variant<Iter>, _>>();

            if try!(interface.ok_or(Error::NotFound)) != "org.freedesktop.systemd1.Unit" {
                continue;
            }

            for (k, v) in try!(dictionary.ok_or(Error::NotFound)) {
                match k {
                    "ActiveState" => {
                        let response = try!(v.0.clone().get::<&str>().ok_or(Error::NotFound));
                        let state: State = try!(response.parse());
                        if state == target_state {
                            return Ok(target_state);
                        }
                    }
                    _ => (),
                }
            }
        }
        Err(Error::NotFound)
    });

    match timer.select(process).map(|(result, _)| result).wait() {
        Ok(val) => Ok(val),
        Err(val) => Err(val.0),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum State {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}

#[derive(Debug)]
pub enum Error {
    Message(String),
    Connection(dbus::Error),
    Props(dbus::Error),
    TimedOut,
    Failed,
    NotFound,
}

impl FromStr for State {
    type Err = Error;
    fn from_str(s: &str) -> Result<State, Error> {
        match s {
            "active" => Ok(State::Active),
            "reloading" => Ok(State::Reloading),
            "inactive" => Ok(State::Inactive),
            "failed" => Ok(State::Failed),
            "activating" => Ok(State::Activating),
            "deactivating" => Ok(State::Deactivating),
            _ => Err(Error::NotFound),
        }
    }
}
