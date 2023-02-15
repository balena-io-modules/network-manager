//! # The Network Manager Library
//!
//! The Network Manager Library provides the essential
//! functionality for configuring Network Manager from Rust.

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;

#[macro_use]
extern crate bitflags;

extern crate dbus;

extern crate ascii;

pub mod errors;

mod connection;
mod dbus_api;
mod dbus_nm;
mod device;
mod manager;
mod service;
mod ssid;
mod wifi;

pub use connection::{Connection, ConnectionSettings, ConnectionState};
pub use device::{Device, DeviceState, DeviceType};
pub use manager::{Connectivity, NetworkManager};
pub use service::ServiceState;
pub use wifi::{AccessPoint, AccessPointCredentials, Security};
