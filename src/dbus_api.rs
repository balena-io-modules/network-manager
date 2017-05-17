use dbus::Connection as DBusConnection;
use dbus::{BusType, Path, ConnPath, Message};
use dbus::arg::{Variant, Iter, Array, Get, RefArg};
use dbus::stdintf::OrgFreedesktopDBusProperties;
use dbus::Error;


pub const DEFAULT_TIMEOUT: u64 = 10;
pub const RETRIES_ALLOWED: usize = 10;


pub struct DBusApi {
    connection: DBusConnection,
    method_timeout: u64,
    base: &'static str,
    method_retry_error_names: Vec<&'static str>,
}

impl DBusApi {
    pub fn new(base: &'static str, method_retry_error_names: Vec<&'static str>) -> Self {
        let connection = DBusConnection::get_private(BusType::System).unwrap();

        DBusApi {
            connection: connection,
            method_timeout: DEFAULT_TIMEOUT,
            base: base,
            method_retry_error_names: method_retry_error_names,
        }
    }

    pub fn method_timeout(&self) -> u64 {
        self.method_timeout
    }

    pub fn set_method_timeout(&mut self, timeout: u64) {
        self.method_timeout = timeout;
    }

    pub fn call(&self, path: &str, interface: &str, method: &str) -> Result<Message, String> {
        self.call_with_args(path, interface, method, &[])
    }

    pub fn call_with_args(&self,
                          path: &str,
                          interface: &str,
                          method: &str,
                          args: &[&RefArg])
                          -> Result<Message, String> {
        let call_error = |details: &str| {
            Err(format!("D-Bus '{}'::'{}' method call failed on '{}': {}",
                        interface,
                        method,
                        path,
                        details))
        };

        match self.call_with_args_retry(path, interface, method, args) {
            Ok(response) => Ok(response),
            Err(error) => call_error(&error),
        }
    }

    fn call_with_args_retry(&self,
                            path: &str,
                            interface: &str,
                            method: &str,
                            args: &[&RefArg])
                            -> Result<Message, String> {
        let mut retries = 0;

        loop {
            if let Ok(result) = self.create_and_send_message(path, interface, method, args) {
                return result;
            }

            retries += 1;

            if retries == RETRIES_ALLOWED {
                return Err(format!("method failed after {} retries", RETRIES_ALLOWED));
            }

            ::std::thread::sleep(::std::time::Duration::from_secs(1));
        }
    }

    fn create_and_send_message(&self,
                               path: &str,
                               interface: &str,
                               method: &str,
                               args: &[&RefArg])
                               -> Result<Result<Message, String>, String> {
        match Message::new_method_call(self.base, path, interface, method) {
            Ok(mut message) => {
                if args.len() > 0 {
                    message = message.append_ref(args);
                }

                self.send_message_checked(message)
            }
            Err(details) => Ok(Err(details)),
        }
    }

    fn send_message_checked(&self, message: Message) -> Result<Result<Message, String>, String> {
        match self.connection
                  .send_with_reply_and_block(message, self.method_timeout as i32 * 1000) {
            Ok(response) => Ok(Ok(response)),
            Err(err) => {
                let message = get_error_message(&err).to_string();

                let name = err.name();
                for error_name in &self.method_retry_error_names {
                    if name == Some(error_name) {
                        return Err(message);
                    }
                }

                Ok(Err(message))
            }
        }
    }

    pub fn property<T>(&self, path: &str, interface: &str, name: &str) -> Result<T, String>
        where DBusApi: VariantTo<T>
    {
        let property_error = |details: &str| {
            Err(format!("D-Bus get '{}'::'{}' property failed on '{}': {}",
                        interface,
                        name,
                        path,
                        details))
        };

        let path = self.with_path(path);

        match path.get(interface, name) {
            Ok(variant) => {
                match DBusApi::variant_to(variant) {
                    Some(data) => Ok(data),
                    None => property_error("wrong property type"),
                }
            }
            Err(err) => {
                match err.message() {
                    Some(details) => property_error(details),
                    None => property_error("no details"),
                }
            }
        }
    }

    pub fn extract<'a, T>(&self, response: &'a Message) -> Result<T, String>
        where T: Get<'a>
    {
        response
            .get1()
            .ok_or("D-Bus wrong response type".to_string())
    }

    pub fn extract_two<'a, T1, T2>(&self, response: &'a Message) -> Result<(T1, T2), String>
        where T1: Get<'a>,
              T2: Get<'a>
    {
        let (first, second) = response.get2();

        if let Some(first) = first {
            if let Some(second) = second {
                return Ok((first, second));
            }
        }

        Err("D-Bus wrong response type".to_string())
    }

    fn with_path<'a, P: Into<Path<'a>>>(&'a self, path: P) -> ConnPath<'a, &'a DBusConnection> {
        self.connection
            .with_path(self.base, path, self.method_timeout as i32 * 1000)
    }
}


pub trait VariantTo<T> {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<T>;
}


impl VariantTo<String> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<String> {
        variant_to_string(value)
    }
}


impl VariantTo<i64> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<i64> {
        variant_to_i64(value)
    }
}


impl VariantTo<u32> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<u32> {
        variant_to_u32(value)
    }
}


impl VariantTo<bool> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<bool> {
        variant_to_bool(value)
    }
}


impl VariantTo<Vec<String>> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
        variant_to_string_vec(value)
    }
}


impl VariantTo<Vec<u8>> for DBusApi {
    fn variant_to(value: Variant<Box<RefArg>>) -> Option<Vec<u8>> {
        variant_to_u8_vec(value)
    }
}


fn variant_to_string_vec(value: Variant<Box<RefArg>>) -> Option<Vec<String>> {
    let mut result = Vec::new();

    if let Some(list) = value.0.as_iter() {
        for element in list {
            if let Some(string) = element.as_str() {
                result.push(string.to_string());
            } else {
                return None;
            }
        }

        Some(result)
    } else {
        None
    }
}


fn variant_to_u8_vec(value: Variant<Box<RefArg>>) -> Option<Vec<u8>> {
    let mut result = Vec::new();

    if let Some(list) = value.0.as_iter() {
        for element in list {
            if let Some(value) = element.as_i64() {
                result.push(value as u8);
            } else {
                return None;
            }
        }

        Some(result)
    } else {
        None
    }
}


fn variant_to_string(value: Variant<Box<RefArg>>) -> Option<String> {
    value.0.as_str().and_then(|v| Some(v.to_string()))
}


fn variant_to_i64(value: Variant<Box<RefArg>>) -> Option<i64> {
    value.0.as_i64()
}


fn variant_to_u32(value: Variant<Box<RefArg>>) -> Option<u32> {
    value.0.as_i64().and_then(|v| Some(v as u32))
}


fn variant_to_bool(value: Variant<Box<RefArg>>) -> Option<bool> {
    value.0.as_i64().and_then(|v| Some(v == 0))
}


pub fn extract<'a, T>(var: &'a Variant<Iter>) -> Result<T, String>
    where T: Get<'a>
{
    var.0
        .clone()
        .get::<T>()
        .ok_or(format!("D-Bus variant type does not match: {:?}", var))
}


pub fn utf8_vec_u8_to_string(var: Vec<u8>) -> Result<String, String> {
    String::from_utf8(var).or(Err(format!("D-Bus variant not a UTF-8 string")))
}

pub fn utf8_variant_to_string(var: &Variant<Iter>) -> Result<String, String> {
    let array_option = &var.0.clone().get::<Array<u8, _>>();

    if let Some(array) = *array_option {
        utf8_vec_u8_to_string(array.collect())
    } else {
        Err(format!("D-Bus variant not an array: {:?}", var))
    }
}

pub fn string_to_utf8_vec_u8(var: &String) -> Vec<u8> {
    var.as_bytes().to_vec()
}

pub fn path_to_string(path: &Path) -> Result<String, String> {
    if let Ok(slice) = path.as_cstr().to_str() {
        Ok(slice.to_string())
    } else {
        Err(format!("Path not a UTF-8 string: {:?}", path))
    }
}

fn get_error_message(err: &Error) -> &str {
    match err.message() {
        Some(details) => details,
        None => "Undefined error message",
    }
}
