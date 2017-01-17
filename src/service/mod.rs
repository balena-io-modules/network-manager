extern crate dbus;

use std;
use general::*;

/// Enables the Network Manager service.
///
/// # Examples
///
/// ```
/// # network_manager::service::disable(10);
/// let state = network_manager::service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(time_out: i32) -> Result<ServiceState, String> {
    if state().unwrap() == ServiceState::Active {
        return Ok(ServiceState::Active);
    }

    let mut message = dbus_message!(SD_SERVICE_MANAGER,
                                    SD_SERVICE_PATH,
                                    SD_MANAGER_INTERFACE,
                                    "StartUnit");
    message.append_items(&["NetworkManager.service".into(), "fail".into()]);
    dbus_connect!(message).unwrap();

    wait(time_out, ServiceState::Active)
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```
/// # network_manager::service::enable(10);
/// let state = network_manager::service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(time_out: i32) -> Result<ServiceState, String> {
    if state().unwrap() == ServiceState::Inactive {
        return Ok(ServiceState::Inactive);
    }

    let mut message = dbus_message!(SD_SERVICE_MANAGER,
                                    SD_SERVICE_PATH,
                                    SD_MANAGER_INTERFACE,
                                    "StopUnit");
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
    let mut message = dbus_message!(SD_SERVICE_MANAGER,
                                    SD_SERVICE_PATH,
                                    SD_MANAGER_INTERFACE,
                                    "GetUnit");
    message.append_items(&["NetworkManager.service".into()]);
    let response = dbus_connect!(message).unwrap();
    let unit_path: dbus::Path = response.get1().unwrap();

    let state: ServiceState = dbus_property!(SD_SERVICE_MANAGER,
                                             unit_path,
                                             SD_UNIT_INTERFACE,
                                             "ActiveState")
        .unwrap()
        .inner::<&String>()
        .unwrap()
        .parse()
        .unwrap();

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
        std::thread::sleep(std::time::Duration::from_secs(1));
        total_time = total_time + 1;
    }

    Err("service timed out".to_string())
}
