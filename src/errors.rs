error_chain! {
    foreign_links {
        Ascii(::ascii::AsAsciiStrError);
        Utf8(::std::str::Utf8Error);
        DBus(::dbus::Error);
    }

    errors {
        NetworkManager(info: String) {
            description("NetworkManager error")
            display("NetworkManager failure: {}", info)
        }

        SSID(info: String) {
            description("Invalid SSID")
            display("{}", info)
        }

        PreSharedKey(info: String) {
            description("Invalid Pre-Shared-Key")
            display("{}", info)
        }

        DBusAPI(info: String) {
            description("D-Bus API error")
            display("D-Bus failure: {}", info)
        }

        Service
    }
}
