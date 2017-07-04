//! # The Network Manager Library
//!
//! The Network Manager Library provides the essential
//! functionality for configuring Network Manager from Rust.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;

#[macro_use]
extern crate bitflags;

extern crate dbus;

extern crate ascii;

mod dbus_nm;
mod dbus_api;
mod manager;
mod service;
mod connection;
mod device;
mod wifi;
mod ssid;

pub use manager::{NetworkManager, Connectivity};
pub use connection::{Connection, ConnectionSettings, ConnectionState};
pub use device::{Device, DeviceType, DeviceState};
pub use wifi::AccessPoint;
pub use service::ServiceState;
