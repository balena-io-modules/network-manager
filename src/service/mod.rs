extern crate dbus;

use std::str::FromStr;
use self::dbus::{Connection, Message, Props, BusType};

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
pub fn enable(time_out: i32) -> Result<State, Error> {
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
pub fn disable(time_out: i32) -> Result<State, String> {
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

    let path = try!(response.get1::<dbus::Path>().ok_or(Error::NotFound));

    let response = try!(Props::new(&connection,
                                   SD_SERVICE_MANAGER,
                                   path,
                                   SD_UNIT_INTERFACE,
                                   2000)
        .get("ActiveState")
        .map_err(Error::Props));

    try!(response.inner::<&str>().ok().ok_or(Error::NotFound)).parse()
}

fn handler(time_out: i32, target_state: State) -> Result<State, Error> {
    if time_out == 0 {
        return state();
    }

    let connection = try!(Connection::get_private(BusType::System).map_err(Error::Connection));
    try!(connection.add_match("interface='org.freedesktop.systemd1.Unit',member='ActiveState'").map_err(Error::Connection));

    println!("hey");

    for item in connection.iter(time_out) {
        println!("{:?}", item);
    }

    println!("bye");

    Ok(State::Active)
}




// fn wait(time_out: i32, target_state: ServiceState) -> Result<ServiceState, String> {
//
//     let mut total_time = 0;
//     while total_time < time_out {
//         if state().unwrap() == target_state {
//             return state();
//         }
//         std::thread::sleep(std::time::Duration::from_secs(1));
//         total_time += 1;
//     }
//
//     Err("service timed out".to_string())
// }

#[derive(Debug)]
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
