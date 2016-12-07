#[derive(Debug)]
pub struct AccessPoint {
    ssid: String,
    signal: u8,
    security: Vec<Security>,
}

#[derive(Debug)]
enum Security {
    WEP,
    WPA1,
    WPA2,
}

pub fn scan() -> Result<Vec<AccessPoint>, String> {
    let ap1 = AccessPoint {
        ssid: "ap1".to_owned(),
        signal: 60,
        security: vec![Security::WEP],
    };

    let ap2 = AccessPoint {
        ssid: "ap2".to_owned(),
        signal: 92,
        security: vec![Security::WPA1, Security::WPA2],
    };

    Ok(vec![ap1, ap2])
}
