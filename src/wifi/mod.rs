

/// Scans for Wi-Fi access points.
///
/// # Examples
///
/// ```
/// let access_points = network_manager::wifi::scan().unwrap();
/// println!("{:?}", access_points);
/// ```
pub fn scan() -> Result<Vec<AccessPoint>, String> {
    // Scan for access points

    let ap1 = AccessPoint {
        ssid: "ap1".to_string(),
        signal: 60,
        security: vec![Security::WEP],
    };

    let ap2 = AccessPoint {
        ssid: "ap2".to_string(),
        signal: 92,
        security: vec![Security::WPA1, Security::WPA2],
    };

    Ok(vec![ap1, ap2])
}


#[derive(Debug)]
pub enum Security {
    None,
    WEP,
    WPA1,
    WPA2,
}

#[derive(Debug)]
pub struct AccessPoint {
    ssid: String,
    signal: u8,
    security: Vec<Security>,
}
