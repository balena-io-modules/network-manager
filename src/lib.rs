//! # The Network Manager Library
//!
//! The Network Manager Library provides the essential
//! functionality for configuring Network Manager from Rust.

#![feature(plugin)]
#![plugin(clippy)]

pub mod general;
pub mod wifi;
pub mod service;
pub mod connection;
pub mod device;
