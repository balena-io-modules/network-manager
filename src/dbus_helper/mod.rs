extern crate dbus;

macro_rules! dbus_message {
    ($service:expr, $path:expr, $interface:expr, $function:expr) => {{
       dbus::Message::new_method_call($service, $path, $interface, $function).
           unwrap()
    }}
}

macro_rules! dbus_property {
   ($service:expr, $path:expr, $interface:expr, $property:expr) => {{
        let connection = dbus::Connection::get_private(dbus::BusType::System).unwrap();
        dbus::Props::new(&connection, $service, $path, $interface, 2000).
            get($property).unwrap()
    }}
}

macro_rules! dbus_connect {
    ($message:expr) => {{
        dbus::Connection::get_private(dbus::BusType::System).unwrap().
            send_with_reply_and_block($message, 2000)
    }}
}
