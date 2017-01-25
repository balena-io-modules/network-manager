extern crate dbus;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Connection(dbus::Error),
    Props(dbus::Error),
    TimedOut,
    Failed,
    NotFound,
}

