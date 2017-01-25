extern crate dbus;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Connection(dbus::Error),
    Props(dbus::Error),
    TimedOut,
    Failed,
    NotFound,
}

