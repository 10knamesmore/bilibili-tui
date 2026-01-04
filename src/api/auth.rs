//! Authentication API types

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QrcodeData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Deserialize)]
pub struct QrcodePollData {
    pub url: String,
    pub refresh_token: String,
    pub timestamp: i64,
    pub code: i32,
    pub message: String,
}

/// QR code poll status codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QrcodePollStatus {
    /// Waiting for scan (86101)
    Waiting,
    /// Scanned, waiting for confirmation (86090)
    Scanned,
    /// Login successful (0)
    Success,
    /// QR code expired (86038)
    Expired,
    /// Unknown status
    Unknown(i32),
}

impl From<i32> for QrcodePollStatus {
    fn from(code: i32) -> Self {
        match code {
            86101 => QrcodePollStatus::Waiting,
            86090 => QrcodePollStatus::Scanned,
            0 => QrcodePollStatus::Success,
            86038 => QrcodePollStatus::Expired,
            _ => QrcodePollStatus::Unknown(code),
        }
    }
}

pub struct QrcodePollResult {
    pub data: Option<QrcodePollData>,
    pub cookies: Vec<(String, String)>,
}
