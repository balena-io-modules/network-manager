//! # The Network Manager Library
//!
//! The Network Manager Library provides the essential
//! functionality for configuring Network Manager from Rust.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate enum_primitive;

#[macro_use]
extern crate bitflags;

extern crate dbus;

mod dbus_nm;
mod dbus_api;
mod manager;
mod service;
mod connection;
mod device;
mod wifi;

pub use manager::NetworkManager;
pub use connection::{Connection, ConnectionSettings};
pub use device::{Device, DeviceType};
