//! # The Network Manager Library
//!
//! The Network Manager Library provides the essential
//! functionality for configuring Network Manager from Rust.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate enum_primitive;

#[macro_use]
mod dbus_helper;
pub mod general;
pub mod wifi;
pub mod service;
pub mod connection;
pub mod device;
