use std::ops::Deref;
use std::mem;
use std::str;
use std::fmt;
use std::fmt::Write;
use std::ascii;

use errors::*;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ssid {
    vec: Vec<u8>,
}

impl Ssid {
    pub fn new() -> Self {
        Ssid { vec: Vec::new() }
    }

    pub fn from_bytes<B>(bytes: B) -> Result<Self>
    where
        B: Into<Vec<u8>> + AsRef<[u8]>,
    {
        match bytes.as_ref().as_ssid_slice() {
            Ok(_) => Ok(unsafe { Ssid::from_bytes_unchecked(bytes) }),
            Err(e) => Err(e),
        }
    }

    unsafe fn from_bytes_unchecked<B>(bytes: B) -> Self
    where
        B: Into<Vec<u8>>,
    {
        Ssid { vec: bytes.into() }
    }
}

pub trait IntoSsid: Sized {
    fn into_ssid(self) -> Result<Ssid>;
}

impl Deref for Ssid {
    type Target = SsidSlice;

    #[inline]
    fn deref(&self) -> &SsidSlice {
        unsafe { mem::transmute(&self.vec[..]) }
    }
}

impl fmt::Debug for Ssid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self as &SsidSlice, f)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SsidSlice {
    slice: [u8],
}

pub trait AsSsidSlice {
    fn as_ssid_slice(&self) -> Result<&SsidSlice>;
}

impl SsidSlice {
    pub fn as_str(&self) -> Result<&str> {
        Ok(str::from_utf8(&self.slice)?)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { mem::transmute(&self.slice) }
    }
}

impl fmt::Debug for SsidSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char('"')?;
        for byte in self.as_bytes() {
            for c in ascii::escape_default(*byte) {
                f.write_char(c as char)?;
            }
        }
        f.write_char('"')
    }
}

impl AsSsidSlice for [u8] {
    fn as_ssid_slice(&self) -> Result<&SsidSlice> {
        if self.len() > 32 {
            bail!(ErrorKind::SSID(format!(
                "SSID length should not exceed 32: {} len",
                self.len()
            )))
        } else {
            Ok(unsafe { mem::transmute(self) })
        }
    }
}

impl AsSsidSlice for str {
    fn as_ssid_slice(&self) -> Result<&SsidSlice> {
        self.as_bytes().as_ssid_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssid_from_bytes_as_bytes() {
        let vec_u8 = vec![0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let ssid = Ssid::from_bytes(vec_u8.clone()).unwrap();
        let slice = &ssid as &SsidSlice;
        let as_bytes = slice.as_bytes();
        assert_eq!(vec_u8, as_bytes);
    }

    #[test]
    fn test_ssid_from_bytes_as_str() {
        let vec_u8 = vec![0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let ssid = Ssid::from_bytes(vec_u8.clone()).unwrap();
        let slice = &ssid as &SsidSlice;
        let as_str = slice.as_str().unwrap();
        assert_eq!(vec_u8, as_str.as_bytes());
    }

    #[test]
    fn test_ssid_from_bytes_eq() {
        let from_string = Ssid::from_bytes("hello".to_string()).unwrap();
        let vec_u8 = vec![0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let from_vec_u8 = Ssid::from_bytes(vec_u8).unwrap();
        assert_eq!(from_string, from_vec_u8);
    }

    #[test]
    fn test_as_ssid_slice() {
        let slice_from_str = "hello".as_ssid_slice().unwrap();
        let ssid_u8 = [0x68_u8, 0x65_u8, 0x6c_u8, 0x6c_u8, 0x6f_u8];
        let slice_from_u8 = (&ssid_u8 as &[u8]).as_ssid_slice().unwrap();
        assert_eq!(slice_from_str, slice_from_u8);
    }

    #[test]
    fn test_ssid_debug() {
        let ssid = Ssid::from_bytes(b"hello\0\x7F".to_vec()).unwrap();
        let debug = format!("{:?}", ssid);
        assert_eq!(debug, "\"hello\\x00\\x7f\"");
    }

    #[test]
    fn test_ssid_slice_debug() {
        let ssid = Ssid::from_bytes(b"hello\0\x7F".to_vec()).unwrap();
        let slice = &ssid as &SsidSlice;
        let debug = format!("{:?}", slice);
        assert_eq!(debug, "\"hello\\x00\\x7f\"");
    }
}
