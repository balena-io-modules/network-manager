/// Enables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::enable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn enable(t: i32) -> Result<State, String> {
    // Start service

    if t != 0 {
        // Wait until service has started or
        // until the time has elapsed
    }

    Ok(State::Active)
}

/// Disables the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::disable(10).unwrap();
/// println!("{:?}", state);
/// ```
pub fn disable(t: i32) -> Result<State, String> {
    // Stop service

    if t != 0 {
        // Wait until service has stopped or
        // until the time has elapsed
    }

    Ok(State::Inactive)
}

/// Gets the state of the Network Manager service.
///
/// # Examples
///
/// ```
/// let state = network_manager::service::state().unwrap();
/// println!("{:?}", state);
/// ```
pub fn state() -> Result<State, String> {
    // Get service state

    Ok(State::Failed)
}

#[derive(Debug)]
pub enum State {
    Active,
    Reloading,
    Inactive,
    Failed,
    Activating,
    Deactivating,
}
