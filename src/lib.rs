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

pub mod status;
pub mod wifi;
pub mod service;
pub mod connection;
pub mod device;
pub mod dbus_nm;
mod dbus_api;
