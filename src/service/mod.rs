use general::ServiceState;

/// Enables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(t: i32) -> Result<ServiceState, String> {
    // Enable service

    if t != 0 {
        // Wait until service has started or
        // until the time has elapsed
    }

    Ok(ServiceState::Active)
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(t: i32) -> Result<ServiceState, String> {
    // Disable service

    if t != 0 {
        // Wait until service has stopped or
        // until the time has elapsed
    }

    Ok(ServiceState::Inactive)
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
    // Get service state

    Ok(ServiceState::Failed)
}
