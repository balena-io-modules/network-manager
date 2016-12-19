extern crate dbus;

use self::dbus::{Connection, BusType, Message, Path, Props};
use std::{thread, time};
use general::ServiceState;

macro_rules! dbus_message {
    ($function:expr) => {{
        let service = "org.freedesktop.systemd1";
        let path = "/org/freedesktop/systemd1";
        let interface = "org.freedesktop.systemd1.Manager";
        Message::new_method_call(service, path, interface, $function).unwrap()
    }}
}

macro_rules! dbus_connect {
    ($message:expr) => {{
        Connection::get_private(BusType::System).unwrap().
            send_with_reply_and_block($message, 2000)
    }}
}

/// Enables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(time_out: i32) -> Result<ServiceState, String> {
    let mut message = dbus_message!("StartUnit");
    message.append_items(&["NetworkManager.service".into(), "fail".into()]);
    dbus_connect!(message).unwrap();

    wait(time_out, ServiceState::Active)
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(time_out: i32) -> Result<ServiceState, String> {
    let mut message = dbus_message!("StopUnit");
    message.append_items(&["NetworkManager.service".into(), "fail".into()]);
    dbus_connect!(message).unwrap();

    wait(time_out, ServiceState::Inactive)
}

/// Gets the state of the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::state().unwrap();
/// println!("{:?}", state);
/// ```
pub fn state() -> Result<ServiceState, String> {
    let mut message = dbus_message!("GetUnit");
    message.append_items(&["NetworkManager.service".into()]);
    let response = dbus_connect!(message).unwrap();
    let path: Path = response.get1().unwrap();

    let connection = Connection::get_private(BusType::System).unwrap();
    let message = Props::new(&connection,
                             "org.freedesktop.systemd1",
                             path,
                             "org.freedesktop.systemd1.Unit",
                             2000);
    let property = message.get("ActiveState").unwrap();
    let value: &String = property.inner().unwrap();
    let state: ServiceState = value.parse().unwrap();

    Ok(state)
}

fn wait(time_out: i32, target_state: ServiceState) -> Result<ServiceState, String> {
    if time_out == 0 {
        return state();
    }

    let mut total_time = 0;
    while total_time < time_out {
        if state().unwrap() == target_state {
            return state();
        }
        thread::sleep(time::Duration::from_secs(1));
        total_time = total_time + 1;
    }

    Err("service timed out".into())
}
