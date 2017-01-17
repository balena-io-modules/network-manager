extern crate dbus;

use std;
use general::*;

/// Enables the Network Manager service.
///
/// # Examples
///
/// ```no_run
/// use network_manager::service;
/// let state = service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(time_out: i32) -> Result<ServiceState, String> {
    match state().expect("Unable to get service state") {
        ServiceState::Active => Ok(ServiceState::Active),
        ServiceState::Activating => wait(time_out, ServiceState::Active),
        ServiceState::Failed => Err("Service has failed".to_string()),
        _ => {
            let mut message = dbus_message!(SD_SERVICE_MANAGER,
                                            SD_SERVICE_PATH,
                                            SD_MANAGER_INTERFACE,
                                            "StartUnit");
            message.append_items(&["NetworkManager.service".into(), "fail".into()]);
            dbus_connect!(message);

            wait(time_out, ServiceState::Active)
        }
    }
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```no_run
/// use network_manager::service;
/// let state = service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(time_out: i32) -> Result<ServiceState, String> {
    match state().expect("Unable to get service state") {
        ServiceState::Inactive => Ok(ServiceState::Inactive),
        ServiceState::Deactivating => wait(time_out, ServiceState::Inactive),
        ServiceState::Failed => Err("Service has failed".to_string()),
        _ => {
            let mut message = dbus_message!(SD_SERVICE_MANAGER,
                                            SD_SERVICE_PATH,
                                            SD_MANAGER_INTERFACE,
                                            "StopUnit");
            message.append_items(&["NetworkManager.service".into(), "fail".into()]);
            dbus_connect!(message);

            wait(time_out, ServiceState::Inactive)
        }
    }
}

#[test]
fn test_enable_disable_functions() {
    let s = state().unwrap();

    assert!(s == ServiceState::Active || s == ServiceState::Inactive);

    match s {
        ServiceState::Active => {
            disable(10).unwrap();
            assert_eq!(ServiceState::Inactive, state().unwrap());

            enable(10).unwrap();
            assert_eq!(ServiceState::Active, state().unwrap());
        }
        ServiceState::Inactive => {
            enable(10).unwrap();
            assert_eq!(ServiceState::Active, state().unwrap());

            disable(10).unwrap();
            assert_eq!(ServiceState::Inactive, state().unwrap());
        }
        _ => (),
    }
}

/// Gets the state of the Network Manager service.
///
/// # Examples
///
/// ```no_run
/// use network_manager::service;
/// let state = service::state().unwrap();
/// println!("{:?}", state);
/// ```
pub fn state() -> Result<ServiceState, String> {
    let mut message = dbus_message!(SD_SERVICE_MANAGER,
                                    SD_SERVICE_PATH,
                                    SD_MANAGER_INTERFACE,
                                    "GetUnit");
    message.append_items(&["NetworkManager.service".into()]);
    let response = dbus_connect!(message);
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
        total_time += 1;
    }

    Err("service timed out".to_string())
}
