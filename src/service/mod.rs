extern crate dbus;

use self::dbus::{Connection, BusType, Message, Path, Props};
use std::{thread, time};
use general::ServiceState;

/// Enables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(to: i32) -> Result<ServiceState, String> {
    let c = Connection::get_private(BusType::System).unwrap();

    let mut m = Message::new_method_call("org.freedesktop.systemd1",
                                         "/org/freedesktop/systemd1",
                                         "org.freedesktop.systemd1.Manager",
                                         "StartUnit")
        .unwrap();
    m.append_items(&["NetworkManager.service".into(), "fail".into()]);
    c.send_with_reply_and_block(m, 2000).unwrap();

    let mut s = state().unwrap();
    let mut t = 0;
    while t < to {
        thread::sleep(time::Duration::from_secs(1));
        t = t + 1;

        s = state().unwrap();
        if s == ServiceState::Active {
            break;
        }
    }

    Ok(s)
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(to: i32) -> Result<ServiceState, String> {
    let c = Connection::get_private(BusType::System).unwrap();

    let mut m = Message::new_method_call("org.freedesktop.systemd1",
                                         "/org/freedesktop/systemd1",
                                         "org.freedesktop.systemd1.Manager",
                                         "StopUnit")
        .unwrap();
    m.append_items(&["NetworkManager.service".into(), "fail".into()]);
    c.send_with_reply_and_block(m, 2000).unwrap();

    let mut s = state().unwrap();
    let mut t = 0;
    while t < to {
        thread::sleep(time::Duration::from_secs(1));
        t = t + 1;

        s = state().unwrap();
        if s == ServiceState::Inactive {
            break;
        }
    }

    Ok(s)
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
    let c = Connection::get_private(BusType::System).unwrap();

    let mut m = Message::new_method_call("org.freedesktop.systemd1",
                                         "/org/freedesktop/systemd1",
                                         "org.freedesktop.systemd1.Manager",
                                         "GetUnit")
        .unwrap();
    m.append_items(&["NetworkManager.service".into()]);
    let r = c.send_with_reply_and_block(m, 2000).unwrap();
    let p: Path = r.get1().unwrap();

    let m = Props::new(&c,
                       "org.freedesktop.systemd1",
                       p,
                       "org.freedesktop.systemd1.Unit",
                       2000);
    let r = m.get("ActiveState").unwrap();
    let v: &String = r.inner().unwrap();
    let s: ServiceState = v.parse().unwrap();

    Ok(s)
}
